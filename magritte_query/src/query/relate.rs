use std::marker::PhantomData;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::backend::QueryBuilder;
use crate::returns::Returns;
use crate::types::{RecordType, ReturnType, TableType};
use crate::SurrealDB;

/// Builder for RELATE statements
#[derive(Clone, Debug, PartialEq)]
pub struct RelateStatement<T> {
    from_record: String,
    to_record: String,
    edge_table: String,
    only: bool,
    content: Option<Value>,
    set_fields: Vec<(String, Value)>,
    return_type: Option<ReturnType>,
    return_fields: Option<Vec<String>>,
    timeout: Option<Duration>,
    parallel: bool,
    phantom: PhantomData<T>,
}

#[async_trait]
impl<T> QueryBuilder<T> for RelateStatement<T>
where
    T: RecordType
{
    fn new() -> Self {
        Self {
            from_record: String::new(),
            to_record: String::new(),
            edge_table: String::new(),
            only: false,
            content: None,
            set_fields: Vec::new(),
            return_type: None,
            return_fields: None,
            timeout: None,
            parallel: false,
            phantom: PhantomData,
        }
    }

    fn build(&self) -> Result<String> {
        let mut query = String::new();

        // Basic RELATE structure
        query.push_str("RELATE ");
        if self.only {
            query.push_str("ONLY ");
        }
        query.push_str(&format!(
            "{}->{}->{}",
            self.from_record, self.edge_table, self.to_record
        ));

        // Add CONTENT if present
        if let Some(content) = &self.content {
            query.push_str(&format!(" CONTENT {}", content));
        }

        // Add SET fields
        if !self.set_fields.is_empty() {
            query.push_str(" SET ");
            let fields: Vec<String> = self
                .set_fields
                .iter()
                .map(|(field, value)| format!("{} = {}", field, value))
                .collect();
            query.push_str(&fields.join(", "));
        }

        // Add RETURN clause
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

        // Add TIMEOUT
        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }

        // Add PARALLEL
        if self.parallel {
            query.push_str(" PARALLEL");
        }

        query.push(';');
        Ok(query)
    }

    async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<T>> {
        let query = self.build()?;
        let surreal_query = conn.query(query);

        // Execute query and handle response
        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

impl<T> RelateStatement<T>
where
    T: RecordType
{
    /// Set ONLY flag for single relation
    pub fn only(mut self) -> Self {
        self.only = true;
        self
    }

    /// Set content for the relation
    pub fn content<V: Serialize>(mut self, content: V) -> anyhow::Result<Self> {
        self.content = Some(serde_json::to_value(content)?);
        Ok(self)
    }

    /// Set a field value
    pub fn set<V: Serialize>(mut self, field: &str, value: V) -> anyhow::Result<Self> {
        self.set_fields
            .push((field.to_string(), serde_json::to_value(value)?));
        Ok(self)
    }

    /// Add timeout duration
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Enable parallel processing
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }
}

impl<T> Returns for RelateStatement<T>
where
    T: RecordType
{
    fn return_type_mut(&mut self) -> &mut Option<ReturnType> {
        &mut self.return_type
    }
}
