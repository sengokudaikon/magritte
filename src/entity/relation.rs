use crate::{EdgeTrait, TableTrait};
use anyhow::Result;
use async_trait::async_trait;
use magritte_query::types::RelationType;
use magritte_query::{HasId, Query, RelateStatement, SelectStatement, SurrealId};
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadStrategy {
    Eager,
    Lazy,
    Default,
}

/// Defines a Relation between Tables through an Edge
#[derive(Debug, Clone, PartialEq)]
pub struct RelationDef {
    pub(crate) from: String,                        // Source record
    pub(crate) to: String,                          // Target record
    pub(crate) via: String,                         // Edge Table name
    pub(crate) content: Option<String>,             // Optional content for the edge
    pub(crate) load_strategy: Option<LoadStrategy>, // How to load this relation
}

#[async_trait]
pub trait RelationTrait: RelationType {
    type Source: TableTrait; // The Table that owns this relation
    type Edge: EdgeTrait + HasId; // The edge type for this relation
    type Target: TableTrait + HasId; // The target table type for this relation

    /// Get the relation definition
    fn def() -> RelationDef
    where
        Self: Sized;

    fn def_owned(&self) -> RelationDef {
        Self::def()
    }

    /// Create a relate statement for this relation
    fn to_statement(&self) -> Result<RelateStatement> {
        let def = Self::def();
        let mut stmt = Query::relate()
            .from_record(&def.from)
            .to_record(&def.to)
            .edge_table(&def.via);

        if let Some(content) = def.content {
            stmt = stmt.content(content).map_err(anyhow::Error::from)?;
        }
        Ok(stmt)
    }

    /// Build a query to load related entities
    fn build_load_query(entity_id: &str) -> SelectStatement<Self::Target> {
        Query::select()
            .field("*", None)
            .relation_wildcard_as("related")
            .fetch(&["related"])
            .where_id(SurrealId::<Self::Target>::from(entity_id))
    }

    /// Check if this relation should be loaded eagerly
    fn should_load_eagerly() -> bool {
        matches!(Self::def().load_strategy, Some(LoadStrategy::Eager))
    }
}

impl RelationDef {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        via: impl Into<String>,
        content: impl Into<Option<String>>,
        load_strategy: Option<LoadStrategy>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            via: via.into(),
            content: content.into(),
            load_strategy,
        }
    }

    pub fn relation_name(&self) -> &str {
        self.via.as_str()
    }
    pub fn relation_from(&self) -> &str {
        self.from.as_str()
    }
    pub fn relation_to(&self) -> &str {
        self.to.as_str()
    }
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }
    pub fn load_strategy(&self) -> LoadStrategy {
        self.load_strategy.unwrap_or(LoadStrategy::Default)
    }
}
