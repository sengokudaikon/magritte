use crate::{ColumnTrait, EdgeTrait, EventTrait, HasEvents, HasIndexes, IndexTrait, TableTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use tracing::event;

// Basic table snapshot structure, can be extended as needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub name: String,
    pub define_table_statement: String,
    pub fields: HashMap<String, String>,
    pub indexes: HashMap<String, String>,
    pub events: HashMap<String, String>,
}

impl Default for TableSnapshot {
    fn default() -> Self {
        Self {
            name: String::new(),
            define_table_statement: String::new(),
            fields: HashMap::new(),
            indexes: HashMap::new(),
            events: HashMap::new(),
        }
    }
}
impl TableSnapshot {
    pub fn new(name: String, define_table_statement: String) -> Self {
        Self {
            name,
            define_table_statement,
            ..Default::default()
        }
    }
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
impl Default for EdgeSnapshot {
    fn default() -> Self {
        Self {
            name: String::new(),
            define_edge_statement: String::new(),
            fields: HashMap::new(),
            indexes: HashMap::new(),
            events: HashMap::new(),
        }
    }
}
impl EdgeSnapshot {
    pub fn new(name: String, define_edge_statement: String) -> Self {
        Self {
            name,
            define_edge_statement,
            ..Default::default()
        }
    }

}

impl Snapshot for TableSnapshot {
    fn add_field(&mut self, field_name: String, field: String) {
        self.fields.insert(field_name, field);
    }

    fn add_index(&mut self, index_name: String, index: String) {
        self.indexes.insert(index_name, index);
    }

    fn add_event(&mut self, event_name: String, event: String) {
        self.events.insert(event_name, event);
    }
}

impl Snapshot for EdgeSnapshot {
    fn add_field(&mut self, field_name: String, field: String) {
        self.fields.insert(field_name, field);
    }

    fn add_index(&mut self, index_name: String, index: String) {
        self.indexes.insert(index_name, index);
    }

    fn add_event(&mut self, event_name: String, event: String) {
        self.events.insert(event_name, event);
    }
}

pub trait Snapshot: Debug + Clone + Serialize + DeserializeOwned{
    fn add_field(&mut self, field_name: String, field: String);
    fn add_index(&mut self, index_name: String, index: String);
    fn add_event(&mut self, event_name: String, event: String);
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

    let snapshot = TableSnapshot {
        name: T::table_name().to_string(),
        define_table_statement: define_table,
        fields,
        ..Default::default()
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

    let snapshot = EdgeSnapshot {
        name: T::table_name().to_string(),
        define_edge_statement: define_table,
        fields,
        indexes: HashMap::new(),
        events: HashMap::new(),
    };

    Ok(snapshot)
}

/// Build a full `SchemaSnapshot` from code. If you have multiple tables, you might call this multiple times for each table type, or have a registry of tables.
pub fn full_snapshot<T, E>() -> anyhow::Result<SchemaSnapshot>
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
