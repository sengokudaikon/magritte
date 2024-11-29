//! Value functions for SurrealDB queries
//!
//! This module contains several miscellaneous functions that can be used
//! with values of any type.

use std::fmt::{self, Display};

use super::Callable;

/// Value function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum ValueFunction {
    /// Allows an anonymous function to be called on a value
    Chain(String, String), // value, closure
    /// Returns the operation required for one value to equal another
    Diff(String, String), // value1, value2
    /// Applies JSON Patch operations to a value
    Patch(String, String), // value, patch_array
}

impl Display for ValueFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chain(val, closure) => write!(f, "{}.chain({})", val, closure),
            Self::Diff(val1, val2) => write!(f, "{}.diff({})", val1, val2),
            Self::Patch(val, patches) => write!(f, "{}.patch({})", val, patches),
        }
    }
}

impl Callable for ValueFunction {
    fn namespace() -> &'static str { "value" }

    fn category(&self) -> &'static str {
        match self {
            Self::Chain(..) => "functional",
            Self::Diff(..) => "comparison",
            Self::Patch(..) => "modification",
        }
    }

    fn can_filter(&self) -> bool {
        false // Value functions return modified values or patches, not boolean
    }
}
