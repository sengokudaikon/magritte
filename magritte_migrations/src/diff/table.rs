use crate::schema::{ColumnSnapshot, EventSnapshot, IndexSnapshot};
use magritte::prelude::define::define_table::DefineTableStatement;
use magritte::prelude::TableTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff {
    pub previous: Option<DefineTableStatement<dyn TableTrait>>,
    pub current: DefineTableStatement<dyn TableTrait>,
    pub added_columns: Vec<ColumnSnapshot>,
    pub removed_columns: Vec<String>,
    pub modified_columns: HashMap<String, ColumnSnapshot>,
    pub added_indexes: Vec<IndexSnapshot>,
    pub removed_indexes: Vec<String>,
    pub modified_indexes: HashMap<String, IndexSnapshot>,
    pub added_events: Vec<EventSnapshot>,
    pub removed_events: Vec<String>,
    pub modified_events: HashMap<String, EventSnapshot>,
}

impl Default for TableDiff {
    fn default() -> Self {
        Self {
            previous: Default::default(),
            current: Default::default(),
            added_columns: Default::default(),
            removed_columns: Default::default(),
            modified_columns: Default::default(),
            added_indexes: Default::default(),
            removed_indexes: Default::default(),
            modified_indexes: Default::default(),
            added_events: Default::default(),
            removed_events: Default::default(),
            modified_events: Default::default(),
        }
    }
}

impl TableDiff {
    pub fn new(
        previous: Option<DefineTableStatement<dyn TableTrait>>,
        current: Option<DefineTableStatement<dyn TableTrait>>,
    ) -> Self {
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
            (None, Some(previous)) => panic!("Current table is not defined"),
            (None, None) => panic!("Both tables are not defined"),
        }
    }
    pub fn generate_statements(&self, table_name: &str) -> Vec<String> {
        todo!()
    }

    pub fn reverse(&self) -> Self {
        todo!()
    }
}
