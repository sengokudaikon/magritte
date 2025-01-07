use std::time::Duration;

use crate::transaction::Transactional;
use crate::{ReturnType, Returns, SurrealDB};
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// Builder for RELATE statements
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RelateStatement {
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
    in_transaction: bool,
}

impl RelateStatement {
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

    pub fn from_record(mut self, record: &str) -> Self {
        self.from_record = record.to_string();
        self
    }

    pub fn to_record(mut self, record: &str) -> Self {
        self.to_record = record.to_string();
        self
    }

    pub fn edge_table(mut self, table: &str) -> Self {
        self.edge_table = table.to_string();
        self
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> Result<String> {
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

    pub async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
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

impl Returns for RelateStatement {
    fn return_type_mut(&mut self) -> &mut Option<ReturnType> {
        &mut self.return_type
    }
}
impl Transactional for RelateStatement {
    fn is_transaction(&self) -> bool {
        self.in_transaction
    }

    fn in_transaction(&mut self) -> &mut bool {
        &mut self.in_transaction
    }
}
