//! Not function for SurrealDB queries
//!
//! The not function reverses the truthiness of a value. It is functionally
//! identical to !, the NOT operator.

use std::fmt::{self, Display};

use super::Callable;

/// Not function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum NotFunction {
    /// Reverses the truthiness of a value
    Not(String),
}

impl Display for NotFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Not(val) => write!(f, "not({})", val),
        }
    }
}

impl Callable for NotFunction {
    fn namespace() -> &'static str { "not" }

    fn category(&self) -> &'static str { "logical" }

    fn can_filter(&self) -> bool {
        true // Not function returns boolean and can be used in WHERE
    }
}
