//! Function definition functionality for SurrealDB.
//!
//! This module provides functionality to define custom functions that can be reused
//! throughout a database. Functions can take arguments, perform complex operations,
//! and return values.
//!
//! See [SurrealDB Function Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/function)
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Define a function that checks if a relation exists
//! let stmt = Define::function()
//!     .name("relation_exists")
//!     .args(vec![
//!         FunctionArg::new("in", "record"),
//!         FunctionArg::new("tb", "string"),
//!         FunctionArg::new("out", "record"),
//!     ])
//!     .query(r#"
//!         LET $results = SELECT VALUE id FROM type::Table($tb) WHERE in = $in AND out = $out;
//!         RETURN array::len($results) > 0;
//!     "#)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root owner/editor, namespace owner/editor, or database owner/editor
//! - Selected namespace and database before using the statement

use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};
use magritte_db::{db, QueryType, SurrealDB};

/// Function argument definition
#[derive(Clone, Debug)]
pub struct FunctionArg {
    /// Name of the argument
    pub name: String,
    /// Type of the argument (e.g., "string", "number", "record", etc.)
    pub type_name: String,
    /// Whether the argument is optional
    pub optional: bool,
}

impl FunctionArg {
    /// Creates a new required function argument
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: false,
        }
    }

    /// Creates a new optional function argument
    pub fn optional(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: true,
        }
    }
}

impl Display for FunctionArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "${}: {}{}",
            self.name,
            if self.optional { "option<" } else { "" },
            self.type_name
        )?;
        if self.optional {
            write!(f, ">")?;
        }
        Ok(())
    }
}

/// Access permissions for functions
#[derive(Clone, Debug)]
pub enum FnPermission {
    /// No access for record users
    None,
    /// Full access for record users
    Full,
    /// Conditional access based on a WHERE clause
    Where(String),
}

impl Display for FnPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FnPermission::None => write!(f, "NONE"),
            FnPermission::Full => write!(f, "FULL"),
            FnPermission::Where(condition) => write!(f, "WHERE {}", condition),
        }
    }
}

/// Statement for defining a custom function in SurrealDB
#[derive(Clone, Debug, Default)]
pub struct DefineFunctionStatement {
    pub(crate) name: Option<String>,
    pub(crate) args: Vec<FunctionArg>,
    pub(crate) query: Option<String>,
    pub(crate) permissions: Option<FnPermission>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineFunctionStatement {
    /// Creates a new empty function definition statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the function name (must be prefixed with "fn::")
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.name = Some(if !name.starts_with("fn::") {
            format!("fn::{}", name)
        } else {
            name
        });
        self
    }

    /// Sets a single function argument
    pub fn arg(mut self, arg: FunctionArg) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets multiple function arguments
    pub fn args(mut self, args: Vec<FunctionArg>) -> Self {
        self.args = args;
        self
    }

    /// Sets the function query/body
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(cleanup(query.into()));
        self
    }

    /// Sets the function permissions
    pub fn permissions(mut self, permissions: FnPermission) -> Self {
        self.permissions = Some(permissions);
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

    /// Adds a comment to the function definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the function definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let name = self.name.as_ref().ok_or_else(|| anyhow!("Function name is required"))?;
        if name.is_empty() {
            bail!("Function name cannot be empty");
        }
        if !name.starts_with("fn::") {
            bail!("Function name must start with 'fn::'");
        }
        if name.len() <= 4 {  // "fn::" is 4 chars
            bail!("Function name must not be empty after 'fn::'");
        }

        let query = self.query.as_ref().ok_or_else(|| anyhow!("Function query/body is required"))?;
        if query.is_empty() {
            bail!("Function query/body cannot be empty");
        }

        let mut stmt = String::new();
        stmt.push_str("DEFINE FUNCTION ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        stmt.push_str(name);

        // Add arguments if any
        if !self.args.is_empty() {
            stmt.push('(');
            stmt.push_str(
                &self
                    .args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            stmt.push(')');
        } else {
            stmt.push_str("()");
        }

        // Add function body
        stmt.push_str(" {\n");
        stmt.push_str(query);
        stmt.push_str("\n}");

        // Add permissions if specified
        if let Some(perms) = &self.permissions {
            stmt.push_str(" PERMISSIONS ");
            stmt.push_str(&perms.to_string());
        }

        // Add comment if specified
        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the function definition statement on the database
    pub async fn execute(self, ) -> anyhow::Result<Vec<serde_json::Value>> {
        db().execute(self.build()?, vec![]).await
    }
}

/// Cleans up function expressions by handling variable references and string escaping
fn cleanup(s: String) -> String {
    s.replace("var:", "$")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .trim()
        .to_string()
}

impl Display for DefineFunctionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function() {
        let stmt = DefineFunctionStatement::new()
            .name("greet")
            .arg(FunctionArg::new("name", "string"))
            .query("RETURN \"Hello, \" + $name + \"!\";")
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE FUNCTION fn::greet($name: string) {\nRETURN \"Hello, \" + $name + \"!\";\n};"
        );
    }

    #[test]
    fn test_function_with_optional_args() {
        let stmt = DefineFunctionStatement::new()
            .name("last_option")
            .args(vec![
                FunctionArg::new("required", "number"),
                FunctionArg::optional("optional", "number"),
            ])
            .query(r#"
                RETURN {
                    required_present: type::is::number($required),
                    optional_present: type::is::number($optional),
                };
            "#)
            .build()
            .unwrap();
        assert!(stmt.contains("$required: number"));
        assert!(stmt.contains("$optional: option<number>"));
    }

    #[test]
    fn test_function_with_permissions() {
        let stmt = DefineFunctionStatement::new()
            .name("fetchAllProducts")
            .query("RETURN (SELECT * FROM product LIMIT 10);")
            .permissions(FnPermission::Where("$auth.admin = true".into()))
            .build()
            .unwrap();
        assert!(stmt.contains("PERMISSIONS WHERE $auth.admin = true"));
    }

    #[test]
    fn test_relation_exists_function() {
        let stmt = DefineFunctionStatement::new()
            .name("relation_exists")
            .args(vec![
                FunctionArg::new("in", "record"),
                FunctionArg::new("tb", "string"),
                FunctionArg::new("out", "record"),
            ])
            .query(r#"
                LET $results = SELECT VALUE id FROM type::Table($tb) WHERE in = $in AND out = $out;
                RETURN array::len($results) > 0;
            "#)
            .build()
            .unwrap();
        assert!(stmt.contains("$in: record"));
        assert!(stmt.contains("$tb: string"));
        assert!(stmt.contains("$out: record"));
        assert!(stmt.contains("SELECT VALUE id FROM type::Table($tb)"));
    }

    #[test]
    fn test_function_with_comment() {
        let stmt = DefineFunctionStatement::new()
            .name("greet")
            .arg(FunctionArg::new("name", "string"))
            .query("RETURN \"Hello, \" + $name + \"!\";")
            .comment("Simple greeting function")
            .build()
            .unwrap();
        assert!(stmt.contains("COMMENT \"Simple greeting function\""));
    }

    #[test]
    fn test_function_with_overwrite() {
        let stmt = DefineFunctionStatement::new()
            .name("greet")
            .arg(FunctionArg::new("name", "string"))
            .query("RETURN \"Hello, \" + $name + \"!\";")
            .overwrite()
            .build()
            .unwrap();
        assert!(stmt.contains("DEFINE FUNCTION OVERWRITE fn::greet"));
    }

    #[test]
    fn test_function_if_not_exists() {
        let stmt = DefineFunctionStatement::new()
            .name("greet")
            .arg(FunctionArg::new("name", "string"))
            .query("RETURN \"Hello, \" + $name + \"!\";")
            .if_not_exists()
            .build()
            .unwrap();
        assert!(stmt.contains("DEFINE FUNCTION IF NOT EXISTS fn::greet"));
    }

    #[test]
    fn test_missing_name() {
        let stmt = DefineFunctionStatement::new()
            .query("RETURN true;")
            .build();
        assert!(stmt.is_err());
    }

    #[test]
    fn test_empty_name() {
        let stmt = DefineFunctionStatement::new()
            .name("")
            .query("RETURN true;")
            .build();
        assert!(stmt.is_err());
    }

    #[test]
    fn test_missing_query() {
        let stmt = DefineFunctionStatement::new()
            .name("test")
            .build();
        assert!(stmt.is_err());
    }

    #[test]
    fn test_empty_query() {
        let stmt = DefineFunctionStatement::new()
            .name("test")
            .query("")
            .build();
        assert!(stmt.is_err());
    }
}
