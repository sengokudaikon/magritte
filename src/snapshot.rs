use crate::{ColumnTrait, EdgeTrait, EventTrait, HasEvents, HasIndexes, IndexTrait, TableTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Basic table snapshot structure, can be extended as needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub name: String,
    pub define_table_statement: String,
    pub fields: HashMap<String, String>,
    pub indexes: HashMap<String, String>,
    pub events: HashMap<String, String>,
}

// An edge snapshot, similar to tables but for edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub name: String,
    pub define_edge_statement: String,
    pub fields: HashMap<String, String>,
    pub indexes: HashMap<String, String>,
    pub events: HashMap<String, String>,
}

// Full database schema snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaSnapshot {
    pub tables: HashMap<String, TableSnapshot>,
    pub edges: HashMap<String, EdgeSnapshot>,
}

impl SchemaSnapshot {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    // Utility function to insert a table
    pub fn add_table(&mut self, table: TableSnapshot) {
        self.tables.insert(table.name.clone(), table);
    }

    // Utility function to add an edge
    pub fn add_edge(&mut self, edge: EdgeSnapshot) {
        self.edges.insert(edge.name.clone(), edge);
    }
}
pub fn table_snapshot<T>() -> anyhow::Result<TableSnapshot>
where
    T: TableTrait,
{
    // Get main table definition
    let define_table = T::to_statement().build().map_err(anyhow::Error::from)?;

    // Extract fields (columns)
    let mut fields = HashMap::new();
    for col in <T as TableTrait>::columns() {
        let def = ColumnTrait::def(&col);
        if def.name() == "id" {
            continue;
        }
        let field_def = ColumnTrait::to_statement(&col)
            .build()
            .map_err(anyhow::Error::from)?;
        fields.insert(def.name().to_string(), field_def);
    }

    // Extract indexes
    let mut indexes = HashMap::new();
    for idx in T::indexes() {
        let def = IndexTrait::def(&idx);
        let idx_def = IndexTrait::to_statement(&idx)
            .build()
            .map_err(anyhow::Error::from)?;
        indexes.insert(def.index_name().to_string(), idx_def);
    }

    // Extract events
    let mut events = HashMap::new();
    for evt in T::events() {
        let def = EventTrait::def(&evt);
        let evt_def = EventTrait::to_statement(&evt)?
            .build()
            .map_err(anyhow::Error::from)?;
        events.insert(def.event_name().to_string(), evt_def);
    }

    let snapshot = TableSnapshot {
        name: T::table_name().to_string(),
        define_table_statement: define_table,
        fields,
        indexes,
        events,
    };

    Ok(snapshot)
}

pub fn edge_snapshot<T>() -> anyhow::Result<EdgeSnapshot>
where
    T: EdgeTrait,
{
    // Get main table definition
    let define_table = T::to_statement().build().map_err(anyhow::Error::from)?;

    // Extract fields (columns)
    let mut fields = HashMap::new();
    for col in <T as EdgeTrait>::columns() {
        let def = ColumnTrait::def(&col);
        if def.name() == "id" {
            continue;
        }
        let field_def = ColumnTrait::to_statement(&col)
            .build()
            .map_err(anyhow::Error::from)?;
        fields.insert(def.name().to_string(), field_def);
    }

    // Extract indexes
    let mut indexes = HashMap::new();
    for idx in T::indexes() {
        let def = IndexTrait::def(&idx);
        let idx_def = IndexTrait::to_statement(&idx)
            .build()
            .map_err(anyhow::Error::from)?;
        indexes.insert(def.index_name().to_string(), idx_def);
    }

    // Extract events
    let mut events = HashMap::new();
    for evt in T::events() {
        let def = EventTrait::def(&evt);
        let evt_def = EventTrait::to_statement(&evt)?
            .build()
            .map_err(anyhow::Error::from)?;
        events.insert(def.event_name().to_string(), evt_def);
    }

    let snapshot = EdgeSnapshot {
        name: T::table_name().to_string(),
        define_edge_statement: define_table,
        fields,
        indexes,
        events,
    };

    Ok(snapshot)
}

/// Build a full `SchemaSnapshot` from code. If you have multiple tables, you might call this multiple times for each table type, or have a registry of tables.
pub fn full_schema<T, E>() -> anyhow::Result<SchemaSnapshot>
where
    T: TableTrait,
    E: EdgeTrait,
{
    let mut schema = SchemaSnapshot::new();
    let table_snap = table_snapshot::<T>()?;
    let edge_snap = edge_snapshot::<E>()?;
    schema.add_table(table_snap);
    schema.add_edge(edge_snap);
    Ok(schema)
}

pub fn empty_schema() -> SchemaSnapshot {
    SchemaSnapshot::new()
}
