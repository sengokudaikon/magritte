use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use surrealdb::{Surreal, Value};
use surrealdb::engine::any::Any;
use tracing::instrument;
use crate::database::SurrealDB;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbInfo {
    pub accesses: HashMap<String, String>,
    pub analyzers: HashMap<String, String>,
    pub configs: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    pub models: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub tables: HashMap<String, String>,
    pub users: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableInfo {
    pub events: HashMap<String, String>,
    pub fields: HashMap<String, String>,
    pub indexes: HashMap<String, String>,
    pub lives: HashMap<String, String>,
    pub tables: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct InfoStatement {
    conn: Surreal<Any>,
}
impl InfoStatement {
    pub fn new(conn: Surreal<Any>) -> Self {
        Self { conn }
    }
    /// Get root level info (namespaces and users)
    #[instrument(skip(self))]
    pub async fn info_root(&self) -> anyhow::Result<JsonValue> {
        let result: Value = self.conn.query("INFO FOR ROOT").await?.take(0)?;
        Ok(serde_json::to_value(result)?)
    }

    /// Get namespace level info (databases, users, access)
    #[instrument(skip(self))]
    pub async fn info_ns(&self) -> anyhow::Result<JsonValue> {
        let result: Value = self.conn.query("INFO FOR NS").await?.take(0)?;
        Ok(serde_json::to_value(result)?)
    }

    /// Get database level info (tables, functions, users etc)
    #[instrument(skip(self))]
    pub async fn info_db(&self) -> anyhow::Result<DbInfo> {
        let result: Option<DbInfo> = self.conn.query("INFO FOR DB").await?.take(0)?;
        let db_info = result.ok_or(anyhow!("Could not deserialize DbInfo"))?;
        Ok(db_info)
    }

    /// Get Table level info (fields, indexes, events)
    #[instrument(skip(self))]
    pub async fn info_table(&self, table: &str) -> anyhow::Result<TableInfo> {
        let mut query = String::from("INFO FOR TABLE ");
        query.push_str(table);
        let result: Option<TableInfo> = self.conn.query(query).await?.check()?.take(0)?;
        println!(
            "Info for {}: {}",
            table,
            serde_json::to_string_pretty(&result)?
        );
        result.ok_or(anyhow!("Could not deserialize TableInfo"))
    }

    /// Get user info at specified level
    #[instrument(skip(self))]
    pub async fn info_user(&self, user: &str, level: Option<&str>) -> anyhow::Result<JsonValue> {
        let query = match level {
            Some(level) => format!("INFO FOR USER {} ON {}", user, level),
            None => format!("INFO FOR USER {}", user),
        };

        let result: Value = self.conn.query(query).await?.take(0)?;
        Ok(serde_json::to_value(result)?)
    }

    /// Get index info
    #[instrument(skip(self))]
    pub async fn info_index(&self, index: &str, table: &str) -> anyhow::Result<JsonValue> {
        let result: Value = self
            .conn
            .query("INFO FOR INDEX $index ON TABLE $Table")
            .bind(("index", index.to_owned()))
            .bind(("Table", table.to_owned()))
            .await?
            .take(0)?;
        Ok(serde_json::to_value(result)?)
    }
}
