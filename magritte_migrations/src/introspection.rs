//! Database schema introspection functionality.
//!
//! This module provides tools for inspecting and capturing the current state
//! of a SurrealDB database schema, including tables, edges, fields, indexes,
//! and events.

use crate::error::Result;
use magritte::{DbInfo, Query, SchemaSnapshot, Snapshot, SurrealDB, TableInfo, TableSnapshot};

/// Report of validation issues found when comparing schemas.
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// Schema elements that don't match expected definitions
    pub mismatches: Vec<String>,
    /// Schema elements that are missing from the current schema
    pub missing: Vec<String>,
    /// Schema elements that exist but weren't expected
    pub unexpected: Vec<String>,
}

impl ValidationReport {
    /// Creates a new empty validation report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if any validation issues were found.
    pub fn has_issues(&self) -> bool {
        !self.mismatches.is_empty() || !self.missing.is_empty() || !self.unexpected.is_empty()
    }
}

/// Gets database information including all tables and their definitions.
pub async fn get_db_info(db: SurrealDB) -> Result<DbInfo> {
    Ok(Query::info(db)
        .info_db()
        .await
        .map_err(anyhow::Error::from)?)
}

/// Gets detailed information about a specific table.
pub async fn get_table_info(db: SurrealDB, table: &str) -> Result<TableInfo> {
    Ok(Query::info(db)
        .info_table(table)
        .await
        .map_err(anyhow::Error::from)?)
}

/// Creates a snapshot of the current database schema state.
///
/// This function queries the database to build a complete picture of the
/// current schema, including:
/// - Table definitions
/// - Field definitions
/// - Index definitions
/// - Event definitions
/// - Edge table definitions
pub async fn create_snapshot_from_db(db: SurrealDB) -> Result<SchemaSnapshot> {
    let mut snapshot = SchemaSnapshot::new();
    let db_info = get_db_info(db.clone()).await?;

    // Process tables
    for (table_name, table_def) in db_info.tables {
        let table_info = get_table_info(db.clone(), &table_name).await?;
        let mut table_snapshot = TableSnapshot::new(table_name, table_def);

        // Add fields
        for (field_name, field_def) in table_info.fields {
            table_snapshot.add_field(field_name, field_def)
        }

        // Add indexes
        for (index_name, index_def) in table_info.indexes {
            table_snapshot.add_index(index_name, index_def)
        }

        // Add events
        for (event_name, event_def) in table_info.events {
            table_snapshot.add_event(event_name, event_def)
        }

        snapshot.add_table(table_snapshot);
    }

    Ok(snapshot)
}

/// Validates a migration by comparing expected schema against current DB state.
///
/// This function performs a detailed comparison between the expected schema
/// (from the migration) and the actual schema in the database. It reports:
/// - Mismatched definitions
/// - Missing schema elements
/// - Unexpected schema elements
///
/// # Arguments
///
/// * `db` - Database connection to check current state
/// * `expected` - Expected schema state from the migration
pub async fn validate_migration(
    db: SurrealDB,
    expected: &SchemaSnapshot,
) -> Result<ValidationReport> {
    let current = create_snapshot_from_db(db).await?;
    let mut report = ValidationReport::new();

    // Check tables
    for (table_name, expected_table) in &expected.tables {
        match current.tables.get(table_name) {
            Some(current_table) => {
                // Compare table definitions
                if current_table.define_table_statement != expected_table.define_table_statement {
                    report.mismatches.push(format!(
                        "Table '{}' definition mismatch:\nExpected: {}\nActual: {}",
                        table_name,
                        expected_table.define_table_statement,
                        current_table.define_table_statement
                    ));
                }

                // Compare fields
                for (field_name, expected_field) in &expected_table.fields {
                    match current_table.fields.get(field_name) {
                        Some(current_field) if current_field != expected_field => {
                            report.mismatches.push(format!(
                                "Field '{}' in table '{}' mismatch:\nExpected: {}\nActual: {}",
                                field_name, table_name, expected_field, current_field
                            ));
                        }
                        None => {
                            report.missing.push(format!(
                                "Field '{}' missing in table '{}'",
                                field_name, table_name
                            ));
                        }
                        _ => {}
                    }
                }

                for (index_name, expected_index) in &expected_table.indexes {
                    match current_table.indexes.get(index_name) {
                        Some(current_index) if current_index != expected_index => {
                            report.mismatches.push(format!(
                                "Index '{}' in table '{}' mismatch:\nExpected: {}\nActual: {}",
                                index_name, table_name, expected_index, current_index
                            ));
                        }
                        None => {
                            report.missing.push(format!(
                                "Index '{}' missing in table '{}'",
                                index_name, table_name
                            ));
                        }
                        _ => {}
                    }
                }

                for (event_name, expected_event) in &expected_table.events {
                    match current_table.events.get(event_name) {
                        Some(current_event) if current_event != expected_event => {
                            report.mismatches.push(format!(
                                "Event '{}' in table '{}' mismatch:\nExpected: {}\nActual: {}",
                                event_name, table_name, expected_event, current_event
                            ));
                        }
                        None => {
                            report.missing.push(format!(
                                "Event '{}' missing in table '{}'",
                                event_name, table_name
                            ));
                        }
                        _ => {}
                    }
                }

                // Check for unexpected fields
                for field_name in current_table.fields.keys() {
                    if !expected_table.fields.contains_key(field_name) {
                        report.unexpected.push(format!(
                            "Unexpected field '{}' in table '{}'",
                            field_name, table_name
                        ));
                    }
                }

                for index_name in current_table.indexes.keys() {
                    if !expected_table.indexes.contains_key(index_name) {
                        report.unexpected.push(format!(
                            "Unexpected index '{}' in table '{}'",
                            index_name, table_name
                        ));
                    }
                }

                for event_name in current_table.events.keys() {
                    if !expected_table.events.contains_key(event_name) {
                        report.unexpected.push(format!(
                            "Unexpected event '{}' in table '{}'",
                            event_name, table_name
                        ));
                    }
                }
            }
            None => {
                report
                    .missing
                    .push(format!("Table '{}' is missing", table_name));
            }
        }
    }

    // Check for unexpected tables
    for table_name in current.tables.keys() {
        if !expected.tables.contains_key(table_name) {
            report
                .unexpected
                .push(format!("Unexpected table '{}'", table_name));
        }
    }

    Ok(report)
}
