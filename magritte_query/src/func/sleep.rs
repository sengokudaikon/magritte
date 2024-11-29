//! Sleep function for SurrealDB queries
//!
//! This function can be used to introduce a delay or pause in the execution
//! of a query or a batch of queries for a specific amount of time.

use std::fmt::{self, Display};

use super::Callable;

/// Sleep function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum SleepFunction {
    /// Delays or pauses the execution for a specified duration
    Sleep(String), // duration as string (e.g., "1s", "500ms", etc.)
}

impl Display for SleepFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sleep(duration) => write!(f, "sleep({})", duration),
        }
    }
}

impl Callable for SleepFunction {
    fn namespace() -> &'static str { "sleep" }

    fn category(&self) -> &'static str {
        "control" // Sleep is a control flow function
    }

    fn can_filter(&self) -> bool {
        false // Sleep function returns none, not boolean
    }
}
