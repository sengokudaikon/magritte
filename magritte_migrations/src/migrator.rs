use crate::schema::snapshot::SchemaSnapshot;
use crate::types::{Migration, MigrationContext, MigrationRecord, MigrationStatus};
use anyhow::{anyhow, bail, Result};
use magritte::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use time::OffsetDateTime;
use crate::diff::SchemaDiff;
use crate::MigrationRecordColumns::SchemaDiff;

pub struct Migrator {
    ctx: MigrationContext,
    migrations_dir: String,
    migrations: HashMap<String, Box<dyn Migration>>,
}

impl Migrator {
    pub fn new(ctx: MigrationContext, migrations_dir: PathBuf) -> Self {
        Self {
            ctx,
            migrations_dir: migrations_dir.to_string_lossy().to_string(),
            migrations: HashMap::new(),
        }
    }

    /// Register a migration with the migrator
    pub fn register<M: Migration + 'static>(&mut self, migration: M) {
        self.migrations
            .insert(migration.id().to_string(), Box::new(migration));
    }

    /// Get the current status of all migrations
    pub async fn status(&self) -> Result<Vec<MigrationRecord>> {
        let records: Vec<MigrationRecord> = self
            .ctx
            .db
            .query("SELECT * FROM _migrations ORDER BY version")
            .await?
            .take(0)?;
        Ok(records)
    }

    /// Apply pending migrations up to a specific version
    pub async fn up(&self, target_version: Option<&str>) -> Result<()> {
        let applied = self.get_applied_migrations().await?;
        let pending = self.get_pending_migrations(&applied)?;

        let to_apply = if let Some(target) = target_version {
            pending
                .into_iter()
                .take_while(|m| m.version() <= target)
                .collect::<Vec<_>>()
        } else {
            pending
        };

        let sorted = self.sort_by_dependencies(&to_apply)?;

        for migration in sorted {
            self.apply_migration(migration.as_ref()).await?;
        }

        Ok(())
    }

    pub async fn down(&self, target_version: Option<&str>) -> Result<()> {
        let applied = self.get_applied_migrations().await?;

        let to_rollback = if let Some(target) = target_version {
            applied
                .into_iter()
                .filter(|r| r.version.as_str() > target && r.status == MigrationStatus::Completed)
                .collect::<Vec<_>>()
        } else {
            applied
                .into_iter()
                .filter(|r| r.status == MigrationStatus::Completed)
                .take(1)
                .collect::<Vec<_>>()
        };

        for record in to_rollback.into_iter().rev() {
            if let Some(migration) = self.migrations.get(&record.id.to_string()) {
                self.rollback_migration(migration.as_ref()).await?;
            }
        }

        Ok(())
    }

    /// Generate a new migration based on entity definitions
    pub async fn generate<T: TableTrait>(
        &self,
        name: &str,
        entity: T,
        description: Option<&str>,
    ) -> Result<Box<dyn Migration>> {
        // Capture current schema
        let current = SchemaSnapshot::capture(&self.ctx).await?;

        // Create target schema from the entity
        let target = SchemaSnapshot::from_entities(std::iter::once(entity))?;

        // Generate diff
        let diff = current.diff(&target);

        // Create version using timestamp
        let version = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

        // Generate migration
        Ok(Box::new(MigrationRecord::new(
            format!("{}_{}", version, name),
            name.to_string(),
            version,
            description.map(String::from).unwrap_or("".to_string()),
            diff
        )))
    }

    /// Generate a new migration for multiple entities
    pub async fn generate_multi<I>(
        &self,
        name: &str,
        entities: I,
        description: Option<&str>,
    ) -> Result<Box<dyn Migration>>
    where
        I: IntoIterator<Item = impl TableTrait>,
    {
        // Capture current schema
        let current = SchemaSnapshot::capture(&self.ctx).await?;

        // Create target schema from entities
        let target = SchemaSnapshot::from_entities(entities)?;

        // Generate diff
        let diff = current.diff(&target);

        // Create version using timestamp
        let version = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

        // Generate migration
        Ok(Box::new(MigrationRecord::new(
            format!("{}_{}", version, name),
            name.to_string(),
            version,
            description.map(String::from).unwrap_or("".to_string()),
            diff
        )))
    }

    /// Apply a migration with proper validation and rollback support
    pub async fn apply_migration(&self, migration: &dyn Migration) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Start transaction
        let mut tx_statements = vec![
            "BEGIN TRANSACTION;".to_string(),
            self.create_status_update_statement(migration, MigrationStatus::Running, None, None)?,
        ];

        // Execute migration within transaction
        match self.execute_statements(&tx_statements).await {
            Ok(()) => {
                // Validate the changes
                if let Err(e) = self.validate_migration(migration).await {
                    // Rollback on validation failure
                    self.ctx.db.query("CANCEL TRANSACTION;").await?;

                    // Record failure
                    self.update_migration_status(
                        migration,
                        MigrationStatus::Failed,
                        Some(e.to_string()),
                        Some(start_time.elapsed().as_millis() as i64),
                    )
                    .await?;

                    return Err(e);
                }

                // Commit transaction
                self.ctx.db.query("COMMIT TRANSACTION;").await?;

                // Record success
                self.update_migration_status(
                    migration,
                    MigrationStatus::Completed,
                    None,
                    Some(start_time.elapsed().as_millis() as i64),
                )
                .await?;

                Ok(())
            }
            Err(e) => {
                // Rollback on execution failure
                self.ctx.db.query("CANCEL TRANSACTION;").await?;

                // Record failure
                self.update_migration_status(
                    migration,
                    MigrationStatus::Failed,
                    Some(e.to_string()),
                    Some(start_time.elapsed().as_millis() as i64),
                )
                .await?;

                Err(e)
            }
        }
    }

    /// Validate a migration after applying it
    async fn validate_migration(&self, migration: &dyn Migration) -> Result<()> {
        // Capture current schema
        let current = SchemaSnapshot::capture(&self.ctx).await?;

        // Get expected schema
        let expected = migration.target_schema()?;

        // Compare schemas
        let diff = current.diff(&expected);

        // Build detailed validation error if needed
        let mut validation_error = ValidationError {
            unexpected_tables: diff.added_tables.clone(),
            missing_tables: diff.removed_tables.clone(),
            modified_tables: HashMap::new(),
            modified_edges: HashMap::new(),
        };

        // Check for unexpected differences
        let mut has_errors = false;

        if !validation_error.unexpected_tables.is_empty()
            || !validation_error.missing_tables.is_empty()
        {
            has_errors = true;
        }

        // Check table modifications
        for (table_name, table_diff) in &diff.modified_tables {
            let mut differences = Vec::new();

            if !table_diff.added_columns.is_empty() {
                differences.push(format!(
                    "Added columns: {:?}",
                    table_diff
                        .added_columns
                        .iter()
                        .map(|c| &c.name)
                        .collect::<Vec<_>>()
                ));
            }
            if !table_diff.removed_columns.is_empty() {
                differences.push(format!("Removed columns: {:?}", table_diff.removed_columns));
            }
            if !table_diff.modified_columns.is_empty() {
                differences.push(format!(
                    "Modified columns: {:?}",
                    table_diff.modified_columns.keys().collect::<Vec<_>>()
                ));
            }

            if !differences.is_empty() {
                has_errors = true;
                validation_error
                    .modified_tables
                    .insert(table_name.clone(), differences);
            }
        }

        // Check edge modifications
        for (edge_name, edge_diff) in &diff.modified_edges {
            let mut differences = Vec::new();

            if !differences.is_empty() {
                has_errors = true;
                validation_error
                    .modified_edges
                    .insert(edge_name.clone(), differences);
            }
        }

        if has_errors {
            bail!(validation_error);
        }

        // Run migration-specific validation
        migration.validate(&self.ctx).await?;

        Ok(())
    }

    /// Update migration status in the database
    async fn update_migration_status(
        &self,
        migration: &dyn Migration,
        status: MigrationStatus,
        error: Option<String>,
        execution_time: Option<i64>,
    ) -> Result<()> {
        let stmt = self.create_status_update_statement(migration, status, error, execution_time)?;

        self.ctx.db.query(&stmt).await?;
        Ok(())
    }

    /// Rollback a migration with proper validation
    async fn rollback_migration(&self, migration: &dyn Migration) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Get dry run information
        let dry_run = self.dry_run(migration).await?;

        // Start transaction
        let mut tx_statements = vec![
            "BEGIN TRANSACTION;".to_string(),
            self.create_status_update_statement(migration, MigrationStatus::Running, None, None)?,
        ];

        // Add rollback statements
        tx_statements.extend(dry_run.down_statements);

        // Execute rollback within transaction
        match self.execute_statements(&tx_statements).await {
            Ok(()) => {
                // Validate the rollback
                if let Err(e) = self.validate_rollback(migration).await {
                    // Rollback failed validation
                    self.ctx.db.query("CANCEL TRANSACTION;").await?;

                    // Record failure
                    self.update_migration_status(
                        migration,
                        MigrationStatus::Failed,
                        Some(format!("Rollback validation failed: {}", e)),
                        Some(start_time.elapsed().as_millis() as i64),
                    )
                    .await?;

                    return Err(e);
                }

                // Commit transaction
                self.ctx.db.query("COMMIT TRANSACTION;").await?;

                // Record success
                self.update_migration_status(
                    migration,
                    MigrationStatus::RolledBack,
                    None,
                    Some(start_time.elapsed().as_millis() as i64),
                )
                .await?;

                Ok(())
            }
            Err(e) => {
                // Rollback failed execution
                self.ctx.db.query("CANCEL TRANSACTION;").await?;

                // Record failure
                self.update_migration_status(
                    migration,
                    MigrationStatus::Failed,
                    Some(format!("Rollback execution failed: {}", e)),
                    Some(start_time.elapsed().as_millis() as i64),
                )
                .await?;

                Err(e)
            }
        }
    }

    /// Validate a rollback operation
    async fn validate_rollback(&self, migration: &dyn Migration) -> Result<()> {
        // Get the previous version's schema
        let previous_version = migration
            .get_previous_version()
            .map_err(anyhow::Error::from)?;
        let previous_snapshot = SchemaSnapshot::load_from_file(SchemaSnapshot::get_snapshot_path(
            ".",
            &previous_version,
        ))?;

        // Capture current schema after rollback
        let current = SchemaSnapshot::capture(&self.ctx).await?;

        // Compare current schema with previous version
        let diff = current.diff(&previous_snapshot);

        // Build detailed validation error if needed
        let mut validation_error = ValidationError {
            unexpected_tables: diff.added_tables.clone(),
            missing_tables: diff.removed_tables.clone(),
            modified_tables: HashMap::new(),
            modified_edges: HashMap::new(),
        };

        // Check for unexpected differences
        let mut has_errors = false;

        if !validation_error.unexpected_tables.is_empty()
            || !validation_error.missing_tables.is_empty()
        {
            has_errors = true;
        }

        // Check table modifications
        for (table_name, table_diff) in &diff.modified_tables {
            let mut differences = Vec::new();

            if !table_diff.added_columns.is_empty() {
                differences.push(format!(
                    "Unexpected columns present: {:?}",
                    table_diff
                        .added_columns
                        .iter()
                        .map(|c| &c.name)
                        .collect::<Vec<_>>()
                ));
            }
            if !table_diff.removed_columns.is_empty() {
                differences.push(format!("Missing columns: {:?}", table_diff.removed_columns));
            }
            if !table_diff.modified_columns.is_empty() {
                differences.push(format!(
                    "Columns not restored: {:?}",
                    table_diff.modified_columns.keys().collect::<Vec<_>>()
                ));
            }

            if !differences.is_empty() {
                has_errors = true;
                validation_error
                    .modified_tables
                    .insert(table_name.clone(), differences);
            }
        }

        if has_errors {
            bail!("Rollback validation failed: schema does not match previous version");
        }

        Ok(())
    }

    fn create_status_update_statement(
        &self,
        migration: &dyn Migration,
        status: MigrationStatus,
        error: Option<String>,
        execution_time: Option<i64>,
    ) -> Result<String> {
        let record = MigrationRecord {
            id: format!("{}_{}", migration.version(), migration.name()).into(),
            name: migration.name().to_string(),
            version: migration.version().to_string(),
            checksum: compute_checksum(migration),
            status,
            applied_at: Some(OffsetDateTime::now_utc()),
            execution_time_ms: execution_time,
            error_message: error,
            dependencies: migration.dependencies().to_vec(),
            schema_diff: SchemaDiff::default(),
        };

        Ok(format!(
            "UPDATE $_migrations MERGE {};",
            serde_json::to_string(&record)?
        ))
    }

    /// Execute multiple statements in order
    async fn execute_statements(&self, statements: &[String]) -> Result<()> {
        for stmt in statements {
            self.ctx.db.query(stmt).await?;
        }
        Ok(())
    }

    async fn get_applied_migrations(&self) -> Result<Vec<MigrationRecord>> {
        let records: Vec<MigrationRecord> = self
            .ctx
            .db
            .query("SELECT * FROM _migrations ORDER BY version")
            .await?
            .take(0)?;
        Ok(records)
    }

    fn get_pending_migrations(
        &self,
        applied: &[MigrationRecord],
    ) -> Result<Vec<Box<dyn Migration>>> {
        let applied_ids: HashSet<_> = applied.iter().map(|r| r.id.to_string()).collect();

        Ok(self
            .migrations
            .values()
            .filter(|m| !applied_ids.contains(m.id()))
            .cloned()
            .collect())
    }

    fn sort_by_dependencies(
        &self,
        migrations: &[Box<dyn Migration>],
    ) -> Result<Vec<Box<dyn Migration>>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();

        for migration in migrations {
            if !visited.contains(migration.id()) {
                self.visit_migration(migration.as_ref(), &mut sorted, &mut visited, &mut temp)?;
            }
        }

        Ok(sorted)
    }

    fn visit_migration(
        &self,
        migration: &dyn Migration,
        sorted: &mut Vec<Box<dyn Migration>>,
        visited: &mut HashSet<String>,
        temp: &mut HashSet<String>,
    ) -> Result<()> {
        if temp.contains(migration.id()) {
            return Err(anyhow!("Circular dependency detected in migrations"));
        }
        if visited.contains(migration.id()) {
            return Ok(());
        }

        temp.insert(migration.id().to_string());

        for dep_id in migration.dependencies() {
            if let Some(dep) = self.migrations.get(dep_id) {
                self.visit_migration(dep.as_ref(), sorted, visited, temp)?;
            } else {
                return Err(anyhow!("Missing dependency: {}", dep_id));
            }
        }

        temp.remove(migration.id());
        visited.insert(migration.id().to_string());

        if let Some(migration) = self.migrations.get(migration.id()) {
            sorted.push(migration.clone());
        }

        Ok(())
    }
}

fn compute_checksum(migration: &dyn Migration) -> String {
    let mut hasher = Sha256::new();
    hasher.update(migration.id().as_bytes());
    hasher.update(migration.name().as_bytes());
    hasher.update(migration.version().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// A migration generated from schema differences
#[derive(Debug)]
pub struct ValidationError {
    pub unexpected_tables: Vec<String>,
    pub missing_tables: Vec<String>,
    pub modified_tables: HashMap<String, Vec<String>>,
    pub modified_edges: HashMap<String, Vec<String>>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Schema validation failed:")?;
        if !self.unexpected_tables.is_empty() {
            writeln!(f, "Unexpected tables: {:?}", self.unexpected_tables)?;
        }
        if !self.missing_tables.is_empty() {
            writeln!(f, "Missing tables: {:?}", self.missing_tables)?;
        }
        for (table, diffs) in &self.modified_tables {
            writeln!(f, "Table {} has unexpected changes:", table)?;
            for diff in diffs {
                writeln!(f, "  {}", diff)?;
            }
        }
        for (edge, diffs) in &self.modified_edges {
            writeln!(f, "Edge {} has unexpected changes:", edge)?;
            for diff in diffs {
                writeln!(f, "  {}", diff)?;
            }
        }
        Ok(())
    }
}