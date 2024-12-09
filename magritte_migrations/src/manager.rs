use super::Result;
use crate::types::FlexibleDateTime;
use magritte::SchemaSnapshot;
use std::path::PathBuf;
use crate::snapshot::save_to_file;

pub struct MigrationManager {
    pub migrations_dir: PathBuf,
}

impl MigrationManager {
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self { migrations_dir }
    }

    pub fn create_new_migration(&self, snapshot: &SchemaSnapshot) -> Result<String> {
        // Determine the next migration number based on existing files
        let next_number = self.next_migration_number()?;
        let timestamp = Self::get_current_timestamp();
        let migration_name = format!("{:04}_{}", next_number, timestamp);

        let json_path = self
            .migrations_dir
            .join(format!("{}_schema.json", &migration_name));
        save_to_file(&snapshot, &json_path)?;

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

    fn generate_surql(&self, snapshot: &SchemaSnapshot) -> Result<String> {
        let mut statements = Vec::new();
        for table in snapshot.tables.values() {
            statements.push(table.define_table_statement.clone());
            for field_stmt in table.fields.values() {
                statements.push(field_stmt.clone());
            }
            for idx_stmt in table.indexes.values() {
                statements.push(idx_stmt.clone());
            }
            for evt_stmt in table.events.values() {
                statements.push(evt_stmt.clone());
            }
        }

        for edge in snapshot.edges.values() {
            statements.push(edge.define_edge_statement.clone());
        }

        Ok(statements.join("\n"))
    }

    fn get_current_timestamp() -> String {
        FlexibleDateTime::now().to_string()
    }
}
