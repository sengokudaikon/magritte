//! Random functions for SurrealDB queries
//!
//! These functions can be used when generating random data values.

use std::fmt::{self, Display};

use super::Callable;

/// Random function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum RandFunction {
    /// Generates a random float between 0 and 1
    Float,
    /// Generates a random float between min and max
    FloatRange(f64, f64),
    /// Generates a random boolean
    Bool,
    /// Generates a random value from a set of values
    Enum(Vec<String>),
    /// Generates a random guid
    Guid,
    /// Generates a random guid with specific length
    GuidLen(usize),
    /// Generates a random guid with length between min and max
    GuidRange(usize, usize),
    /// Generates a random integer
    Int,
    /// Generates a random integer between min and max
    IntRange(i64, i64),
    /// Generates a random string
    String,
    /// Generates a random string with specific length
    StringLen(usize),
    /// Generates a random string with length between min and max
    StringRange(usize, usize),
    /// Generates a random datetime
    Time,
    /// Generates a random datetime between start and end
    TimeRange(String, String),
    /// Generates a random UUID
    Uuid,
    /// Generates a random UUID from datetime
    UuidFromTime(String),
    /// Generates a random Version 4 UUID
    UuidV4,
    /// Generates a random Version 4 UUID from datetime
    UuidV4FromTime(String),
    /// Generates a random Version 7 UUID
    UuidV7,
    /// Generates a random Version 7 UUID from datetime
    UuidV7FromTime(String),
    /// Generates a random ULID
    Ulid,
    /// Generates a random ULID from datetime
    UlidFromTime(String),
}

impl Display for RandFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float => write!(f, "rand()"),
            Self::FloatRange(min, max) => write!(f, "rand::float({}, {})", min, max),
            Self::Bool => write!(f, "rand::bool()"),
            Self::Enum(values) => write!(f, "rand::enum({})", values.join(", ")),
            Self::Guid => write!(f, "rand::guid()"),
            Self::GuidLen(len) => write!(f, "rand::guid({})", len),
            Self::GuidRange(min, max) => write!(f, "rand::guid({}, {})", min, max),
            Self::Int => write!(f, "rand::int()"),
            Self::IntRange(min, max) => write!(f, "rand::int({}, {})", min, max),
            Self::String => write!(f, "rand::string()"),
            Self::StringLen(len) => write!(f, "rand::string({})", len),
            Self::StringRange(min, max) => write!(f, "rand::string({}, {})", min, max),
            Self::Time => write!(f, "rand::time()"),
            Self::TimeRange(start, end) => write!(f, "rand::time({}, {})", start, end),
            Self::Uuid => write!(f, "rand::uuid()"),
            Self::UuidFromTime(time) => write!(f, "rand::uuid({})", time),
            Self::UuidV4 => write!(f, "rand::uuid::v4()"),
            Self::UuidV4FromTime(time) => write!(f, "rand::uuid::v4({})", time),
            Self::UuidV7 => write!(f, "rand::uuid::v7()"),
            Self::UuidV7FromTime(time) => write!(f, "rand::uuid::v7({})", time),
            Self::Ulid => write!(f, "rand::ulid()"),
            Self::UlidFromTime(time) => write!(f, "rand::ulid({})", time),
        }
    }
}

impl Callable for RandFunction {
    fn namespace() -> &'static str { "rand" }

    fn category(&self) -> &'static str {
        match self {
            Self::Float | Self::FloatRange(..) | Self::Int | Self::IntRange(..) => "numeric",
            Self::Bool => "boolean",
            Self::Enum(..) => "enum",
            Self::Guid | Self::GuidLen(..) | Self::GuidRange(..) => "guid",
            Self::String | Self::StringLen(..) | Self::StringRange(..) => "string",
            Self::Time | Self::TimeRange(..) => "time",
            Self::Uuid
            | Self::UuidFromTime(..)
            | Self::UuidV4
            | Self::UuidV4FromTime(..)
            | Self::UuidV7
            | Self::UuidV7FromTime(..) => "uuid",
            Self::Ulid | Self::UlidFromTime(..) => "ulid",
        }
    }

    fn can_filter(&self) -> bool {
        false // Random functions generate values, not boolean conditions
    }
}
