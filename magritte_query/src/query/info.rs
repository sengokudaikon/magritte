use crate::SurrealDB;
use serde_json::Value as JsonValue;
use surrealdb::Value;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct InfoStatement {
    conn: SurrealDB,
}
impl InfoStatement
{
    pub fn new (conn: SurrealDB)-> Self {
        Self {
            conn
        }
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
    pub async fn info_db(&self) -> anyhow::Result<JsonValue> {
        let result: Value = self.conn.query("INFO FOR DB").await?.take(0)?;
        Ok(serde_json::to_value(result)?)
    }

    /// Get Table level info (fields, indexes, events)
    #[instrument(skip(self))]
    pub async fn info_table(&self, table: &str) -> anyhow::Result<JsonValue> {
        let result: Value =
            self.conn.query("INFO FOR TABLE $Table").bind(("Table", table.to_owned())).await?.take(0)?;
        Ok(serde_json::to_value(result)?)
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
