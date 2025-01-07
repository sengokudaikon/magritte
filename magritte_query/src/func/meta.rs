//! Meta functions for SurrealDB queries
//!
//! These functions can be used to retrieve specific metadata from a SurrealDB
//! Record ID.
//!
//! Note: As of version 2.0, these functions are now part of SurrealDB's record
//! functions. They are kept here for backward compatibility.

use std::fmt::{self, Display};

use super::Callable;

/// Meta function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum MetaFunction {
    /// Extracts and returns the Table id from a SurrealDB Record ID
    Id(String),
    /// Extracts and returns the Table name from a SurrealDB Record ID
    Tb(String),
    /// Alias for Tb - extracts and returns the Table name
    Table(String),
}

impl Display for MetaFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(record) => write!(f, "meta::id({})", record),
            Self::Tb(record) => write!(f, "meta::tb({})", record),
            Self::Table(record) => write!(f, "meta::Table({})", record),
        }
    }
}

impl Callable for MetaFunction {
    fn namespace() -> &'static str {
        "meta"
    }

    fn category(&self) -> &'static str {
        "record" // All meta functions deal with record metadata
    }

    fn can_filter(&self) -> bool {
        false // Meta functions return strings, not boolean
    }
}
