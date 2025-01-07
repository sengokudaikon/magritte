//! Database definition functionality for SurrealDB.
//!
//! This module provides functionality to define databases in SurrealDB,
//! allowing you to instantiate named databases with security and configuration options.
//!
//! See [SurrealDB Database Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/database)
//!
//! # Example
//!
//! ```rust
//! use magritte_query::DefineDatabaseStatement;
//!
//! // Define a new database
//! let stmt = DefineDatabaseStatement::new().name("app_vitalsense")
//!     .comment("Main application database")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root owner/editor or namespace owner/editor
//! - Selected namespace before using the statement

use crate::SurrealDB;
use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};

/// Statement for defining a database in SurrealDB
#[derive(Clone, Debug, Default)]
pub struct DefineDatabaseStatement {
    pub(crate) name: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineDatabaseStatement {
    /// Creates a new database definition statement with the given name
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the OVERWRITE clause
    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Sets the IF NOT EXISTS clause
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a comment to the database definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the database definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let name = self
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Database name is required"))?;
        if name.is_empty() {
            bail!("Database name is required");
        }
        let mut stmt = String::new();
        stmt.push_str("DEFINE DATABASE ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        stmt.push_str(name);

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the database definition statement on the database
    pub async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        let query = self.build()?;
        info!("Executing query: {}", query);

        let surreal_query = conn.query(query);

        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}

impl Display for DefineDatabaseStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_database() {
        let stmt = DefineDatabaseStatement::new()
            .name("app_vitalsense")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE DATABASE app_vitalsense;");
    }

    #[test]
    fn test_database_with_comment() {
        let stmt = DefineDatabaseStatement::new()
            .name("app_vitalsense")
            .comment("Main application database")
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE DATABASE app_vitalsense COMMENT \"Main application database\";"
        );
    }

    #[test]
    fn test_database_with_overwrite() {
        let stmt = DefineDatabaseStatement::new()
            .name("app_vitalsense")
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE DATABASE OVERWRITE app_vitalsense;");
    }

    #[test]
    fn test_database_if_not_exists() {
        let stmt = DefineDatabaseStatement::new()
            .name("app_vitalsense")
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE DATABASE IF NOT EXISTS app_vitalsense;");
    }

    #[test]
    fn test_empty_name() {
        let stmt = DefineDatabaseStatement::new().name("").build();
        assert!(stmt.is_err());
    }
}
