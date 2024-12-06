use crate::diff::edge::EdgeDiff;
use crate::diff::table::TableDiff;
use crate::schema::snapshot::SchemaSnapshot;
use magritte::prelude::{EdgeTrait, TableTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) mod edge;
pub(crate) mod table;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaDiff {
    pub added_tables: Vec<String>,
    pub removed_tables: Vec<String>,
    pub added_edges: Vec<String>,
    pub removed_edges: Vec<String>,
    pub modified_tables: HashMap<String, TableDiff>,
    pub modified_edges: HashMap<String, EdgeDiff>,
    pub source: SchemaSnapshot,
    pub target: SchemaSnapshot,
}

impl Default for SchemaDiff {
    fn default() -> Self {
        Self {
            added_tables: Vec::new(),
            removed_tables: Vec::new(),
            added_edges: Vec::new(),
            removed_edges: Vec::new(),
            modified_tables: HashMap::new(),
            modified_edges: HashMap::new(),
            source: SchemaSnapshot::default(),
            target: SchemaSnapshot::default(),
        }
    }
}

impl SchemaDiff {
    pub fn generate_statements(&self) -> anyhow::Result<Vec<String>> {
        let mut statements = Vec::new();

        // Handle table modifications
        for (table_name, table_diff) in &self.modified_tables {
            statements.extend(table_diff.generate_statements(table_name));
        }

        // Handle table additions/removals
        for table_name in &self.added_tables {
            if let Some(table) = self.target.tables.get(table_name) {
                if let Some(def) = &table.definition {
                    statements.push(def.clone().build().map_err(anyhow::Error::from)?);
                }
            }
        }

        for table_name in &self.removed_tables {
            statements.push(format!("REMOVE TABLE {};", table_name));
        }

        // Handle edge modifications
        for (edge_name, edge_diff) in &self.modified_edges {
            statements.extend(edge_diff.generate_statements(edge_name));
        }

        // Handle edge additions/removals
        for edge_name in &self.added_edges {
            if let Some(edge) = self.target.edges.get(edge_name) {
                if let Some(def) = &edge.definition {
                    statements.push(
                        def.clone()
                            .overwrite()
                            .build()
                            .map_err(anyhow::Error::from)?,
                    );
                }
            }
        }

        for edge_name in &self.removed_edges {
            statements.push(format!("REMOVE TABLE {};", edge_name));
        }

        Ok(statements)
    }

    pub fn reverse(&self) -> Self {
        Self {
            added_tables: self.removed_tables.clone(),
            removed_tables: self.added_tables.clone(),
            added_edges: self.removed_edges.clone(),
            removed_edges: self.added_edges.clone(),
            modified_tables: self
                .modified_tables
                .iter()
                .map(|(k, v)| (k.clone(), v.reverse()))
                .collect(),
            modified_edges: self
                .modified_edges
                .iter()
                .map(|(k, v)| (k.clone(), v.reverse()))
                .collect(),
            source: self.target.clone(),
            target: self.source.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added_tables.is_empty()
            && self.removed_tables.is_empty()
            && self.modified_tables.is_empty()
            && self.modified_edges.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
    }
}
