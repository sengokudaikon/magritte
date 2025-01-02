//! Magritte Migrations is a schema migration tool for SurrealDB.
//!
//! This crate provides functionality to:
//! - Create and manage database schema migrations
//! - Introspect existing database schemas
//! - Generate and apply schema changes
//! - Handle rollbacks to previous schema versions
//!
//! # Features
//!
//! - Schema snapshots for tables and edges
//! - Migration versioning with timestamps
//! - Schema validation and deviation reporting
//! - Safe schema updates with OVERWRITE semantics
//! - Transaction support for atomic changes
//!
//! # Example
//!
//! ```rust,ignore
//! use magritte_migrations::{manager::MigrationManager, Result};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     use std::sync::Arc;
//!     use surrealdb::engine::any::connect;
//!     let manager = MigrationManager::new(PathBuf::from("./migrations"));
//!
//!     // Create a new migration from current schema
//!     let (path, statements) = manager.create_snapshot(None, None).await?;
//!     let db = Arc::new(connect(None).await?);
//!     // Apply the migration
//!     manager.apply_migration(db, None).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Note on Relations
//!
//! While the crate handles edge table schemas, actual relation data migrations (`RELATE` statements)
//! must be handled manually as they depend on application-specific record IDs.

#![feature(const_type_id)]

pub use error::Error;
pub use error::Result;
use magritte::Snapshot;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod edge;
pub mod error;
pub mod introspection;
pub mod manager;
pub mod snapshot;
pub mod table;
pub mod test_models;
pub mod types;

/// Ensures that schema definition statements use OVERWRITE semantics.
///
/// This function modifies schema statements to use OVERWRITE instead of IF NOT EXISTS
/// to ensure consistent schema updates.
pub(crate) fn ensure_overwrite(stmt: &str) -> String {
    let stmt = stmt.trim();
    if stmt.contains("IF NOT EXISTS") {
        stmt.replace("IF NOT EXISTS", "OVERWRITE")
    } else if !stmt.contains("OVERWRITE") {
        // Split into parts: DEFINE <TYPE> <NAME>
        let parts: Vec<&str> = stmt.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "DEFINE" {
            let def_type = parts[1]; // TABLE, FIELD, etc.
            let rest: Vec<&str> = parts[2..].to_vec();
            format!("DEFINE {} OVERWRITE {}", def_type, rest.join(" "))
        } else {
            stmt.to_string()
        }
    } else {
        stmt.to_string()
    }
}

/// Trait for computing differences between schema snapshots.
///
/// This trait is implemented by types that can compute the differences between
/// two schema snapshots and generate the necessary SQL statements to migrate
/// from one to the other.
pub trait Diff<T>: Debug + Clone + Serialize + DeserializeOwned + Default where T: Snapshot {
    /// Creates a diff between two snapshots
    fn from_snapshots(
        old: &T,
        new: &T,
    ) -> crate::Result<Self>;

    /// Converts the diff back into a snapshot
    fn to_snapshot(&self) -> crate::Result<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_overwrite() {
        let cases = vec![
            (
                "DEFINE TABLE test TYPE NORMAL SCHEMAFULL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL SCHEMAFULL"
            ),
            (
                "DEFINE FIELD title ON TABLE test TYPE string",
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string"
            ),
            (
                "DEFINE TABLE IF NOT EXISTS test TYPE NORMAL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL"
            ),
            (
                "DEFINE TABLE OVERWRITE test TYPE NORMAL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL"
            ),
            (
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string",
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string"
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(ensure_overwrite(input), expected, "Failed for input: {}", input);
        }
    }
}
