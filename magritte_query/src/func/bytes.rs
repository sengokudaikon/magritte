//! Bytes functions for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Bytes function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum BytesFunction {
    /// Gives the length in bytes
    Len(String),
}

impl Display for BytesFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Len(val) => write!(f, "bytes::len({})", val),
        }
    }
}

impl Callable for BytesFunction {
    fn namespace() -> &'static str { "bytes" }

    fn category(&self) -> &'static str {
        match self {
            Self::Len(..) => "analysis",
        }
    }

    fn can_filter(&self) -> bool {
        false // Bytes functions return numeric values, not boolean, so can't be
              // used in WHERE
    }
}
