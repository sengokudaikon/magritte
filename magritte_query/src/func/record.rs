//! Record functions for SurrealDB queries
//!
//! These functions can be used to retrieve specific metadata from a SurrealDB
//! Record ID. These functions replace the older meta functions as of SurrealDB
//! 2.0.

use std::fmt::{self, Display};

use super::Callable;

/// Record function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum RecordFunction {
    /// Checks to see if a given record exists
    Exists(String),
    /// Extracts and returns the Table id from a SurrealDB Record ID
    Id(String),
    /// Extracts and returns the Table name from a SurrealDB Record ID
    Tb(String),
    /// Alias for Tb - extracts and returns the Table name
    Table(String),
}

impl Display for RecordFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exists(record) => write!(f, "record::exists({})", record),
            Self::Id(record) => write!(f, "record::id({})", record),
            Self::Tb(record) => write!(f, "record::tb({})", record),
            Self::Table(record) => write!(f, "record::Table({})", record),
        }
    }
}

impl Callable for RecordFunction {
    fn namespace() -> &'static str { "record" }

    fn category(&self) -> &'static str {
        match self {
            Self::Exists(..) => "validation",
            Self::Id(..) | Self::Tb(..) | Self::Table(..) => "metadata",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            Self::Exists(..) // Only exists function returns boolean and can be used in WHERE
        )
    }
}
