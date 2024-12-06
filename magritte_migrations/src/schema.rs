//! # Schema Snapshot
//!
//! This module provides functions for capturing and comparing the schema state of a SurrealDB database.

use magritte::entity::table::{TableWithEvents, TableWithIndexes};
use magritte::prelude::define::define_table::DefineTableStatement;
use magritte::prelude::define_edge::DefineEdgeStatement;
use magritte::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::Digest;

pub mod snapshot;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub name: String,
    pub definition: Option<DefineTableStatement<dyn TableTrait>>,
    pub definition_lit: Option<String>,
    pub columns: Vec<ColumnSnapshot>,
    pub indexes: Vec<IndexSnapshot>,
    pub events: Vec<EventSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub name: String,
    pub definition: Option<DefineEdgeStatement<dyn EdgeTrait<EntityFrom=dyn TableTrait, EntityTo=dyn TableTrait>>>,
    pub definition_lit: Option<String>,
    pub columns: Vec<ColumnSnapshot>,
    pub indexes: Vec<IndexSnapshot>,
    pub events: Vec<EventSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSnapshot {
    pub name: String,
    pub definition: Option<DefineFieldStatement>,
    pub definition_lit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub name: String,
    pub definition: Option<DefineIndexStatement>,
    pub definition_lit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSnapshot {
    pub name: String,
    pub definition: Option<DefineEventStatement>,
    pub definition_lit: Option<String>,
}


