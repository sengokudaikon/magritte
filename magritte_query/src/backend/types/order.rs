use std::fmt::Debug;

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
