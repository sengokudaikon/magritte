//! Token definition functionality for SurrealDB.
//!
//! This module provides functionality to define tokens for authentication with
//! third-party providers using JWT (JSON Web Tokens).
//!
//! See [SurrealDB Token Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/token)
//! for more details.
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Create a basic token for database authentication
//! let token = Define::token()
//!     .name("auth_token")
//!     .on_database()
//!     .token_type(TokenType::HS512)
//!     .value("your-secret-key")
//!     .build()
//!     .unwrap();
//! ```

use crate::database::{QueryType, SurrealDB};
use anyhow::bail;
use std::fmt::Display;
use tracing::{error, info};

/// Represents the different token verification types supported by SurrealDB
#[derive(Clone, Debug)]
pub enum TokenType {
    /// HMAC algorithms
    HS256,
    HS384,
    HS512,
    /// Public-key cryptography algorithms
    EDDSA,
    ES256,
    ES384,
    ES512,
    PS256,
    PS384,
    PS512,
    RS256,
    RS384,
    RS512,
    /// JSON Web Key Set
    JWKS,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::HS256 => write!(f, "HS256"),
            TokenType::HS384 => write!(f, "HS384"),
            TokenType::HS512 => write!(f, "HS512"),
            TokenType::EDDSA => write!(f, "EDDSA"),
            TokenType::ES256 => write!(f, "ES256"),
            TokenType::ES384 => write!(f, "ES384"),
            TokenType::ES512 => write!(f, "ES512"),
            TokenType::PS256 => write!(f, "PS256"),
            TokenType::PS384 => write!(f, "PS384"),
            TokenType::PS512 => write!(f, "PS512"),
            TokenType::RS256 => write!(f, "RS256"),
            TokenType::RS384 => write!(f, "RS384"),
            TokenType::RS512 => write!(f, "RS512"),
            TokenType::JWKS => write!(f, "JWKS"),
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::HS256
    }
}

/// Represents the token scope level
#[derive(Clone, Debug)]
pub enum TokenScope {
    Namespace,
    Database,
    Scope(String),
}

impl Display for TokenScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenScope::Namespace => write!(f, "NAMESPACE"),
            TokenScope::Database => write!(f, "DATABASE"),
            TokenScope::Scope(scope) => write!(f, "SCOPE {}", scope),
        }
    }
}

/// Statement for defining tokens in SurrealDB.
///
/// Tokens are used for authentication with third-party providers using JWT.
///
/// See [DEFINE TOKEN Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/token)
///
/// # Example
///
/// ```rust
/// use magritte_query::define::*;
///
/// // Create a token with JWKS verification
/// let token = Define::token()
///     .name("oauth_token")
///     .on_scope("users")
///     .token_type(TokenType::JWKS)
///     .value("https://example.com/.well-known/jwks.json")
///     .comment("OAuth provider JWKS endpoint")
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct DefineTokenStatement {
    pub(crate) name: Option<String>,
    pub(crate) scope: Option<TokenScope>,
    pub(crate) token_type: TokenType,
    pub(crate) value: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineTokenStatement {
    /// Creates a new empty token statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of the token
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the token to be used for namespace-level authentication
    pub fn on_namespace(mut self) -> Self {
        self.scope = Some(TokenScope::Namespace);
        self
    }

    /// Sets the token to be used for database-level authentication
    pub fn on_database(mut self) -> Self {
        self.scope = Some(TokenScope::Database);
        self
    }

    /// Sets the token to be used for scope-level authentication
    pub fn on_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(TokenScope::Scope(scope.into()));
        self
    }

    /// Sets the token verification type
    pub fn token_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type;
        self
    }

    /// Sets the token value (secret key, public key, or JWKS URL)
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
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

    /// Adds a comment to the token definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the token definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE TOKEN ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name);
        } else {
            bail!("Token name is required");
        }

        if let Some(scope) = &self.scope {
            stmt.push_str(" ON ");
            stmt.push_str(&scope.to_string());
        } else {
            bail!("Token scope is required");
        }

        stmt.push_str(" TYPE ");
        stmt.push_str(&self.token_type.to_string());

        if let Some(value) = &self.value {
            stmt.push_str(" VALUE ");
            // Handle multi-line values (like public keys) properly
            if value.contains('\n') {
                stmt.push_str(&format!("\"{}\"", value.replace('\n', "\\n")));
            } else {
                stmt.push_str(&format!("\"{}\"", value));
            }
        } else {
            bail!("Token value is required");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the token definition statement on the database
    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema).await
    }
}

impl Display for DefineTokenStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_token() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_database()
            .token_type(TokenType::HS512)
            .value("secret-key")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN test_token ON DATABASE TYPE HS512 VALUE \"secret-key\";");
    }

    #[test]
    fn test_token_with_comment() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_namespace()
            .token_type(TokenType::RS256)
            .value("public-key")
            .comment("Test token")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN test_token ON NAMESPACE TYPE RS256 VALUE \"public-key\" COMMENT \"Test token\";");
    }

    #[test]
    fn test_token_with_scope() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_scope("users")
            .token_type(TokenType::JWKS)
            .value("https://example.com/.well-known/jwks.json")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN test_token ON SCOPE users TYPE JWKS VALUE \"https://example.com/.well-known/jwks.json\";");
    }

    #[test]
    fn test_token_if_not_exists() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_database()
            .token_type(TokenType::HS256)
            .value("secret")
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN IF NOT EXISTS test_token ON DATABASE TYPE HS256 VALUE \"secret\";");
    }

    #[test]
    fn test_token_overwrite() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_database()
            .token_type(TokenType::HS256)
            .value("secret")
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN OVERWRITE test_token ON DATABASE TYPE HS256 VALUE \"secret\";");
    }

    #[test]
    fn test_token_with_multiline_value() {
        let stmt = DefineTokenStatement::new()
            .name("test_token")
            .on_database()
            .token_type(TokenType::RS256)
            .value("-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkq\nABCDEF==\n-----END PUBLIC KEY-----")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE TOKEN test_token ON DATABASE TYPE RS256 VALUE \"-----BEGIN PUBLIC KEY-----\\nMIIBIjANBgkq\\nABCDEF==\\n-----END PUBLIC KEY-----\";");
    }

    #[test]
    fn test_token_without_name() {
        let result = DefineTokenStatement::new()
            .on_database()
            .token_type(TokenType::HS256)
            .value("secret")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_token_without_scope() {
        let result = DefineTokenStatement::new()
            .name("test_token")
            .token_type(TokenType::HS256)
            .value("secret")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_token_without_value() {
        let result = DefineTokenStatement::new()
            .name("test_token")
            .on_database()
            .token_type(TokenType::HS256)
            .build();
        assert!(result.is_err());
    }
}
