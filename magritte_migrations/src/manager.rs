use super::{introspection, snapshot, Error, Result};
use crate::edge::EdgeDiff;
use crate::snapshot::save_to_file;
use crate::table::TableDiff;
use crate::types::FlexibleDateTime;
use magritte::{
    EdgeRegistration, EventRegistration, IndexRegistration, Query, SchemaSnapshot, SurrealDB,
    TableRegistration,
};
use std::path::PathBuf;
use tracing::debug;

pub struct MigrationManager {
    pub migrations_dir: PathBuf,
}

impl MigrationManager {
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self { migrations_dir }
    }

    pub fn current_schema_from_code(&self) -> Result<SchemaSnapshot> {
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

    pub fn get_file_stem(path: &str) -> &str {
        std::path::Path::new(path)
            .file_stem()
            .map(|f| f.to_str().unwrap())
            .unwrap()
    }

    //noinspection RsTraitObligations
    pub fn generate_diff_migration(
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

    pub fn create_empty_migration(&self) -> Result<String> {
        let timestamp = Self::get_current_timestamp();
        let migration_name = format!("{:04}_{}", 0, timestamp);
        let json_path = self
            .migrations_dir
            .join(format!("{}_schema.json", &migration_name));
        save_to_file(&SchemaSnapshot::new(), &json_path)?;

        Ok(migration_name)
    }

    pub fn create_new_migration(&self, snapshot: &SchemaSnapshot) -> Result<String> {
        // Determine the next migration number based on existing files
        let next_number = self.next_migration_number()?;
        let timestamp = Self::get_current_timestamp();
        let migration_name = format!("{:04}_{}", next_number, timestamp);

        let json_path = self
            .migrations_dir
            .join(format!("{}_schema.json", &migration_name));
        save_to_file(snapshot, &json_path)?;

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

    pub async fn generate_migration_with_db_check(
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

    pub async fn apply_migration(&self, db: &SurrealDB, migration_name: &str) -> Result<()> {
        // Load the diff
        let diff_path = self.migrations_dir.join(format!("{}_schema.json", migration_name));
        let stored_snapshot = snapshot::load_from_file(&diff_path)?;

        let code_snapshot = self.current_schema_from_code()?;
        let validated_snapshot =
            self.generate_migration_with_db_check(db.clone(), &stored_snapshot, &code_snapshot)
                .await?;

        let statements = self.generate_diff_migration(&stored_snapshot, &validated_snapshot)?;

        let mut transaction = Query::begin();
        for stmt in statements {
            transaction = transaction.raw(&stmt);
        }
        debug!("Final trn: {}", transaction.clone().commit().build());
        transaction
            .commit()
            .execute(db)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(())
    }
}
