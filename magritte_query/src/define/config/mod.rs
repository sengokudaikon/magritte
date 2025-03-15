//! Configuration definition functionality for SurrealDB.
//!
//! This module provides functionality to define GraphQL configuration in SurrealDB,
//! allowing you to specify how tables and functions are exposed via the GraphQL API.
//!
//! See [SurrealDB Config Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/config)
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Define GraphQL configuration with specific tables and functions
//! let config = Define::config()
//!     .tables(TableConfig::Include(vec!["user".into(), "post".into()]))
//!     .functions(FunctionConfig::Auto)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root, namespace, or database user
//! - Selected namespace and database
//! - At least one table defined in the database
//! - SurrealDB instance started with `SURREAL_EXPERIMENTAL_GRAPHQL=true`

use magritte_db::{db, QueryType, SurrealDB};
use anyhow::anyhow;
use std::fmt::Display;
use tracing::{error, info};

/// Configuration options for tables in GraphQL
#[derive(Clone, Debug)]
pub enum TableConfig {
    /// Automatically include all tables
    Auto,
    /// Do not include any tables
    None,
    /// Include specific tables
    Include(Vec<String>),
}

impl Display for TableConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableConfig::Auto => write!(f, "AUTO"),
            TableConfig::None => write!(f, "NONE"),
            TableConfig::Include(tables) => {
                write!(f, "INCLUDE {}", tables.join(", "))
            }
        }
    }
}

/// Configuration options for functions in GraphQL
#[derive(Clone, Debug)]
pub enum FunctionConfig {
    /// Automatically include all functions
    Auto,
    /// Do not include any functions
    None,
    /// Include specific functions
    Include(Vec<String>),
    /// Exclude specific functions
    Exclude(Vec<String>),
}

impl Display for FunctionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionConfig::Auto => write!(f, "AUTO"),
            FunctionConfig::None => write!(f, "NONE"),
            FunctionConfig::Include(funcs) => {
                write!(f, "INCLUDE [{}]", funcs.join(", "))
            }
            FunctionConfig::Exclude(funcs) => {
                write!(f, "EXCLUDE [{}]", funcs.join(", "))
            }
        }
    }
}

/// Statement for defining GraphQL configuration in SurrealDB
#[derive(Clone, Debug, Default)]
pub struct DefineConfigStatement {
    pub(crate) tables: Option<TableConfig>,
    pub(crate) functions: Option<FunctionConfig>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
}

impl DefineConfigStatement {
    /// Creates a new empty GraphQL configuration statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the table configuration
    pub fn tables(mut self, config: TableConfig) -> Self {
        self.tables = Some(config);
        self
    }

    /// Sets the function configuration
    pub fn functions(mut self, config: FunctionConfig) -> Self {
        self.functions = Some(config);
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

    /// Builds the GraphQL configuration SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE CONFIG ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        stmt.push_str("GRAPHQL ");

        if let Some(tables) = &self.tables {
            stmt.push_str(&format!("TABLES {}", tables));
        }

        if let Some(functions) = &self.functions {
            if self.tables.is_some() {
                stmt.push(' ');
            }
            stmt.push_str(&format!("FUNCTIONS {}", functions));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the GraphQL configuration statement on the database
    pub async fn execute(self, ) -> anyhow::Result<Vec<serde_json::Value>> {
        db().execute(self.build()?, vec![]).await
    }
}

impl Display for DefineConfigStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_config() {
        let stmt = DefineConfigStatement::new()
            .tables(TableConfig::Auto)
            .functions(FunctionConfig::Auto)
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE CONFIG GRAPHQL TABLES AUTO FUNCTIONS AUTO;");
    }

    #[test]
    fn test_include_tables() {
        let stmt = DefineConfigStatement::new()
            .tables(TableConfig::Include(vec!["user".into(), "post".into()]))
            .functions(FunctionConfig::None)
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE CONFIG GRAPHQL TABLES INCLUDE user, post FUNCTIONS NONE;"
        );
    }

    #[test]
    fn test_exclude_functions() {
        let stmt = DefineConfigStatement::new()
            .tables(TableConfig::Auto)
            .functions(FunctionConfig::Exclude(vec![
                "debugFunction".into(),
                "testFunction".into(),
            ]))
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE CONFIG GRAPHQL TABLES AUTO FUNCTIONS EXCLUDE [debugFunction, testFunction];"
        );
    }

    #[test]
    fn test_config_with_overwrite() {
        let stmt = DefineConfigStatement::new()
            .tables(TableConfig::Auto)
            .functions(FunctionConfig::Auto)
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE CONFIG OVERWRITE GRAPHQL TABLES AUTO FUNCTIONS AUTO;"
        );
    }

    #[test]
    fn test_config_if_not_exists() {
        let stmt = DefineConfigStatement::new()
            .tables(TableConfig::Auto)
            .functions(FunctionConfig::Auto)
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE CONFIG IF NOT EXISTS GRAPHQL TABLES AUTO FUNCTIONS AUTO;"
        );
    }
}
