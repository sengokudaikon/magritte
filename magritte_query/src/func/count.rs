//! Count function for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Count function types supported by SurrealDB
///
/// The count function counts the number of times that the function is called.
/// If a value is given as the first argument, then this function checks whether
/// a given value is truthy.
/// If an array is given, this function counts the number of items in the array
/// which are truthy.
#[derive(Debug, Clone)]
pub enum CountFunction {
    /// Simple count, returns 1
    Count,
    /// Count truthy value
    CountValue(String),
    /// Count truthy values in array
    CountArray(String),
}

impl Display for CountFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count => write!(f, "count()"),
            Self::CountValue(val) => write!(f, "count({})", val),
            Self::CountArray(arr) => write!(f, "count({})", arr),
        }
    }
}

impl Callable for CountFunction {
    fn namespace() -> &'static str {
        "count"
    }

    fn category(&self) -> &'static str {
        "aggregation"
    }

    fn can_filter(&self) -> bool {
        false // Count functions return numeric values, not boolean
    }
}
