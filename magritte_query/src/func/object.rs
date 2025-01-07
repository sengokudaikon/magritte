//! Object functions for SurrealDB queries
//!
//! These functions can be used when working with, and manipulating data
//! objects.

use std::fmt::{self, Display};

use super::Callable;

/// Object function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum ObjectFunction {
    /// Transforms an object into an array with arrays of key-value combinations
    Entries(String),
    /// Transforms an array with arrays of key-value combinations into an object
    FromEntries(String),
    /// Returns an array with all the keys of an object
    Keys(String),
    /// Returns the amount of key-value pairs an object holds
    Len(String),
    /// Returns an array with all the values of an object
    Values(String),
}

impl Display for ObjectFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entries(obj) => write!(f, "object::entries({})", obj),
            Self::FromEntries(arr) => write!(f, "object::from_entries({})", arr),
            Self::Keys(obj) => write!(f, "object::keys({})", obj),
            Self::Len(obj) => write!(f, "object::len({})", obj),
            Self::Values(obj) => write!(f, "object::values({})", obj),
        }
    }
}

impl Callable for ObjectFunction {
    fn namespace() -> &'static str {
        "object"
    }

    fn category(&self) -> &'static str {
        match self {
            Self::Entries(..) | Self::FromEntries(..) => "conversion",
            Self::Keys(..) | Self::Values(..) => "extraction",
            Self::Len(..) => "analysis",
        }
    }

    fn can_filter(&self) -> bool {
        false // Object functions return arrays or numbers, not boolean
    }
}
