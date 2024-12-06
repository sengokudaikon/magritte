use anyhow::Result;
use magritte::prelude::{EdgeTrait, SurrealDB, TableTrait};
use crate::schema::snapshot::SchemaSnapshot;
use crate::types::MigrationContext;

/// Manager for handling schema operations and migrations
pub struct SchemaManager {
    ctx: MigrationContext,
}

impl SchemaManager {
    pub fn new(db: SurrealDB, namespace: String, database: String) -> Self {
        Self {
            ctx: MigrationContext {
                db,
                namespace,
                database,
            }
        }
    }

    /// Get the current schema snapshot
    pub async fn get_current_schema<T, E> (&self) -> Result<SchemaSnapshot<T, E>> where T: TableTrait, E: EdgeTrait{
        SchemaSnapshot::capture(&self.ctx).await
    }

    /// Check if a table exists
    pub async fn has_table(&self, table: &str) -> Result<bool> {
        let result: Option<serde_json::Value> = self.ctx.db
            .query("INFO FOR TABLE $table")
            .bind(("table", table))
            .await?
            .take(0)?;
        
        Ok(result.is_some())
    }

    /// Check if a field exists on a table
    pub async fn has_field(&self, table: &str, field: &str) -> Result<bool> {
        let result: Option<serde_json::Value> = self.ctx.db
            .query("INFO FOR FIELD $field ON TABLE $table")
            .bind(("table", table))
            .bind(("field", field))
            .await?
            .take(0)?;
        
        Ok(result.is_some())
    }

    /// Check if an index exists on a table
    pub async fn has_index(&self, table: &str, index: &str) -> Result<bool> {
        let result: Option<serde_json::Value> = self.ctx.db
            .query("INFO FOR INDEX $index ON TABLE $table")
            .bind(("table", table))
            .bind(("index", index))
            .await?
            .take(0)?;
        
        Ok(result.is_some())
    }

    /// Check if an event exists on a table
    pub async fn has_event(&self, table: &str, event: &str) -> Result<bool> {
        let result: Option<serde_json::Value> = self.ctx.db
            .query("INFO FOR EVENT $event ON TABLE $table")
            .bind(("table", table))
            .bind(("event", event))
            .await?
            .take(0)?;
        
        Ok(result.is_some())
    }

    /// Execute a schema statement
    pub async fn execute(&self, stmt: impl AsRef<str>) -> Result<()> {
        self.ctx.db.query(stmt.as_ref()).await?;
        Ok(())
    }

    /// Get the migration context
    pub fn context(&self) -> &MigrationContext {
        &self.ctx
    }

    /// Save current schema state to a snapshot file
    pub async fn save_snapshot(&self, base_dir: impl AsRef<std::path::Path>) -> Result<()> {
        let snapshot = self.get_current_schema().await?;
        snapshot.save(&self.ctx, base_dir).await
    }

    /// Load a schema snapshot from a file
    pub fn load_snapshot(path: impl AsRef<std::path::Path>) -> Result<SchemaSnapshot> {
        SchemaSnapshot::load_from_file(path)
    }
}
