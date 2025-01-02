#![allow(unused)]
use super::{Diff, Result};
use crate::ensure_overwrite;
use magritte::{EdgeSnapshot, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::event;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeDiff {
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

impl EdgeDiff {
    pub fn new(previous: Option<String>, current: Option<String>) -> Self {
        match (current, previous) {
            (Some(current), Some(previous)) => Self {
                previous: Some(previous),
                current,
                ..Self::default()
            },
            (Some(current), None) => Self {
                previous: None,
                current,
                ..Self::default()
            },
            (None, Some(_previous)) => panic!("Current edge is missing"),
            (None, None) => panic!("Current and previous edges are missing"),
        }
    }
    pub fn generate_statements(&self, edge_name: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        // Only include edge definition if this is a new edge (no previous definition)
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
            statements.push(format!("REMOVE FIELD {} ON TABLE {};", column, edge_name));
        }

        // Remove indexes that no longer exist
        for index in self.removed_indexes.keys() {
            statements.push(format!("REMOVE INDEX {} ON TABLE {};", index, edge_name));
        }

        // Remove events that no longer exist
        for event in self.removed_events.keys() {
            statements.push(format!("REMOVE EVENT {} ON TABLE {};", event, edge_name));
        }

        Ok(statements)
    }

    pub fn reverse(&self, edge_name: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        // Table definition (ensure OVERWRITE is present)
        statements.push(ensure_overwrite(self.previous.as_ref().unwrap()));

        for column in self.added_columns.keys() {
            statements.push(format!("REMOVE FIELD {} ON TABLE {};", column, edge_name));
        }

        for index in self.added_indexes.keys() {
            statements.push(format!("REMOVE INDEX {} ON TABLE {};", index, edge_name));
        }

        for event in self.added_events.keys() {
            statements.push(format!("REMOVE EVENT {} ON TABLE {};", event, edge_name));
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

impl Diff<EdgeSnapshot> for EdgeDiff {
    fn from_snapshots(old_edge: &EdgeSnapshot, new_edge: &EdgeSnapshot) -> Result<Self> {
        let mut diff = EdgeDiff::new(
            Some(old_edge.define_edge_statement.clone()),
            Some(new_edge.define_edge_statement.clone()),
        );
        diff.name = new_edge.name.clone();

        // Compare fields
        for (field_name, new_def) in &new_edge.fields {
            if let Some(old_def) = old_edge.fields.get(field_name) {
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
        for (field_name, old_def) in &old_edge.fields {
            if !new_edge.fields.contains_key(field_name) {
                diff.removed_columns
                    .insert(field_name.clone(), old_def.clone());
            }
        }

        // Compare indexes
        for (idx_name, new_def) in &new_edge.indexes {
            if let Some(old_def) = old_edge.indexes.get(idx_name) {
                if old_def.trim() != new_def.trim() {
                    diff.modified_indexes
                        .insert(idx_name.clone(), (old_def.clone(), new_def.clone()));
                }
            } else {
                diff.added_indexes.insert(idx_name.clone(), new_def.clone());
            }
        }

        for (idx_name, old_def) in &old_edge.indexes {
            if !new_edge.indexes.contains_key(idx_name) {
                diff.removed_indexes
                    .insert(idx_name.clone(), old_def.clone());
            }
        }

        // Compare events
        for (evt_name, new_def) in &new_edge.events {
            if let Some(old_def) = old_edge.events.get(evt_name) {
                if old_def.trim() != new_def.trim() {
                    diff.modified_events
                        .insert(evt_name.clone(), (old_def.clone(), new_def.clone()));
                }
            } else {
                diff.added_events.insert(evt_name.clone(), new_def.clone());
            }
        }

        for (evt_name, old_def) in &old_edge.events {
            if !new_edge.events.contains_key(evt_name) {
                diff.removed_events
                    .insert(evt_name.clone(), old_def.clone());
            }
        }

        Ok(diff)
    }

    fn to_snapshot(&self) -> crate::Result<EdgeSnapshot> {
        let mut snapshot = EdgeSnapshot::new(self.name.clone(), self.current.clone());

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