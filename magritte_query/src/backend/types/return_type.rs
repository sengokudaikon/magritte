use std::fmt;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Return type for queries
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ReturnType {
    /// Return all fields (default)
    #[default]
    All,
    /// Return no fields
    None,
    /// Return state before changes
    Before,
    /// Return state after changes
    After,
    /// Return difference between states
    Diff,
    /// Return specific fields
    Fields(Vec<String>),
}

impl Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnType::All => write!(f, "AFTER"),
            ReturnType::None => write!(f, "NONE"),
            ReturnType::Before => write!(f, "BEFORE"),
            ReturnType::After => write!(f, "AFTER"),
            ReturnType::Diff => write!(f, "DIFF"),
            ReturnType::Fields(fields) => write!(f, "{}", fields.join(", ")),
        }
    }
}
