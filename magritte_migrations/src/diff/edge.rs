use crate::schema::{ColumnSnapshot, EventSnapshot, IndexSnapshot};
use magritte::prelude::define_edge::DefineEdgeStatement;
use magritte::prelude::{EdgeTrait, TableTrait};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EdgeDiff {
    pub name: String,
    pub previous: Option<
        DefineEdgeStatement<dyn EdgeTrait<EntityFrom = dyn TableTrait, EntityTo = dyn TableTrait>>,
    >,
    pub current:
        DefineEdgeStatement<dyn EdgeTrait<EntityFrom = dyn TableTrait, EntityTo = dyn TableTrait>>,
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
impl Default for EdgeDiff {
    fn default() -> Self {
        Self {
            name: Default::default(),
            previous: Default::default(),
            current: Default::default(),
            added_columns: Vec::new(),
            removed_columns: Vec::new(),
            modified_columns: HashMap::new(),
            added_indexes: Vec::new(),
            removed_indexes: Vec::new(),
            modified_indexes: HashMap::new(),
            added_events: Vec::new(),
            removed_events: Vec::new(),
            modified_events: HashMap::new(),
        }
    }
}
impl EdgeDiff {
    pub fn new(
        previous: Option<
            DefineEdgeStatement<dyn EdgeTrait<EntityFrom = dyn TableTrait, EntityTo = dyn TableTrait>>,
        >,
        current:
            Option<DefineEdgeStatement<dyn EdgeTrait<EntityFrom = dyn TableTrait, EntityTo = dyn TableTrait>>>,
    ) -> Self {
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
            (None, Some(previous)) => panic!("Current edge is missing"),
            (None, None) => panic!("Current and previous edges are missing"),
        }
    }
    pub fn generate_statements(&self, table_name: &str) -> Vec<String> {
        todo!()
    }

    pub fn reverse(&self) -> Self {
        todo!()
    }
}
