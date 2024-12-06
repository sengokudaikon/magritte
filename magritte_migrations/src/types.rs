use crate::diff::SchemaDiff;
use magritte::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, Table)]
#[table(name = "_migrations")]
pub struct MigrationRecord
{
    pub id: SurrealId<Self>,
    pub name: String,         // Human readable name
    pub version: String,      // Semantic version or timestamp
    pub checksum: String,     // Hash of migration content
    pub status: MigrationStatus,
    #[column(type="datetime")]
    pub applied_at: Option<OffsetDateTime>,
    pub execution_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub dependencies: Vec<String>,
    #[column(type="any")]
    pub schema_diff: SchemaDiff
}

impl  MigrationRecord
{
    pub fn new(
        id: impl Into<SurrealId<Self>>,
        name: String,
        version: String,
        checksum: String,
        schema_diff: SchemaDiff

    ) -> Self {
        MigrationRecord {
            id,
            name,
            version,
            checksum,
            status: MigrationStatus::Pending,
            applied_at: None,
            execution_time_ms: None,
            error_message: None,
            dependencies: vec![],
            schema_diff
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationContext {
    pub db: SurrealDB,
    pub namespace: String,
    pub database: String,
}

/// Core trait for implementing migrations
#[async_trait::async_trait]
pub trait Migration: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn dependencies(&self) -> &[String] {
        &[]  // Default to no dependencies
    }
    
    async fn up(&self, ctx: &MigrationContext) -> anyhow::Result<()>;
    async fn down(&self, ctx: &MigrationContext) -> anyhow::Result<()>;
    
    async fn validate(&self, ctx: &MigrationContext) -> anyhow::Result<()> {
        Ok(()) // Default no-op validation
    }
    fn get_previous_version(&self) -> Result<String>;
    fn description(&self) -> Option<&str>;
}