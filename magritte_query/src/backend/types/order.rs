use std::fmt::{Debug, Display};
use std::str::FromStr;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Order by options for query results
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum OrderBy {
    /// Regular field ordering
    Field(String),
    /// Random ordering using RAND()
    Random,
    /// Unicode collation for text
    Collate(String),
    /// Numeric ordering for text containing numbers
    Numeric(String),
}
