//! Session functions for SurrealDB queries
//!
//! These functions return information about the current SurrealDB session.

use std::fmt::{self, Display};

use super::Callable;

/// Session function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum SessionFunction {
    /// Returns the current user's access method
    Ac,
    /// Returns the currently selected database
    Db,
    /// Returns the current user's session ID
    Id,
    /// Returns the current user's session IP address
    Ip,
    /// Returns the currently selected namespace
    Ns,
    /// Returns the current user's HTTP origin
    Origin,
    /// Returns the current user's record authentication data
    Rd,
    /// Returns the current user's authentication token
    Token,
}

impl Display for SessionFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ac => write!(f, "session::ac()"),
            Self::Db => write!(f, "session::db()"),
            Self::Id => write!(f, "session::id()"),
            Self::Ip => write!(f, "session::ip()"),
            Self::Ns => write!(f, "session::ns()"),
            Self::Origin => write!(f, "session::origin()"),
            Self::Rd => write!(f, "session::rd()"),
            Self::Token => write!(f, "session::token()"),
        }
    }
}

impl Callable for SessionFunction {
    fn namespace() -> &'static str {
        "session"
    }

    fn category(&self) -> &'static str {
        match self {
            Self::Ac | Self::Token | Self::Rd => "auth",
            Self::Db | Self::Ns => "scope",
            Self::Id | Self::Ip | Self::Origin => "connection",
        }
    }

    fn can_filter(&self) -> bool {
        false // Session functions return session info, not boolean conditions
    }
}
