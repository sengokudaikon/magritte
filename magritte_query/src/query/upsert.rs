use crate::database::{QueryType, SurrealDB};
use crate::{
    FromTarget, HasConditions, HasParams, Operator, RecordType, ReturnType, Returns, SqlValue,
    SurrealId,
};
use serde::Serialize;
use serde_json::Value;
use std::marker::PhantomData;
use std::time::Duration;
use tracing::instrument;

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Content(serde_json::Value),
    /// MERGE operations (for UPDATE)
    Merge(serde_json::Value),
    /// PATCH operations (for UPDATE)
    Patch(serde_json::Value),
    /// REPLACE operations (for UPDATE)
    Replace(Vec<(String, Option<serde_json::Value>)>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpsertStatement<T>
where
    T: RecordType,
{
    targets: Option<Vec<FromTarget<T>>>,
    with_id: Option<SurrealId<T>>,
    only: bool,
    content: Option<Content>,
    conditions: Vec<(String, Operator, SqlValue)>,
    parameters: Vec<(String, serde_json::Value)>,
    parallel: bool,
    timeout: Option<Duration>,
    return_type: Option<ReturnType>,
    in_transaction: bool,
    _marker: PhantomData<T>,
}

impl<T> Default for UpsertStatement<T>
where
    T: RecordType,
{
    fn default() -> Self {
        Self {
            targets: None,
            with_id: None,
            only: false,
            content: None,
            conditions: vec![],
            parameters: vec![],
            parallel: false,
            timeout: None,
            return_type: None,
            in_transaction: false,
            _marker: PhantomData,
        }
    }
}

impl<T> UpsertStatement<T>
where
    T: RecordType,
{
    #[instrument(skip_all)]
    pub fn content<C: Serialize>(mut self, content: &C) -> anyhow::Result<Self> {
        self.content = Some(Content::Content(serde_json::to_value(content)?));
        Ok(self)
    }

    pub fn where_id(mut self, id: SurrealId<T>) -> Self {
        self.with_id = Some(id);
        self
    }

    /// Add a MERGE operation for UPSERT
    #[instrument(skip_all)]
    pub fn merge<V: Serialize>(mut self, value: V) -> anyhow::Result<Self> {
        self.content = Some(Content::Merge(serde_json::to_value(value)?));
        Ok(self)
    }

    #[instrument(skip_all)]
    pub fn patch<V: Serialize>(mut self, value: V) -> anyhow::Result<Self> {
        self.content = Some(Content::Patch(serde_json::to_value(value)?));
        Ok(self)
    }

    #[instrument(skip_all)]
    pub fn replace<V: Serialize>(
        mut self,
        replacements: impl IntoIterator<Item = (String, Option<V>)>,
    ) -> anyhow::Result<Self> {
        let replacements: anyhow::Result<Vec<(String, Option<serde_json::Value>)>> = replacements
            .into_iter()
            .map(|(field, value)| Ok((field, value.map(|v| serde_json::to_value(v)).transpose()?)))
            .collect();

        match &mut self.content {
            Some(Content::Replace(sets)) => sets.extend(replacements?),
            _ => self.content = Some(Content::Replace(replacements?)),
        }

        Ok(self)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let mut query = String::new();
        query.push_str("UPSERT ");
        if self.only {
            query.push_str("ONLY ");
        } else if let Some(targets) = &self.targets {
            if !targets.is_empty() {
                for v in targets {
                    query.push_str(&format!("{},", v.to_string().as_str()));
                }
            }
        }

        if let Some(content) = &self.content {
            match content {
                Content::Content(value) => {
                    query.push_str(" CONTENT ");
                    query.push_str(&value.to_string());
                }
                Content::Merge(value) => {
                    query.push_str(" MERGE ");
                    query.push_str(&value.to_string());
                }
                Content::Patch(value) => {
                    query.push_str(" PATCH ");
                    query.push_str(&value.to_string());
                }
                Content::Replace(sets) => {
                    query.push_str(" REPLACE ");
                    let has_unset = sets.iter().any(|(_, value)| value.is_none());

                    if has_unset {
                        // Use SET/UNSET syntax
                        let mut set_strs = Vec::new();
                        let mut unset_strs = Vec::new();

                        for (field, value) in sets {
                            if let Some(v) = value {
                                set_strs.push(format!("SET {} = {}", field, v));
                            } else {
                                unset_strs.push(format!("UNSET {}", field));
                            }
                        }

                        query.push_str(&set_strs.join(", "));
                        if !set_strs.is_empty() && !unset_strs.is_empty() {
                            query.push_str(", ");
                        }
                        query.push_str(&unset_strs.join(", "));
                    } else {
                        // Use JSON object syntax
                        let set_obj: serde_json::Map<String, serde_json::Value> = sets
                            .iter()
                            .filter_map(|(field, value)| {
                                value.as_ref().map(|v| (field.clone(), v.clone()))
                            })
                            .collect();
                        query.push_str(&serde_json::to_string(&set_obj).unwrap_or_default());
                    }
                }
            }
        }
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .conditions
                .iter()
                .map(|(field, op, value)| format!("{} {} {}", field, String::from(*op), value))
                .collect();
            query.push_str(&conditions.join(" AND "));
        }

        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }
        if let Some(return_type) = &self.return_type {
            match return_type {
                ReturnType::All => query.push_str(" RETURN AFTER"),
                ReturnType::None => query.push_str(" RETURN NONE"),
                ReturnType::Before => query.push_str(" RETURN BEFORE"),
                ReturnType::After => query.push_str(" RETURN AFTER"),
                ReturnType::Diff => query.push_str(" RETURN DIFF"),
                ReturnType::Fields(fields) => {
                    query.push_str(" RETURN ");
                    query.push_str(&fields.join(", "));
                }
            }
        }

        if self.parallel {
            query.push_str(" PARALLEL");
        }
        query.push(';');
        Ok(query)
    }

    async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<T>> {
        conn.execute(self.build()?, self.parameters, QueryType::Write, Some(T::table_name().to_string())).await
    }
}
impl<T> Returns for UpsertStatement<T>
where
    T: RecordType,
{
    fn return_type_mut(&mut self) -> &mut Option<ReturnType> {
        &mut self.return_type
    }
}

impl<T> HasParams for UpsertStatement<T>
where
    T: RecordType,
{
    fn params(&self) -> &Vec<(String, Value)> {
        &self.parameters
    }

    fn params_mut(&mut self) -> &mut Vec<(String, Value)> {
        &mut self.parameters
    }
}

impl<T> HasConditions for UpsertStatement<T>
where
    T: RecordType,
{
    fn conditions_mut(&mut self) -> &mut Vec<(String, Operator, SqlValue)> {
        &mut self.conditions
    }
}
