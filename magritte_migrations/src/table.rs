#![allow(unused)]
use crate::ensure_overwrite;
use magritte::TableSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::event;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableDiff {
    pub name: String,
    pub previous: Option<String>,
    pub current: String,
    pub added_columns: HashMap<String, String>,
    pub removed_columns: HashMap<String, String>,
    pub modified_columns: HashMap<String, (String, String)>,
    pub added_indexes: HashMap<String, String>,
    pub removed_indexes: HashMap<String, String>,
    pub modified_indexes: HashMap<String, (String, String)>,
    pub added_events: HashMap<String, String>,
    pub removed_events: HashMap<String, String>,
    pub modified_events: HashMap<String, (String, String)>,
}

impl TableDiff {
    pub fn new(previous: Option<String>, current: Option<String>) -> Self {
        match (current, previous) {
            (Some(current), Some(previous)) => Self {
                previous: Some(previous),
                current,
                ..Self::default()
            },
            (Some(current), None) => Self {
                current,
                ..Self::default()
            },
            (None, Some(_)) => panic!("Current table is not defined"),
            (None, None) => panic!("Both tables are not defined"),
        }
    }

    pub fn generate_statements(&self, table_name: &str) -> anyhow::Result<Vec<String>> {
        let mut statements = Vec::new();

        // Only include table definition if this is a new table (no previous definition)
        if self.previous.is_none() {
            statements.push(ensure_overwrite(&self.current));
        }

        // Add new columns
        statements.extend(
            self.added_columns
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        // Modify existing columns
        statements.extend(
            self.modified_columns
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(new_value)),
        );

        // Add new indexes
        statements.extend(
            self.added_indexes
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        // Modify existing indexes
        statements.extend(
            self.modified_indexes
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(new_value)),
        );

        // Add new events
        statements.extend(
            self.added_events
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        // Modify existing events
        statements.extend(
            self.modified_events
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(new_value)),
        );

        // Remove columns that no longer exist
        for column in self.removed_columns.keys() {
            statements.push(format!("REMOVE FIELD {} ON TABLE {};", column, table_name));
        }

        // Remove indexes that no longer exist
        for index in self.removed_indexes.keys() {
            statements.push(format!("REMOVE INDEX {} ON TABLE {};", index, table_name));
        }

        // Remove events that no longer exist
        for event in self.removed_events.keys() {
            statements.push(format!("REMOVE EVENT {} ON TABLE {};", event, table_name));
        }

        Ok(statements)
    }

    pub fn reverse(&self, table_name: &str) -> anyhow::Result<Vec<String>> {
        let mut statements = Vec::new();

        // Table definition (ensure OVERWRITE is present)
        statements.push(ensure_overwrite(self.previous.as_ref().unwrap()));

        for column in self.added_columns.keys() {
            statements.push(format!("REMOVE FIELD {} ON TABLE {};", column, table_name));
        }

        for index in self.added_indexes.keys() {
            statements.push(format!("REMOVE INDEX {} ON TABLE {};", index, table_name));
        }

        for event in self.added_events.keys() {
            statements.push(format!("REMOVE EVENT {} ON TABLE {};", event, table_name));
        }

        // Modify existing columns
        statements.extend(
            self.modified_columns
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(old_value)),
        );

        let added_columns = self.removed_columns.clone();
        statements.extend(
            added_columns
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        // Modify existing indexes
        statements.extend(
            self.modified_indexes
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(old_value)),
        );

        let added_indexes = self.removed_indexes.clone();
        statements.extend(
            added_indexes
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        // Modify existing events
        statements.extend(
            self.modified_events
                .iter()
                .map(|(key, (old_value, new_value))| ensure_overwrite(old_value)),
        );

        let added_events = self.removed_events.clone();
        statements.extend(
            added_events
                .iter()
                .map(|(key, value)| ensure_overwrite(value)),
        );

        Ok(statements)
    }
}
impl TableDiff {
    pub fn from_snapshots(
        old_table: &TableSnapshot,
        new_table: &TableSnapshot,
    ) -> anyhow::Result<Self> {
        let mut diff = TableDiff::new(
            Some(old_table.define_table_statement.clone()),
            Some(new_table.define_table_statement.clone()),
        );
        diff.name = new_table.name.clone();

        // Compare fields
        for (field_name, new_def) in &new_table.fields {
            if let Some(old_def) = old_table.fields.get(field_name) {
                // Field exists in both
                if old_def.trim() != new_def.trim() {
                    diff.modified_columns
                        .insert(field_name.clone(), (old_def.clone(), new_def.clone()));
                }
            } else {
                // Field only in new
                diff.added_columns
                    .insert(field_name.clone(), new_def.clone());
            }
        }
        // Fields removed from new
        for (field_name, old_def) in &old_table.fields {
            if !new_table.fields.contains_key(field_name) {
                diff.removed_columns
                    .insert(field_name.clone(), old_def.clone());
            }
        }

        // Compare indexes
        for (idx_name, new_def) in &new_table.indexes {
            if let Some(old_def) = old_table.indexes.get(idx_name) {
                if old_def.trim() != new_def.trim() {
                    diff.modified_indexes
                        .insert(idx_name.clone(), (old_def.clone(), new_def.clone()));
                }
            } else {
                diff.added_indexes.insert(idx_name.clone(), new_def.clone());
            }
        }

        for (idx_name, old_def) in &old_table.indexes {
            if !new_table.indexes.contains_key(idx_name) {
                diff.removed_indexes
                    .insert(idx_name.clone(), old_def.clone());
            }
        }

        // Compare events
        for (evt_name, new_def) in &new_table.events {
            if let Some(old_def) = old_table.events.get(evt_name) {
                if old_def.trim() != new_def.trim() {
                    diff.modified_events
                        .insert(evt_name.clone(), (old_def.clone(), new_def.clone()));
                }
            } else {
                diff.added_events.insert(evt_name.clone(), new_def.clone());
            }
        }

        for (evt_name, old_def) in &old_table.events {
            if !new_table.events.contains_key(evt_name) {
                diff.removed_events
                    .insert(evt_name.clone(), old_def.clone());
            }
        }

        Ok(diff)
    }
    pub fn to_snapshot(&self) -> anyhow::Result<TableSnapshot> {
        let mut snapshot = TableSnapshot::new(self.name.clone(), self.current.clone());

        // Add all fields that weren't removed and were either added or modified
        for (field_name, field_def) in &self.added_columns {
            snapshot.add_field(field_name.clone(), field_def.clone());
        }
        for (field_name, (_, new_def)) in &self.modified_columns {
            snapshot.add_field(field_name.clone(), new_def.clone());
        }

        // Same for indexes
        for (index_name, index_def) in &self.added_indexes {
            snapshot.add_index(index_name.clone(), index_def.clone());
        }
        for (index_name, (_, new_def)) in &self.modified_indexes {
            snapshot.add_index(index_name.clone(), new_def.clone());
        }

        // And events
        for (event_name, event_def) in &self.added_events {
            snapshot.add_event(event_name.clone(), event_def.clone());
        }
        for (event_name, (_, new_def)) in &self.modified_events {
            snapshot.add_event(event_name.clone(), new_def.clone());
        }

        Ok(snapshot)
    }
}
