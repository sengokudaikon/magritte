//! User definition functionality for SurrealDB.
//!
//! This module provides functionality to define system users in SurrealDB.
//!
//! See [SurrealDB User Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/user)
//! for more details.
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//! use std::time::Duration;
//!
//! // Create a basic database user
//! let user = Define::user()
//!     .name("admin")
//!     .on_database()
//!     .password("secure123")
//!     .roles(vec![UserRole::Owner])
//!     .session_duration(Duration::from_secs(3600))
//!     .build()
//!     .unwrap();
//! ```

use crate::database::{QueryType, SurrealDB};
use anyhow::bail;
use std::fmt::Display;
use std::time::Duration;
use tracing::{error, info};

/// Represents the different user roles in SurrealDB
#[derive(Clone, Debug)]
pub enum UserRole {
    Owner,
    Editor,
    Viewer,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Owner => write!(f, "OWNER"),
            UserRole::Editor => write!(f, "EDITOR"),
            UserRole::Viewer => write!(f, "VIEWER"),
        }
    }
}

/// Represents the user level in SurrealDB
#[derive(Clone, Debug)]
pub enum UserLevel {
    Root,
    Namespace,
    Database,
}

impl Display for UserLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserLevel::Root => write!(f, "ROOT"),
            UserLevel::Namespace => write!(f, "NAMESPACE"),
            UserLevel::Database => write!(f, "DATABASE"),
        }
    }
}

/// Statement for defining users in SurrealDB.
///
/// Users can be created at different levels (ROOT, NAMESPACE, DATABASE) with
/// different roles and authentication methods.
///
/// See [DEFINE USER Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/user)
///
/// # Example
///
/// ```rust
/// use magritte_query::define::*;
/// use std::time::Duration;
///
/// // Create a namespace user with session and token durations
/// let user = Define::user()
///     .name("admin")
///     .on_namespace()
///     .password("secure123")
///     .roles(vec![UserRole::Editor])
///     .session_duration(Duration::from_secs(3600))
///     .token_duration(Duration::from_secs(300))
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct DefineUserStatement {
    pub(crate) name: Option<String>,
    pub(crate) level: Option<UserLevel>,
    pub(crate) password: Option<String>,
    pub(crate) passhash: Option<String>,
    pub(crate) roles: Vec<UserRole>,
    pub(crate) session_duration: Option<Duration>,
    pub(crate) token_duration: Option<Duration>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineUserStatement {
    /// Creates a new empty user statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of the user
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the user level to ROOT
    pub fn on_root(mut self) -> Self {
        self.level = Some(UserLevel::Root);
        self
    }

    /// Sets the user level to NAMESPACE
    pub fn on_namespace(mut self) -> Self {
        self.level = Some(UserLevel::Namespace);
        self
    }

    /// Sets the user level to DATABASE
    pub fn on_database(mut self) -> Self {
        self.level = Some(UserLevel::Database);
        self
    }

    /// Sets the user's password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self.passhash = None; // Mutually exclusive with passhash
        self
    }

    /// Sets the user's password hash
    pub fn passhash(mut self, hash: impl Into<String>) -> Self {
        self.passhash = Some(hash.into());
        self.password = None; // Mutually exclusive with password
        self
    }

    /// Sets the user's roles
    pub fn roles(mut self, roles: Vec<UserRole>) -> Self {
        self.roles = roles;
        self
    }

    /// Sets the session duration
    pub fn session_duration(mut self, duration: Duration) -> Self {
        self.session_duration = Some(duration);
        self
    }

    /// Sets the token duration
    pub fn token_duration(mut self, duration: Duration) -> Self {
        self.token_duration = Some(duration);
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

    /// Adds a comment to the user definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs % (24 * 3600) == 0 {
            format!("{}d", secs / (24 * 3600))
        } else if secs % 3600 == 0 {
            format!("{}h", secs / 3600)
        } else if secs % 60 == 0 {
            format!("{}m", secs / 60)
        } else {
            format!("{}s", secs)
        }
    }

    /// Builds the user definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE USER ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name);
        } else {
            bail!("User name is required");
        }

        if let Some(level) = &self.level {
            stmt.push_str(" ON ");
            stmt.push_str(&level.to_string());
        } else {
            bail!("User level is required");
        }

        if let Some(password) = &self.password {
            stmt.push_str(&format!(" PASSWORD '{}'", password));
        } else if let Some(hash) = &self.passhash {
            stmt.push_str(&format!(" PASSHASH '{}'", hash));
        } else {
            bail!("Either password or passhash is required");
        }

        if !self.roles.is_empty() {
            stmt.push_str(" ROLES ");
            stmt.push_str(&self.roles
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", "));
        }

        if self.session_duration.is_some() || self.token_duration.is_some() {
            stmt.push_str(" DURATION");
            
            if let Some(token_duration) = &self.token_duration {
                stmt.push_str(&format!(" FOR TOKEN {}", Self::format_duration(*token_duration)));
                if self.session_duration.is_some() {
                    stmt.push(',');
                }
            }
            
            if let Some(session_duration) = &self.session_duration {
                stmt.push_str(&format!(" FOR SESSION {}", Self::format_duration(*session_duration)));
            }
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the user definition statement on the database
    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema).await
    }
}

impl Display for DefineUserStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_user() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_database()
            .password("secret123")
            .roles(vec![UserRole::Viewer])
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER test_user ON DATABASE PASSWORD 'secret123' ROLES VIEWER;");
    }

    #[test]
    fn test_user_with_passhash() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_root()
            .passhash("hash123")
            .roles(vec![UserRole::Owner])
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER test_user ON ROOT PASSHASH 'hash123' ROLES OWNER;");
    }

    #[test]
    fn test_user_with_durations() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_namespace()
            .password("secret123")
            .roles(vec![UserRole::Editor])
            .session_duration(Duration::from_secs(3600))
            .token_duration(Duration::from_secs(300))
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER test_user ON NAMESPACE PASSWORD 'secret123' ROLES EDITOR DURATION FOR TOKEN 5m, FOR SESSION 1h;");
    }

    #[test]
    fn test_user_if_not_exists() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_database()
            .password("secret123")
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER IF NOT EXISTS test_user ON DATABASE PASSWORD 'secret123';");
    }

    #[test]
    fn test_user_overwrite() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_database()
            .password("secret123")
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER OVERWRITE test_user ON DATABASE PASSWORD 'secret123';");
    }

    #[test]
    fn test_user_with_comment() {
        let stmt = DefineUserStatement::new()
            .name("test_user")
            .on_database()
            .password("secret123")
            .comment("Test user")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE USER test_user ON DATABASE PASSWORD 'secret123' COMMENT \"Test user\";");
    }

    #[test]
    fn test_user_without_name() {
        let result = DefineUserStatement::new()
            .on_database()
            .password("secret123")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_user_without_level() {
        let result = DefineUserStatement::new()
            .name("test_user")
            .password("secret123")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_user_without_auth() {
        let result = DefineUserStatement::new()
            .name("test_user")
            .on_database()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(DefineUserStatement::format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(DefineUserStatement::format_duration(Duration::from_secs(300)), "5m");
        assert_eq!(DefineUserStatement::format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(DefineUserStatement::format_duration(Duration::from_secs(86400)), "1d");
    }
}
