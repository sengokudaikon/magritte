#![feature(duration_constructors)]
#![feature(associated_type_defaults)]
#![feature(const_type_id)]
#![allow(unused)]
#![allow(clippy::wrong_self_convention)]
//! Magritte - A powerful QueryBuilder for SurrealDB
//!
//! Named after René Magritte, a Belgian surrealist artist.
//! This crate provides a type-safe query
//! builder for SurrealDB with enhanced schema support.

mod defs;
pub mod entity;
pub mod entity_crud;
pub mod snapshot;

pub use entity::relation::LoadStrategy;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use cfg_if::cfg_if;
use thiserror::Error;

/// Error type for magritte operations
#[derive(Debug, Error)]
pub enum MagritteError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid query: {0}")]
    Query(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Error: {0}")]
    Any(#[from] anyhow::Error),
}

/// Error during `impl FromStr for Table::Column`
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Column")]
pub struct ColumnFromStrErr(pub String);
impl ColumnFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        ColumnFromStrErr(s.into())
    }
}
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Table")]
pub struct TableFromStrErr(pub String);
impl TableFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        TableFromStrErr(s.into())
    }
}
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Event")]
pub struct EventFromStrErr(pub String);
impl EventFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        EventFromStrErr(s.into())
    }
}
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Index")]
pub struct IndexFromStrErr(pub String);
impl IndexFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        IndexFromStrErr(s.into())
    }
}
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Relation")]
pub struct RelationFromStrErr(pub String);
impl RelationFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        RelationFromStrErr(s.into())
    }
}
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Edge")]
pub struct EdgeFromStrErr(pub String);
impl EdgeFromStrErr {
    pub fn new(s: impl Into<String>) -> Self {
        EdgeFromStrErr(s.into())
    }
}

// Re-exports for convenience
pub use crate::snapshot::*;
pub use defs::*;
pub use entity::column::ColumnTrait;
pub use entity::edge::EdgeTrait;
pub use entity::event::EventTrait;
pub use entity::index::IndexTrait;
pub use entity::relation::RelationTrait;
pub use entity::table::TableTrait;
pub use entity::HasColumns;
pub use entity::HasEvents;
pub use entity::HasIndexes;
pub use entity::HasRelations;
pub use magritte_macros::EnumIter;
pub use magritte_macros::*;
pub use magritte_macros::{Column, Edge, Event, Index, Relation, Table};
pub use magritte_query::types::{EventType, IndexType, RelationType};
pub use magritte_query::*;
pub use strum;
pub use surrealdb::RecordId;
pub use RecordType;
#[derive(Clone)]
pub struct TableRegistration {
    pub builder: fn() -> anyhow::Result<TableSnapshot>,
    pub type_id: TypeId,
}

#[derive(Clone)]
pub struct EdgeRegistration {
    pub builder: fn() -> anyhow::Result<EdgeSnapshot>,
    pub type_id: TypeId,
}

#[derive(Clone)]
pub struct EventRegistration {
    pub builder: fn() -> Vec<EventDef>,
    pub type_id: std::any::TypeId,
}

impl EventRegistration {
    pub fn new<T: NamedType + 'static>(builder: fn() -> Vec<EventDef>) -> Self {
        Self {
            builder,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

#[derive(Clone)]
pub struct IndexRegistration {
    pub builder: fn() -> Vec<IndexDef>,
    pub type_id: std::any::TypeId,
}

impl IndexRegistration {
    pub fn new<T: NamedType + 'static>(builder: fn() -> Vec<IndexDef>) -> Self {
        Self {
            builder,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

// Global inventory of `TableRegistration` entries
inventory::collect!(TableRegistration);

// Global inventory of `EdgeRegistration` entries
inventory::collect!(EdgeRegistration);

// Global inventory of `EventRegistration` entries
inventory::collect!(EventRegistration);

// Global inventory of `IndexRegistration` entries
inventory::collect!(IndexRegistration);

use crate::entity_crud::SurrealCrud;

#[cfg(feature = "uuid")]
pub use magritte_query::uuid::*;
