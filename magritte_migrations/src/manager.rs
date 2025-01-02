use super::{introspection, snapshot, Diff, Error, Result, ensure_overwrite};
use crate::edge::EdgeDiff;
use crate::snapshot::save_to_file;
use crate::table::TableDiff;
use crate::types::FlexibleDateTime;
use magritte::{EdgeRegistration, EventRegistration, IndexRegistration, Query, SchemaSnapshot, Snapshot, SurrealDB, TableRegistration};
use std::path::PathBuf;
use tracing::debug;

/// Manager for handling database schema migrations.
/// 
/// The `MigrationManager` is responsible for:
/// - Creating new migrations from current schema
/// - Applying migrations to the database
/// - Rolling back to previous schema versions
/// - Validating schema changes
/// 
/// # Example
/// 
/// ```rust,ignore
/// use magritte_migrations::MigrationManager;
/// use std::path::PathBuf;
/// 
/// let manager = MigrationManager::new(PathBuf::from("./migrations"));
/// ```
pub struct MigrationManager {
    /// Directory where migration files are stored
    pub migrations_dir: PathBuf,
}

impl MigrationManager {
    /// Creates a new migration manager with the specified migrations directory.
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self { migrations_dir }
    }

    /// Gets the current schema from registered tables and edges.
    /// 
    /// This method collects schema information from all registered tables and edges,
    /// including their fields, indexes, and events.
    pub fn current_schema(&self) -> Result<SchemaSnapshot> {
        let mut schema = SchemaSnapshot::new();

        // Process tables
        for reg in inventory::iter::<TableRegistration> {
            // First get basic table snapshot
            let mut table_snap = (reg.builder)().map_err(Error::from)?;
            for event_reg in inventory::iter::<EventRegistration> {
                if event_reg.type_id == reg.type_id {
                    for event_def in (event_reg.builder)() {
                        table_snap.add_event(event_def.event_name().into(), event_def.to_statement()?.build().map_err(anyhow::Error::from)?);
                    }
                }
            }

            // Find any registered indexes for this table
            for index_reg in inventory::iter::<IndexRegistration> {
                if index_reg.type_id == reg.type_id {
                    for index_def in (index_reg.builder)() {
                        table_snap.add_index(index_def.index_name().into(), index_def.to_statement().to_string());
                    }
                }
            }
            schema.add_table(table_snap);
        }

        // Process edges similarly
        for reg in inventory::iter::<EdgeRegistration> {
            let mut edge_snap = (reg.builder)().map_err(Error::from)?;
            for event_reg in inventory::iter::<EventRegistration> {
                if event_reg.type_id == reg.type_id {
                    for event_def in (event_reg.builder)() {
                        edge_snap.add_event(event_def.event_name().into(), event_def.to_statement()?.build().map_err(anyhow::Error::from)?);
                    }
                }
            }

            // Find any registered indexes for this table
            for index_reg in inventory::iter::<IndexRegistration> {
                if index_reg.type_id == reg.type_id {
                    for index_def in (index_reg.builder)() {
                        edge_snap.add_index(index_def.index_name().into(), index_def.to_statement().to_string());
                    }
                }
            }
            schema.add_edge(edge_snap);
        }

        Ok(schema)
    }

    /// Creates a new migration from the current schema.
    /// 
    /// This method generates a new migration file with the current schema state.
    /// The migration name is automatically generated based on the timestamp.
    pub fn new_migration(&self, snapshot: &SchemaSnapshot) -> Result<String> {
        // Determine the next migration number based on existing files
        let migration_name = self.get_name()?;

        let json_path = self
            .migrations_dir
            .join(format!("{}_schema.json", &migration_name));
        save_to_file(snapshot, &json_path)?;

        Ok(migration_name)
    }

    /// Creates a snapshot of the current schema state.
    /// 
    /// If a database connection is provided, this will also include the current
    /// database schema state in the snapshot. Otherwise, it will only include
    /// the registered schema.
    /// 
    /// # Arguments
    /// 
    /// * `db` - Optional database connection to include current DB state
    /// * `name` - Optional name for the snapshot file
    pub async fn create_snapshot(
        &self,
        db: Option<SurrealDB>,
        name: Option<String>,
    ) -> Result<(PathBuf, Vec<String>)> {
        // 1. Create base schema from registered entities
        let current_schema = self.current_schema()?;
        // 2. If migrations exist, diff against latest
        let intermediary_schema = if let Some((_, last_snap)) = self.latest()? {
            let diff_statements = self.diff(&last_snap, &current_schema)?;
            if !diff_statements.is_empty() {
                current_schema.clone()
            } else {
                last_snap
            }
        } else {
            current_schema.clone()
        };

        // 3. If DB exists, diff against DB schema and include relations
        let (final_schema, statements) = if let Some(db) = db {
            let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;
            let mut diff_statements = self.diff(&db_snapshot, &intermediary_schema)?;
            
            (intermediary_schema, diff_statements)
        } else {
            (intermediary_schema.clone(), self.diff(&SchemaSnapshot::new(), &intermediary_schema)?)
        };

        // Save the final snapshot
        let path = if let Some(name) = name {
            self.migrations_dir.join(format!("{}.json", name))
        } else {
            self.migrations_dir.join(format!("{}_schema.json", self.get_name()?))
        };
        save_to_file(&final_schema, &path)?;

        Ok((path, statements))
    }

    async fn generate_snapshot_diff(
        &self,
        db: Option<SurrealDB>,
        target_snapshot: &SchemaSnapshot,
    ) -> Result<(SchemaSnapshot, Vec<String>)> {
        match db {
            Some(db) => {
                // Get current DB state
                let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;

                // Validate and generate diff
                let validated = self.validate_db(
                    db,
                    &db_snapshot,
                    target_snapshot
                ).await?;

                let statements = self.diff(&db_snapshot, &validated)?;
                Ok((validated, statements))
            }
            None => {
                // Just diff against latest snapshot if exists
                let latest = self.latest()?;
                match latest {
                    Some((_, last)) => {
                        let statements = self.diff(&last, target_snapshot)?;
                        Ok((target_snapshot.clone(), statements))
                    }
                    None => Ok((target_snapshot.clone(), vec![]))
                }
            }
        }
    }

    //noinspection RsTraitObligations
    pub fn diff(
        &self,
        old_snapshot: &SchemaSnapshot,
        new_snapshot: &SchemaSnapshot,
    ) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        // Compare tables
        for (table_name, new_table) in &new_snapshot.tables {
            if let Some(old_table) = old_snapshot.tables.get(table_name) {
                // Build TableDiff
                let diff = TableDiff::from_snapshots(old_table, new_table)?;

                let up_stmts = diff.generate_statements(table_name)?;
                statements.extend(up_stmts);
            } else {
                // Table didn't exist before
                let mut diff = TableDiff::new(None, Some(new_table.define_table_statement.clone()));
                // Populate diff fields etc. if needed, or if fields are already in the `from_snapshots` logic, no need.
                // Actually, since we don't have an old snapshot, we just treat all fields/indexes/events as "added".
                for (f, v) in &new_table.fields {
                    diff.added_columns.insert(f.clone(), v.clone());
                }
                for (i, v) in &new_table.indexes {
                    diff.added_indexes.insert(i.clone(), v.clone());
                }
                for (e, v) in &new_table.events {
                    diff.added_events.insert(e.clone(), v.clone());
                }

                let up_stmts = diff.generate_statements(table_name)?;
                statements.extend(up_stmts);
            }
        }

        // Handle removed tables
        for (table_name, old_table) in &old_snapshot.tables {
            if !new_snapshot.tables.contains_key(table_name) {
                for field in old_table.fields.keys() {
                    statements.push(format!("REMOVE FIELD {} ON TABLE {};", field, table_name));
                }
                for idx in old_table.indexes.keys() {
                    statements.push(format!("REMOVE INDEX {} ON TABLE {};", idx, table_name));
                }
                for evt in old_table.events.keys() {
                    statements.push(format!("REMOVE EVENT {} ON TABLE {};", evt, table_name));
                }
                statements.push(format!("REMOVE TABLE {};", table_name));
            }
        }

        for (edge_name, new_edge) in &new_snapshot.edges {
            if let Some(old_edge) = old_snapshot.edges.get(edge_name) {
                // Build TableDiff
                let diff = EdgeDiff::from_snapshots(old_edge, new_edge)?;
                let up_stmts = diff.generate_statements(edge_name)?;
                statements.extend(up_stmts);
            } else {
                // Table didn't exist before
                let mut diff = EdgeDiff::new(None, Some(new_edge.define_edge_statement.clone()));
                // Populate diff fields etc. if needed, or if fields are already in the `from_snapshots` logic, no need.
                // Actually, since we don't have an old snapshot, we just treat all fields/indexes/events as "added".
                for (f, v) in &new_edge.fields {
                    diff.added_columns.insert(f.clone(), v.clone());
                }
                for (i, v) in &new_edge.indexes {
                    diff.added_indexes.insert(i.clone(), v.clone());
                }
                for (e, v) in &new_edge.events {
                    diff.added_events.insert(e.clone(), v.clone());
                }

                let up_stmts = diff.generate_statements(edge_name)?;
                statements.extend(up_stmts);
            }
        }

        // Handle removed edges
        for (edge_name, old_edge) in &old_snapshot.edges {
            if !new_snapshot.edges.contains_key(edge_name) {
                for field in old_edge.fields.keys() {
                    statements.push(format!("REMOVE FIELD {} ON TABLE {};", field, edge_name));
                }
                for idx in old_edge.indexes.keys() {
                    statements.push(format!("REMOVE INDEX {} ON TABLE {};", idx, edge_name));
                }
                for evt in old_edge.events.keys() {
                    statements.push(format!("REMOVE EVENT {} ON TABLE {};", evt, edge_name));
                }
                statements.push(format!("REMOVE TABLE {};", edge_name));
            }
        }

        Ok(statements)
    }

    fn get_name(&self) -> Result<String> {
        let next_number = self.next_migration_number()?;
        let timestamp = Self::get_current_timestamp();
        let migration_name = format!("{:04}_{}", next_number, timestamp);

        Ok(migration_name)
    }


    fn next_migration_number(&self) -> Result<usize> {
        let entries = std::fs::read_dir(&self.migrations_dir)?;
        let max_num = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|f| f.to_string_lossy().into_owned())
            })
            .filter_map(|filename| {
                filename
                    .split('_')
                    .next()
                    .and_then(|num_str| num_str.parse::<usize>().ok())
            })
            .max()
            .unwrap_or(0);
        Ok(max_num + 1)
    }

    fn get_current_timestamp() -> String {
        FlexibleDateTime::now().to_string()
    }

    async fn validate_db(
        &self,
        db: SurrealDB,
        stored_snapshot: &SchemaSnapshot,
        code_snapshot: &SchemaSnapshot,
    ) -> Result<SchemaSnapshot> {
        // 1. Get current DB state
        let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;

        // 2. Compare DB state with stored snapshot to detect drift
        let validation = introspection::validate_migration(db.clone(), stored_snapshot).await?;
        if validation.has_issues() {
            tracing::warn!(
                "Database state has drifted from stored snapshot: {:?}",
                validation
            );
        }

        let mut schema = SchemaSnapshot::new();
        for (table_name, new_table) in &code_snapshot.tables {
            let diff = if let Some(db_table) = db_snapshot.tables.get(table_name) {
                TableDiff::from_snapshots(db_table, new_table)?
            } else {
                let mut diff = TableDiff::new(None, Some(new_table.define_table_statement.clone()));
                diff.name = table_name.clone();
                diff.added_columns = new_table.fields.clone();
                diff.added_indexes = new_table.indexes.clone();
                diff.added_events = new_table.events.clone();
                diff
            };
            schema.add_table(diff.to_snapshot()?);
        }

        for (edge_name, new_edge) in &code_snapshot.edges {
            let diff = if let Some(db_edge) = db_snapshot.edges.get(edge_name) {
                EdgeDiff::from_snapshots(db_edge, new_edge)?
            } else {
                let mut diff = EdgeDiff::new(None, Some(new_edge.define_edge_statement.clone()));
                diff.name = edge_name.clone();
                diff.added_columns = new_edge.fields.clone();
                diff.added_indexes = new_edge.indexes.clone();
                diff.added_events = new_edge.events.clone();
                diff
            };
            schema.add_edge(diff.to_snapshot()?);
        }

        Ok(schema)
    }

    fn latest(&self) -> Result<Option<(PathBuf, SchemaSnapshot)>> {
        let latest = std::fs::read_dir(&self.migrations_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .max_by_key(|e| e.metadata().unwrap().modified().unwrap());

        match latest {
            Some(entry) => {
                let path = entry.path();
                let snapshot = snapshot::load_from_file(&path)?;
                Ok(Some((path, snapshot)))
            }
            None => Ok(None),
        }
    }

    pub async fn check_deviations(
        &self,
        db: &SurrealDB,
        snapshot_path: &PathBuf,
    ) -> Result<DeviationReport> {
        let target_snapshot = snapshot::load_from_file(snapshot_path)?;
        let current_schema = self.current_schema()?;
        let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;

        let schema_diff = self.diff(&target_snapshot, &current_schema)?;
        let db_diff = self.diff(&db_snapshot, &target_snapshot)?;

        Ok(DeviationReport {
            db_deviations: if !db_diff.is_empty() { Some(db_diff) } else { None },
            schema_deviations: if !schema_diff.is_empty() { Some(schema_diff) } else { None },
            action: MigrationAction::Skip,
        })
    }

    pub async fn apply_migration(
        &self,
        db: &SurrealDB,
        snapshot_name: Option<String>,
    ) -> Result<()> {
        let snapshot_path = match snapshot_name {
            Some(name) => self.migrations_dir.join(format!("{}_schema.json", name)),
            None => {
                // Create new snapshot if none specified
                let (path, _) = self.create_snapshot(Some(db.clone()), None).await?;
                path
            }
        };

        // Check for deviations
        let report = self.check_deviations(db, &snapshot_path).await?;
        
        if report.schema_deviations.is_some() || report.db_deviations.is_some() {
            tracing::warn!("Found deviations in schema or database state");
            if let Some(schema_diff) = &report.schema_deviations {
                tracing::warn!("Schema deviations:\n{}", schema_diff.join("\n"));
            }
            if let Some(db_diff) = &report.db_deviations {
                tracing::warn!("Database deviations:\n{}", db_diff.join("\n"));
            }
            // Note: In CLI this would prompt for user input
            // For library usage, we'll need to expose this information
            return Ok(());
        }

        // Load and apply the snapshot
        let snapshot = snapshot::load_from_file(&snapshot_path)?;
        let (_validated_snapshot, statements) = self.generate_snapshot_diff(Some(db.clone()), &snapshot).await?;

        if !statements.is_empty() {
            let mut transaction = Query::begin();
            for stmt in statements {
                transaction = transaction.raw(&stmt);
            }
            transaction.commit().execute(db).await.map_err(Error::from)?;
        }

        Ok(())
    }

    /// Rolls back the schema to a previous version.
    /// 
    /// This method reverts the schema to a specified snapshot version. If no
    /// specific version is provided, it rolls back to the previous version.
    /// 
    /// # Arguments
    /// 
    /// * `db` - Database connection to apply the rollback
    /// * `snapshot_name` - Optional specific snapshot to roll back to
    pub async fn rollback(
        &self,
        db: &SurrealDB,
        snapshot_name: Option<String>,
    ) -> Result<()> {
        // Get latest migration if none specified
        let target_path = match snapshot_name {
            Some(name) => self.migrations_dir.join(format!("{}_schema.json", name)),
            None => {
                let (path, _) = self.latest()?.ok_or_else(|| {
                    Error::from(anyhow::anyhow!("No migrations found to rollback to"))
                })?;
                path
            }
        };

        // Load snapshots
        let target_snapshot = snapshot::load_from_file(&target_path)?;
        let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;
        
        let mut statements = Vec::new();
        
        // First, handle tables that exist in DB but not in target
        for (table_name, _) in &db_snapshot.tables {
            if !target_snapshot.tables.contains_key(table_name) {
                statements.push(format!("REMOVE TABLE {};", table_name));
            }
        }

        // Then handle tables that exist in both or only in target
        for (table_name, target_table) in &target_snapshot.tables {
            if let Some(db_table) = db_snapshot.tables.get(table_name) {
                // Table exists in both - first remove fields that don't exist in target
                for field in db_table.fields.keys() {
                    if !target_table.fields.contains_key(field) {
                        statements.push(format!("REMOVE FIELD {} ON TABLE {};", field, table_name));
                    }
                }
                
                // Remove indexes that don't exist in target
                for index in db_table.indexes.keys() {
                    if !target_table.indexes.contains_key(index) {
                        statements.push(format!("REMOVE INDEX {} ON TABLE {};", index, table_name));
                    }
                }
                
                // Remove events that don't exist in target
                for event in db_table.events.keys() {
                    if !target_table.events.contains_key(event) {
                        statements.push(format!("REMOVE EVENT {} ON TABLE {};", event, table_name));
                    }
                }
            }
            
            // Now define/redefine the table
            statements.push(ensure_overwrite(&target_table.define_table_statement));
            
            // Add/update fields in target schema
            for (field_name, stmt) in &target_table.fields {
                if let Some(db_table) = db_snapshot.tables.get(table_name) {
                    if let Some(db_field) = db_table.fields.get(field_name) {
                        if db_field != stmt {
                            statements.push(ensure_overwrite(stmt));
                        }
                    } else {
                        statements.push(ensure_overwrite(stmt));
                    }
                } else {
                    statements.push(ensure_overwrite(stmt));
                }
            }
            
            // Add/update indexes in target schema
            for (index_name, stmt) in &target_table.indexes {
                if let Some(db_table) = db_snapshot.tables.get(table_name) {
                    if let Some(db_index) = db_table.indexes.get(index_name) {
                        if db_index != stmt {
                            statements.push(ensure_overwrite(stmt));
                        }
                    } else {
                        statements.push(ensure_overwrite(stmt));
                    }
                } else {
                    statements.push(ensure_overwrite(stmt));
                }
            }
            
            // Add/update events in target schema
            for (event_name, stmt) in &target_table.events {
                if let Some(db_table) = db_snapshot.tables.get(table_name) {
                    if let Some(db_event) = db_table.events.get(event_name) {
                        if db_event != stmt {
                            statements.push(ensure_overwrite(stmt));
                        }
                    } else {
                        statements.push(ensure_overwrite(stmt));
                    }
                } else {
                    statements.push(ensure_overwrite(stmt));
                }
            }
        }

        if !statements.is_empty() {
            let mut transaction = Query::begin();
            for stmt in statements {
                transaction = transaction.raw(&stmt);
            }
            transaction.commit().execute(db).await.map_err(Error::from)?;
        }

        Ok(())
    }
}

/// Represents possible actions to take when applying migrations.
#[derive(Debug)]
pub enum MigrationAction {
    /// Apply the migration normally
    Apply,
    /// Override existing schema with the migration
    Override,
    /// Skip this migration
    Skip,
}

/// Report of schema deviations found during migration.
#[derive(Debug)]
pub struct DeviationReport {
    /// Deviations found in the database schema
    pub db_deviations: Option<Vec<String>>,
    /// Deviations found in the migration schema
    pub schema_deviations: Option<Vec<String>>,
    /// Action to take based on the deviations
    pub action: MigrationAction,
}
