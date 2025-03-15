//! Namespace definition functionality for SurrealDB.
//!
//! This module provides functionality to define namespaces in SurrealDB. Namespaces are used
//! to scope databases and provide multi-tenancy support.
//!
//! See [SurrealDB Namespace Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/namespace)
//! for more details.
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Create a basic namespace
//! let namespace = Define::namespace()
//!     .name("tenant_1")
//!     .comment("Namespace for Tenant 1")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root owner or editor
//! - Root access privileges

use magritte_db::{db, QueryType, SurrealDB};
use anyhow::bail;
use std::fmt::Display;
use tracing::{error, info};

/// Statement for defining namespaces in SurrealDB.
///
/// A namespace is a logical container that can hold multiple databases, providing
/// multi-tenancy support and data isolation between different tenants.
///
/// See [DEFINE NAMESPACE Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/namespace)
///
/// # Example
///
/// ```rust
/// use magritte_query::define::*;
///
/// // Create a namespace with IF NOT EXISTS clause
/// let namespace = Define::namespace()
///     .name("tenant_1")
///     .if_not_exists()
///     .comment("Production tenant namespace")
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct DefineNamespaceStatement {
    pub(crate) name: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineNamespaceStatement {
    /// Creates a new empty namespace statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of the namespace
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the OVERWRITE clause
    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self.if_not_exists = false; // Mutually exclusive with IF NOT EXISTS
        self
    }

    /// Sets the IF NOT EXISTS clause
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self.overwrite = false; // Mutually exclusive with OVERWRITE
        self
    }

    /// Adds a comment to the namespace definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the namespace definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE NAMESPACE ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name);
        } else {
            bail!("Namespace name is required");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the namespace definition statement on the database
    pub async fn execute(self, ) -> anyhow::Result<Vec<serde_json::Value>> {
        db().execute(self.build()?, vec![]).await
    }
}

impl Display for DefineNamespaceStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_namespace() {
        let stmt = DefineNamespaceStatement::new()
            .name("test")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE NAMESPACE test;");
    }

    #[test]
    fn test_namespace_with_comment() {
        let stmt = DefineNamespaceStatement::new()
            .name("test")
            .comment("Test namespace")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE NAMESPACE test COMMENT \"Test namespace\";");
    }

    #[test]
    fn test_namespace_if_not_exists() {
        let stmt = DefineNamespaceStatement::new()
            .name("test")
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE NAMESPACE IF NOT EXISTS test;");
    }

    #[test]
    fn test_namespace_overwrite() {
        let stmt = DefineNamespaceStatement::new()
            .name("test")
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE NAMESPACE OVERWRITE test;");
    }

    #[test]
    fn test_namespace_overwrite_and_if_not_exists_mutually_exclusive() {
        let stmt = DefineNamespaceStatement::new()
            .name("test")
            .overwrite()
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE NAMESPACE IF NOT EXISTS test;");
    }

    #[test]
    fn test_namespace_without_name() {
        let result = DefineNamespaceStatement::new().build();
        assert!(result.is_err());
    }
}
