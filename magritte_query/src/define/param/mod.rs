//! Parameter definition functionality for SurrealDB.
//!
//! This module provides functionality to define global (database-wide) parameters
//! that are available to every client.
//!
//! See [SurrealDB Param Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/param)
//! for more details.
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Create a basic parameter
//! let param = Define::param()
//!     .name("$endpointBase")
//!     .value("https://dummyjson.com")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root/namespace/database owner or editor
//! - Namespace and database must be selected

use crate::database::{QueryType, SurrealDB};
use anyhow::bail;
use serde::Serialize;
use std::fmt::Display;
use tracing::{error, info};

/// Represents the permission level for a parameter
#[derive(Clone, Debug)]
pub enum ParamPermission {
    None,
    Full,
    Where(String),
}

impl Display for ParamPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamPermission::None => write!(f, "NONE"),
            ParamPermission::Full => write!(f, "FULL"),
            ParamPermission::Where(condition) => write!(f, "WHERE {}", condition),
        }
    }
}

/// Statement for defining parameters in SurrealDB.
///
/// Parameters are global (database-wide) values that are available to every client.
///
/// See [DEFINE PARAM Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/param)
///
/// # Example
///
/// ```rust
/// use magritte_query::define::*;
///
/// // Create a parameter with permissions
/// let param = Define::param()
///     .name("$apiKey")
///     .value("secret-key-123")
///     .permissions(ParamPermission::None)
///     .comment("API key for external service")
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct DefineParamStatement {
    pub(crate) name: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
    pub(crate) permissions: Option<ParamPermission>,
}

impl DefineParamStatement {
    /// Creates a new empty parameter statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of the parameter (must start with $)
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.name = Some(if name.starts_with('$') {
            name
        } else {
            format!("${}", name)
        });
        self
    }

    /// Sets the value of the parameter
    pub fn value<T: Serialize>(mut self, value: T) -> Self {
        self.value = Some(serde_json::to_string(&value).unwrap_or_default());
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

    /// Adds a comment to the parameter definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Sets the permissions for the parameter
    pub fn permissions(mut self, permissions: ParamPermission) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Builds the parameter definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE PARAM ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name);
        } else {
            bail!("Parameter name is required");
        }

        if let Some(value) = &self.value {
            stmt.push_str(" VALUE ");
            stmt.push_str(value);
        } else {
            bail!("Parameter value is required");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        if let Some(permissions) = &self.permissions {
            stmt.push_str(" PERMISSIONS ");
            stmt.push_str(&permissions.to_string());
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the parameter definition statement on the database
    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema).await
    }
}

impl Display for DefineParamStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_param() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value(123)
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM $test VALUE 123;");
    }

    #[test]
    fn test_param_with_comment() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value("hello")
            .comment("Test parameter")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM $test VALUE \"hello\" COMMENT \"Test parameter\";");
    }

    #[test]
    fn test_param_with_permissions() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value(true)
            .permissions(ParamPermission::None)
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM $test VALUE true PERMISSIONS NONE;");
    }

    #[test]
    fn test_param_with_where_permission() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value(42)
            .permissions(ParamPermission::Where("user = 'admin'".to_string()))
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM $test VALUE 42 PERMISSIONS WHERE user = 'admin';");
    }

    #[test]
    fn test_param_if_not_exists() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value(123)
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM IF NOT EXISTS $test VALUE 123;");
    }

    #[test]
    fn test_param_overwrite() {
        let stmt = DefineParamStatement::new()
            .name("$test")
            .value(123)
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM OVERWRITE $test VALUE 123;");
    }

    #[test]
    fn test_param_without_name() {
        let result = DefineParamStatement::new()
            .value(123)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_param_without_value() {
        let result = DefineParamStatement::new()
            .name("$test")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_param_auto_prefix_dollar() {
        let stmt = DefineParamStatement::new()
            .name("test")
            .value(123)
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE PARAM $test VALUE 123;");
    }
}
