//! Access control definition functionality for SurrealDB.
//!
//! This module provides functionality to define access control rules in SurrealDB,
//! including both JWT token verification and record-level access control.
//!
//! See [SurrealDB Access Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/access)
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::access::{DefineAccessStatement, AccessRule, Scope};
//!
//! // Define JWT access rule
//! let jwt_access = DefineAccessStatement::new() // or Define::access()
//!     .scope(Scope::Jwt)
//!     .rule(AccessRule::RS256 {
//!         issuer: "auth.example.com".into(),
//!         key_id: "default".into(),
//!     })
//!     .build()
//!     .unwrap();
//!
//! // Define record-level access rule
//! let record_access = DefineAccessStatement::new()
//!     .scope(Scope::Record("user".into()))
//!     .rule(AccessRule::Expression("$auth.id = id".into()))
//!     .build()
//!     .unwrap();
//! ```

use crate::SurrealDB;
use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};

/// Supported signing algorithms for JWT verification
#[derive(Clone, Debug)]
pub enum Algorithm {
    /// HMAC using SHA-256
    HS256,
    /// HMAC using SHA-384
    HS384,
    /// HMAC using SHA-512
    HS512,
    /// RSASSA-PKCS1-v1_5 using SHA-256
    RS256,
    /// RSASSA-PKCS1-v1_5 using SHA-384
    RS384,
    /// RSASSA-PKCS1-v1_5 using SHA-512
    RS512,
    /// ECDSA using P-256 and SHA-256
    ES256,
    /// ECDSA using P-384 and SHA-384
    ES384,
    /// ECDSA using P-521 and SHA-512
    ES512,
    /// RSASSA-PSS using SHA-256
    PS256,
    /// RSASSA-PSS using SHA-384
    PS384,
    /// RSASSA-PSS using SHA-512
    PS512,
    /// EdDSA signature algorithms
    EdDSA,
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::HS256 => write!(f, "HS256"),
            Algorithm::HS384 => write!(f, "HS384"),
            Algorithm::HS512 => write!(f, "HS512"),
            Algorithm::RS256 => write!(f, "RS256"),
            Algorithm::RS384 => write!(f, "RS384"),
            Algorithm::RS512 => write!(f, "RS512"),
            Algorithm::ES256 => write!(f, "ES256"),
            Algorithm::ES384 => write!(f, "ES384"),
            Algorithm::ES512 => write!(f, "ES512"),
            Algorithm::PS256 => write!(f, "PS256"),
            Algorithm::PS384 => write!(f, "PS384"),
            Algorithm::PS512 => write!(f, "PS512"),
            Algorithm::EdDSA => write!(f, "EdDSA"),
        }
    }
}

/// Scope for access control rules
#[derive(Clone, Debug)]
pub enum Scope {
    /// JWT token verification rules
    Jwt,
    /// Record-level access control rules
    Record(String),
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Jwt => write!(f, "jwt"),
            Scope::Record(table) => write!(f, "record {}", table),
        }
    }
}

/// Access control rules for JWT verification or record-level access
#[derive(Clone, Debug)]
pub enum AccessRule {
    /// HMAC-based JWT verification
    HMAC {
        /// The algorithm to use (HS256, HS384, or HS512)
        algorithm: Algorithm,
        /// The secret key for verification
        key: String,
        /// Optional issuer to validate
        issuer: Option<String>,
    },
    /// RSA-based JWT verification
    RS256 {
        /// The issuer to validate
        issuer: String,
        /// The key ID to use
        key_id: String,
    },
    /// Record-level access expression
    Expression(String),
}

impl Display for AccessRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessRule::HMAC {
                algorithm,
                key,
                issuer,
            } => {
                write!(f, "SIGNIN WITH {} KEY '{}'", algorithm, key)?;
                if let Some(iss) = issuer {
                    write!(f, " ISSUER '{}'", iss)?;
                }
                Ok(())
            }
            AccessRule::RS256 { issuer, key_id } => {
                write!(
                    f,
                    "SIGNIN VERIFY RS256 KEY '{}' ISSUER '{}'",
                    key_id, issuer
                )
            }
            AccessRule::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

/// Statement for defining access control rules in SurrealDB
#[derive(Clone, Debug, Default)]
pub struct DefineAccessStatement {
    pub(crate) scope: Option<Scope>,
    pub(crate) rule: Option<AccessRule>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineAccessStatement {
    /// Creates a new empty access control statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the scope for the access control rule
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Sets the access control rule
    pub fn rule(mut self, rule: AccessRule) -> Self {
        self.rule = Some(rule);
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

    /// Adds a comment to the access control definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the access control definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE ");

        if let Some(scope) = &self.scope {
            stmt.push_str(&format!("ACCESS {}", scope));
        } else {
            bail!("Access scope is required");
        }

        if self.if_not_exists {
            stmt.push_str(" IF NOT EXISTS");
        } else if self.overwrite {
            stmt.push_str(" OVERWRITE");
        }

        if let Some(rule) = &self.rule {
            stmt.push(' ');
            stmt.push_str(&rule.to_string());
        } else {
            bail!("Access rule is required");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the access control definition statement on the database
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

impl Display for DefineAccessStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_access_hs256() {
        let stmt = DefineAccessStatement::new()
            .scope(Scope::Jwt)
            .rule(AccessRule::HMAC {
                algorithm: Algorithm::HS256,
                key: "secret".into(),
                issuer: Some("auth.example.com".into()),
            })
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE ACCESS jwt SIGNIN WITH HS256 KEY 'secret' ISSUER 'auth.example.com';"
        );
    }

    #[test]
    fn test_jwt_access_rs256() {
        let stmt = DefineAccessStatement::new()
            .scope(Scope::Jwt)
            .rule(AccessRule::RS256 {
                issuer: "auth.example.com".into(),
                key_id: "default".into(),
            })
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE ACCESS jwt SIGNIN VERIFY RS256 KEY 'default' ISSUER 'auth.example.com';"
        );
    }

    #[test]
    fn test_record_access() {
        let stmt = DefineAccessStatement::new()
            .scope(Scope::Record("user".into()))
            .rule(AccessRule::Expression("$auth.id = id".into()))
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE ACCESS record user $auth.id = id;");
    }

    #[test]
    fn test_access_with_comment() {
        let stmt = DefineAccessStatement::new()
            .scope(Scope::Record("user".into()))
            .rule(AccessRule::Expression("$auth.id = id".into()))
            .comment("Only allow access to own records")
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE ACCESS record user $auth.id = id COMMENT \"Only allow access to own records\";"
        );
    }

    #[test]
    fn test_access_with_overwrite() {
        let stmt = DefineAccessStatement::new()
            .scope(Scope::Record("user".into()))
            .rule(AccessRule::Expression("$auth.id = id".into()))
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE ACCESS record user OVERWRITE $auth.id = id;");
    }
}
