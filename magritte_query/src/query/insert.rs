//! INSERT query operations
//!
//! This module contains operations related to inserting records into tables.

use std::fmt::Debug;
use std::time::Duration;

use crate::transaction::Transactional;
use crate::{FromTarget, HasParams, RecordType, ReturnType, SelectStatement, SurrealId};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info, instrument};
use crate::database::{QueryType, SurrealDB};

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    /// Direct content value
    Value(serde_json::Value),
    /// SET operations
    Set(Vec<(String, serde_json::Value)>),
}
/// INSERT query builder with allowed method chains
#[derive(Clone, Debug, PartialEq)]
pub struct InsertStatement<T>
where
    T: RecordType,
{
    targets: Option<Vec<FromTarget<T>>>,
    with_id: Option<SurrealId<T>>,
    only: bool,
    content: Option<Content>,
    parameters: Vec<(String, serde_json::Value)>,
    parallel: bool,
    timeout: Option<Duration>,
    return_type: Option<ReturnType>,
    as_relation: bool,
    ignore: bool,
    in_transaction: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for InsertStatement<T>
where
    T: RecordType,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InsertStatement<T>
where
    T: RecordType,
{
    pub fn where_id(mut self, id: SurrealId<T>) -> Self {
        self.with_id = Some(id);
        self
    }
    pub fn content(mut self, content: &T) -> Result<Self> {
        self.content = Some(Content::Value(serde_json::to_value(content)?));
        Ok(self)
    }
    #[instrument(skip_all)]
    pub fn values<C>(mut self, values: Vec<C>) -> Result<Self>
    where
        C: Serialize,
    {
        let values = values
            .into_iter()
            .map(|v| serde_json::to_value(v))
            .collect::<Result<Vec<_>, _>>()?;
        self.content = Some(Content::Value(serde_json::Value::Array(values)));
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE clause
    #[instrument(skip_all)]
    pub fn on_duplicate_key_update<V: Serialize>(mut self, field: &str, value: V) -> Result<Self> {
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((field.to_string(), serde_json::to_value(value)?));
            }
            _ => {
                self.content = Some(Content::Set(vec![(
                    field.to_string(),
                    serde_json::to_value(value)?,
                )]));
            }
        }
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE with increment
    #[instrument(skip_all)]
    pub fn on_duplicate_key_increment(mut self, field: &str, value: i64) -> Result<Self> {
        let expr = format!("{} += {}", field, value);
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((expr, serde_json::Value::Null));
            }
            _ => {
                self.content = Some(Content::Set(vec![(expr, serde_json::Value::Null)]));
            }
        }
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE with decrement
    #[instrument(skip_all)]
    pub fn on_duplicate_key_decrement(mut self, field: &str, value: i64) -> Result<Self> {
        let expr = format!("{} -= {}", field, value);
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((expr, serde_json::Value::Null));
            }
            _ => {
                self.content = Some(Content::Set(vec![(expr, serde_json::Value::Null)]));
            }
        }
        Ok(self)
    }

    /// Insert as a relation
    #[instrument(skip(self))]
    pub fn as_relation(mut self) -> Self {
        self.as_relation = true;
        self
    }

    /// Insert with IGNORE option
    #[instrument(skip(self))]
    pub fn ignore(mut self) -> Self {
        self.ignore = true;
        self
    }
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }
    pub fn new() -> Self {
        Self {
            targets: None,
            with_id: None,
            only: false,
            content: None,
            parameters: vec![],
            parallel: false,
            timeout: None,
            return_type: Default::default(),
            as_relation: false,
            ignore: false,
            in_transaction: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn build(&self) -> Result<String> {
        let mut query = String::new();
        query.push_str("INSERT ");

        if self.only {
            query.push_str("ONLY ");
        } else if let Some(targets) = &self.targets {
            if !targets.is_empty() {
                for v in targets {
                    query.push_str(&format!("{},", v.to_string().as_str()));
                }
            }
        }

        query.push_str("INTO ");
        query.push_str(T::table_name());

        if self.as_relation {
            query.push_str(" RELATION ");
        }
        if self.ignore {
            query.push_str("IGNORE ");
        }

        if let Some(content) = &self.content {
            match content {
                Content::Value(value) => {
                    query.push(' ');
                    query.push_str(&process_value(value)?);
                }
                Content::Set(sets) => {
                    query.push_str(" (");
                    query.push_str(
                        &sets
                            .iter()
                            .map(|(field, _)| {
                                field
                                    .strip_prefix("\"")
                                    .unwrap()
                                    .strip_suffix("\"")
                                    .unwrap()
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    query.push_str(") VALUES (");
                    query.push_str(
                        &sets
                            .iter()
                            .map(|(field, value)| {
                                if field == "id" {
                                    strip_id(value)
                                        .unwrap_or_else(|_| serde_json::to_string(value).unwrap())
                                } else {
                                    serde_json::to_string(value).unwrap()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    query.push(')');

                    // Add ON DUPLICATE KEY UPDATE if present
                    if !sets.is_empty() {
                        query.push_str(" ON DUPLICATE KEY UPDATE ");
                        query.push_str(
                            &sets
                                .iter()
                                .map(|(field, value)| {
                                    format!("{} = {}", field, serde_json::to_string(value).unwrap())
                                })
                                .collect::<Vec<_>>()
                                .join(", "),
                        );
                    }
                }
            }
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

        // Add TIMEOUT if specified
        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_millis()));
        }

        // Add PARALLEL if specified
        if self.parallel {
            query.push_str(" PARALLEL");
        }

        query.push(';');
        Ok(query)
    }

    pub async fn execute(self, conn: &SurrealDB) -> Result<Vec<T>> {
        conn.execute(self.build()?, self.parameters, QueryType::Write, Some(T::table_name().to_string())).await
    }
}
impl<T> HasParams for InsertStatement<T>
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

fn unquote_keys(value: &serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let value_str = match (k.as_str(), v) {
                        ("id", serde_json::Value::Object(id_obj)) => {
                            // Handle the special case for id
                            if let Some(serde_json::Value::Object(inner_obj)) = id_obj.get("id") {
                                if let Some(id_value) = inner_obj.values().next() {
                                    return Ok(format!("id: {}", value_to_string(id_value)?));
                                }
                            }
                            Err(anyhow!("Invalid id format"))
                        }
                        (_, v) => value_to_string(v),
                    }?;
                    Ok(format!("{}: {}", k, value_str))
                })
                .collect::<Result<Vec<String>>>()?;
            Ok(format!("{{{}}}", entries.join(", ")))
        }
        serde_json::Value::Array(arr) => {
            let values: Vec<String> = arr
                .iter()
                .map(unquote_keys)
                .collect::<Result<Vec<String>>>()?;
            Ok(format!("[{}]", values.join(", ")))
        }
        _ => value_to_string(value),
    }
}
fn value_to_string(value: &serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::String(s) => Ok(format!("\"{}\"", s)),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Null => Ok("null".to_string()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => unquote_keys(value),
    }
}
fn strip_id(value: &serde_json::Value) -> Result<String> {
    if let serde_json::Value::Object(id_obj) = value {
        if let Some(serde_json::Value::Object(inner_obj)) = id_obj.get("id") {
            if let Some(id_value) = inner_obj.values().next() {
                return value_to_string(id_value);
            }
        }
    }
    Err(anyhow!("Invalid id format"))
}
fn process_value(value: &serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let value_str = if k == "id" {
                        strip_id(v).unwrap_or_else(|_| serde_json::to_string(v).unwrap())
                    } else {
                        serde_json::to_string(v).unwrap()
                    };
                    Ok(format!("{}: {}", k, value_str))
                })
                .collect::<Result<Vec<String>>>()?;
            Ok(format!("{{{}}}", entries.join(", ")))
        }
        _ => Err(anyhow!("Expected an object for INSERT")),
    }
}
impl<T> Transactional for InsertStatement<T>
where
    T: RecordType,
{
    fn is_transaction(&self) -> bool {
        self.in_transaction
    }

    fn in_transaction(&mut self) -> &mut bool {
        &mut self.in_transaction
    }
}
