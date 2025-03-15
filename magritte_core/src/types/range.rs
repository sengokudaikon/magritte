use std::fmt;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum RangeTarget {
    Count(String),         // person:3
    Full,                  // person:1..1000
    FullInclusive,         // person:1..=1000
    From(String),          // person:1..
    To(String),            // person:..1000
    ToInclusive(String),   // person:..=1000
    Range(String, String), // person:1..5000
}

impl Display for RangeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count(count) => write!(f, "{}", count),
            Self::Full => write!(f, ".."),
            Self::FullInclusive => write!(f, "..="),
            Self::From(start) => write!(f, "{}...", start),
            Self::To(end) => write!(f, "..{}", end),
            Self::ToInclusive(end) => write!(f, "..={}", end),
            Self::Range(start, end) => write!(f, "{}..{}", start, end),
        }
    }
}
