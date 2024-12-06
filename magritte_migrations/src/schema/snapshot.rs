use crate::diff::edge::EdgeDiff;
use crate::diff::table::TableDiff;
use crate::diff::SchemaDiff;
use crate::schema::{ColumnSnapshot, EdgeSnapshot, EventSnapshot, IndexSnapshot, TableSnapshot};
use crate::MigrationContext;
use anyhow::bail;
use magritte::entity::table::{TableWithEvents, TableWithIndexes};
use magritte::prelude::{ColumnDef, EdgeTrait, EventDef, IndexDef, TableTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSnapshot {
    pub version: String,
    pub checksum: String,
    pub tables: HashMap<String, TableSnapshot>,
    pub edges: HashMap<String, EdgeSnapshot>,
}
impl Default for SchemaSnapshot {
    fn default() -> Self {
        Self {
            version: String::new(),
            checksum: String::new(),
            tables: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}
impl SchemaSnapshot {
    /// Capture the current schema state from the database
    pub async fn capture(ctx: &MigrationContext) -> anyhow::Result<Self> {
        let mut tables = HashMap::new();
        let mut edges = HashMap::new();

        // Query INFO FOR DB to get all tables and edges
        let info = ctx.db.query("INFO FOR DB").await?;

        // TODO: Parse the response and build snapshots
        // This will require careful handling of SurrealDB's INFO response format

        Ok(Self {
            version: chrono::Utc::now().to_rfc3339(),
            checksum: "".to_string(), // TODO: Implement checksum
            tables,
            edges,
        })
    }

    /// Create a snapshot from entity definitions
    pub fn from_entities<T: IntoIterator<Item = impl TableTrait>>(
        entities: T,
    ) -> anyhow::Result<Self> {
        let mut tables = HashMap::new();

        for entity in entities {
            tables.insert(
                entity.def_owned().table_name().to_string(),
                TableSnapshot {
                    name: entity.def_owned().table_name().to_string(),
                    definition: Some(entity.to_statement_owned()),
                    definition_lit: Some(entity.to_statement_owned().build()?),
                    columns: entity
                        .columns()
                        .into_iter()
                        .map(|col| {
                            let def: ColumnDef = col.def();
                            ColumnSnapshot {
                                name: def.name().to_string(),
                                definition: Some(def.to_statement()),
                                definition_lit: Some(def.to_statement().build()?),
                            }
                        })
                        .collect(),
                    indexes: entity
                        .indexes()
                        .into_iter()
                        .map(|idx| {
                            let def: IndexDef = idx.def();
                            IndexSnapshot {
                                definition: Some(def.to_statement()),
                                name: def.index_name().to_string(),
                                definition_lit: Some(def.to_statement().build()?),
                            }
                        })
                        .collect(),
                    events: entity
                        .events()
                        .into_iter()
                        .map(|evt| {
                            let def: EventDef = evt.def();
                            EventSnapshot {
                                definition: Some(def.to_statement()?),
                                name: def.event_name().to_string(),
                                definition_lit: Some(def.to_statement()?.build()?),
                            }
                        })
                        .collect(),
                },
            );
        }

        Ok(Self {
            version: chrono::Utc::now().to_rfc3339(),
            checksum: "".to_string(), // TODO: Implement checksum
            tables,
            edges: HashMap::new(),
        })
    }

    pub fn from_edge_entities<T: IntoIterator<Item = impl EdgeTrait>>(
        entities: T,
    ) -> anyhow::Result<Self> {
        let mut edges = HashMap::new();

        for entity in entities {
            edges.insert(
                entity.def_owned().edge_name().to_string(),
                EdgeSnapshot {
                    name: entity.def_owned().edge_name().to_string(),
                    definition: Some(entity.to_statement_owned()),
                    columns: entity
                        .columns()
                        .into_iter()
                        .map(|col| {
                            let def: ColumnDef = col.def();
                            ColumnSnapshot {
                                name: def.name().to_string(),
                                definition: Some(def.to_statement()),
                                definition_lit: Some(def.to_statement().build()?),
                            }
                        })
                        .collect(),
                    definition_lit: Some(entity.to_statement_owned().build()?),
                    indexes: entity
                        .indexes()
                        .into_iter()
                        .map(|idx| {
                            let def: IndexDef = idx.def();
                            IndexSnapshot {
                                definition: Some(def.to_statement()),
                                name: def.index_name().to_string(),
                                definition_lit: Some(def.to_statement().build()?),
                            }
                        })
                        .collect(),
                    events: entity
                        .events()
                        .into_iter()
                        .map(|evt| {
                            let def: EventDef = evt.def();
                            EventSnapshot {
                                definition: Some(def.to_statement()?),
                                name: def.event_name().to_string(),
                                definition_lit: Some(def.to_statement()?.build()?),
                            }
                        })
                        .collect(),
                },
            );
        }

        Ok(Self {
            version: chrono::Utc::now().to_rfc3339(),
            checksum: "".to_string(), // TODO: Implement checksum
            tables: HashMap::new(),
            edges,
        })
    }

    /// Compare two schema snapshots and generate a diff
    pub fn diff(&self, other: &Self) -> SchemaDiff {
        let mut diff = SchemaDiff::default();

        // Find added and removed tables
        for table_name in self.tables.keys() {
            if !other.tables.contains_key(table_name) {
                diff.removed_tables.push(table_name.clone());
            }
        }

        for table_name in other.tables.keys() {
            if !self.tables.contains_key(table_name) {
                diff.added_tables.push(table_name.clone());
            }
        }

        // Compare modified tables
        for (table_name, new_table) in other.tables.iter() {
            if let Some(old_table) = self.tables.get(table_name) {
                if let Some(old_definition) = &old_table.definition_lit {
                    if let Some(new_definition) = &new_table.definition_lit {
                        if old_definition != new_definition {
                            diff.modified_tables.insert(
                                table_name.clone(),
                                TableDiff::new(
                                    old_table.clone().definition,
                                    new_table.clone().definition,
                                ),
                            );
                        }
                    }
                }
            }
        }

        // Similar for edges
        for edge_name in self.edges.keys() {
            if !other.edges.contains_key(edge_name) {
                diff.removed_edges.push(edge_name.clone());
            }
        }

        for edge_name in other.edges.keys() {
            if !self.edges.contains_key(edge_name) {
                diff.added_edges.push(edge_name.clone());
            }
        }

        // Compare modified edges
        for (edge_name, new_edge) in other.edges.iter() {
            if let Some(old_edge) = self.edges.get(edge_name) {
                if let Some(old_definition) = &old_edge.definition_lit {
                    if let Some(new_definition) = &new_edge.definition_lit {
                        if old_definition != new_definition {
                            diff.modified_edges.insert(
                                edge_name.clone(),
                                EdgeDiff::new(
                                    old_edge.clone().definition,
                                    new_edge.clone().definition,
                                ),
                            );
                        }
                    }
                }
            }
        }

        diff
    }

    /// Save the snapshot as a .surql file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let mut content = String::new();

        // Add metadata
        content.push_str(&format!("-- Schema snapshot version: {}\n", self.version));
        content.push_str(&format!("-- Checksum: {}\n\n", self.checksum));

        // Add tables
        for (_, table) in &self.tables {
            if let Some(definition) = &table.definition {
                content.push_str(definition.build().map_err(anyhow::Error::from)?.as_str());
            } else {
                bail!("No definition provided for table {}", table.name);
            }
            // Column definitions
            for col in &table.columns {
                if let Some(definition) = &col.definition {
                    content.push_str(definition.build().map_err(anyhow::Error::from)?.as_str());
                } else {
                    bail!("No definition provided for column {}", col.name);
                }
            }

            // Index definitions
            for idx in &table.indexes {
                if let Some(definition) = &idx.definition {
                    content.push_str(definition.build().map_err(anyhow::Error::from)?.as_str());
                } else {
                    bail!("No definition provided for index {}", idx.name);
                }
            }

            // Event definitions
            for evt in &table.events {
                if let Some(definition) = &evt.definition {
                    content.push_str(definition.build().map_err(anyhow::Error::from)?.as_str());
                } else {
                    bail!("No definition provided for event {}", evt.name);
                }
            }

            content.push('\n');
        }

        // Add edges
        for (_, edge) in &self.edges {
            if let Some(definition) = &edge.definition {
                content.push_str(definition.build().map_err(anyhow::Error::from)?.as_str());
            } else {
                bail!("No definition provided for edge {}", edge.name);
            }
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Load a snapshot from a .surql file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse_surql(&content)
    }

    /// Parse SurrealQL content into a snapshot
    fn parse_surql(content: &str) -> anyhow::Result<Self> {
        let mut tables = HashMap::new();
        let mut edges = HashMap::new();
        let mut current_table = None;
        let mut version = String::new();
        let mut checksum = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("--") {
                // Handle metadata comments
                if line.starts_with("-- Schema snapshot version:") {
                    version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("-- Checksum:") {
                    checksum = line.split(':').nth(1).unwrap_or("").trim().to_string();
                }
                continue;
            }

            if line.starts_with("DEFINE TABLE") {
                let parts: Vec<_> = line.split_whitespace().collect();
                let table_name = parts[2].to_string();

                if line.contains("TYPE RELATION") {
                    // Edge table
                    edges.insert(
                        table_name.clone(),
                        EdgeSnapshot {
                            name: table_name,
                            definition: None,
                            definition_lit: Some(line.to_string()),
                            columns: Vec::new(),
                            indexes: Vec::new(),
                            events: Vec::new(),
                        },
                    );
                } else if line.contains("TYPE NORMAL") {
                    // Regular table
                    current_table = Some(table_name.clone());
                    tables.insert(
                        table_name.clone(),
                        TableSnapshot {
                            name: table_name,
                            definition_lit: Some(line.to_string()),
                            definition: None,
                            columns: Vec::new(),
                            indexes: Vec::new(),
                            events: Vec::new(),
                        },
                    );
                } else {
                    bail!("ANY type tables are unsupported, use properly typed tables.")
                }
            } else if line.starts_with("DEFINE FIELD") && current_table.is_some() {
                let table_name = current_table.as_ref().unwrap();
                if let Some(table) = tables.get_mut(table_name) {
                    // Parse field definition
                    let parts: Vec<_> = line.split_whitespace().collect();
                    let field_name = parts[2].to_string();

                    table.columns.push(ColumnSnapshot {
                        name: field_name,
                        definition_lit: Some(line.to_string()),
                        definition: None,
                    });
                }
            } else if line.starts_with("DEFINE INDEX") && current_table.is_some() {
                let table_name = current_table.as_ref().unwrap();
                if let Some(table) = tables.get_mut(table_name) {
                    // Parse index definition
                    let parts: Vec<_> = line.split_whitespace().collect();
                    let index_name = parts[2].to_string();

                    table.indexes.push(IndexSnapshot {
                        name: index_name,
                        definition_lit: Some(line.to_string()),
                        definition: None,
                    });
                }
            } else if line.starts_with("DEFINE EVENT") && current_table.is_some() {
                let table_name = current_table.as_ref().unwrap();
                if let Some(table) = tables.get_mut(table_name) {
                    // Parse event definition
                    let parts: Vec<_> = line.split_whitespace().collect();
                    let event_name = parts[2].to_string();

                    let when_start = line.find("WHEN").unwrap_or(0) + 5;
                    let when_end = line.find("THEN").unwrap_or(line.len());
                    let when = line[when_start..when_end].trim().to_string();

                    let then_start = line.find("THEN").unwrap_or(0) + 5;
                    let then_end = line.rfind(';').unwrap_or(line.len());
                    let then = line[then_start..then_end].trim().to_string();

                    table.events.push(EventSnapshot {
                        name: event_name,
                        definition_lit: Some(line.to_string()),
                        definition: None,
                    });
                }
            }
        }

        Ok(Self {
            version,
            checksum,
            tables,
            edges,
        })
    }

    /// Get the path for snapshot files
    pub fn get_snapshot_path(base_dir: impl AsRef<Path>, version: &str) -> PathBuf {
        let mut path = PathBuf::from(base_dir.as_ref());
        path.push("snapshots");
        fs::create_dir_all(&path).expect("Failed to create snapshots directory");
        path.push(format!("schema_{}.surql", version));
        path
    }

    /// Save the current schema state to both DB and file
    pub async fn save(
        &self,
        ctx: &MigrationContext,
        base_dir: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        // Save to file
        let path = Self::get_snapshot_path(base_dir, &self.version);
        self.save_to_file(path)?;

        // Save to DB
        ctx.db
            .query(
                "
        LET $snapshot = {
            version: type::string($version),
            checksum: type::string($checksum),
            created_at: time::now()
        };
        UPDATE $_schema_snapshots MERGE $snapshot;
    ",
            )
            .bind(("version", &self.version))
            .bind(("checksum", &self.checksum))
            .await?;

        Ok(())
    }
}
