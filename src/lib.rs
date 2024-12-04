#![feature(duration_constructors)]
#![allow(unused)]
#![allow(clippy::wrong_self_convention)]
//! magritte - A powerful QueryBuilder for SurrealDB
//!
//! Named after René Magritte, a Belgian surrealist artist.
//! This crate provides a type-safe query
//! builder for SurrealDB with enhanced schema support.

pub mod entity;
pub mod prelude;
mod defs;
pub mod entity_crud;

use std::collections::HashMap;
use thiserror::Error;

/// Result type for magritte operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for magritte operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid query: {0}")]
    Query(String),

    #[error("Schema error: {0}")]
    Schema(String),
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