//! Base types used throughout magritte_query.

use serde::de::DeserializeOwned;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
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

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    Schemafull,
    Schemaless,
}
impl FromStr for SchemaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SchemaType::from(s.to_string()))
    }
}
impl From<&str> for SchemaType {
    fn from(value: &str) -> Self {
        SchemaType::from(value.to_string())
    }
}
impl From<String> for SchemaType {
    fn from(s: String) -> Self {
        SchemaType::from(&s)
    }
}
impl From<&String> for SchemaType {
    fn from(value: &String) -> Self {
        match value.to_lowercase().as_str() {
            "schemafull" => SchemaType::Schemafull,
            "schemaless" => SchemaType::Schemaless,
            _ => panic!("Invalid schema type"),
        }
    }
}
impl Display for SchemaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SchemaType::Schemafull => write!(f, "SCHEMAFULL"),
            SchemaType::Schemaless => write!(f, "SCHEMALESS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    None,
    Full,
    Select(String),
    Create(String),
    Update(String),
    Delete(String),
}
impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission::from(&value)
    }
}
impl From<&String> for Permission {
    fn from(value: &String) -> Self {
        let value = value.to_lowercase();
        if value.starts_with("for select where ") {
            let condition = value.trim_start_matches("for select where ");
            Permission::Select(condition.to_string())
        } else if value.starts_with("for create where ") {
            let condition = value.trim_start_matches("for create where ");
            Permission::Create(condition.to_string())
        } else if value.starts_with("for update where ") {
            let condition = value.trim_start_matches("for update where ");
            Permission::Update(condition.to_string())
        } else if value.starts_with("for delete where ") {
            let condition = value.trim_start_matches("for delete where ");
            Permission::Delete(condition.to_string())
        } else if value == "none" {
            Permission::None
        } else if value == "full" {
            Permission::Full
        } else {
            panic!("Invalid permission type")
        }
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Permission::None => write!(f, " NONE"),
            Permission::Full => write!(f, " FULL"),
            Permission::Select(v) => write!(f, " FOR select WHERE {}", v),
            Permission::Create(v) => write!(f, " FOR create WHERE {}", v),
            Permission::Update(v) => write!(f, " FOR update WHERE {}", v),
            Permission::Delete(v) => write!(f, " FOR delete WHERE {}", v),
        }
    }
}

pub trait NamedType {
    fn table_name() -> &'static str;
}
pub trait TableType: NamedType +
    Display
    + AsRef<str>
    + Debug
    + Serialize
    + DeserializeOwned
    + Clone
    + Send
    + Sync
    + 'static
{
    fn schema_type() -> SchemaType;
}
pub trait ColumnType:
    FromStr
    + Display
    + AsRef<str>
    + Debug
    + Copy
    + Serialize
    + DeserializeOwned
    + Clone
    + Send
    + Sync
    + strum::IntoEnumIterator
    + 'static
{
    fn table_name() -> &'static str;
    fn column_name(&self) -> & str;
    fn column_type(&self) -> & str;
}
pub trait EdgeType: NamedType +
    Display
    + AsRef<str>
    + Debug
    + Serialize
    + DeserializeOwned
    + Clone
    + Send
    + Sync
    + 'static
{
    fn edge_from(&self) -> & str;
    fn edge_to(&self) -> & str;
    fn is_enforced(&self) -> bool;
}

pub trait EventType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Debug
    + Send
    + Sync
    + Copy
    + strum::IntoEnumIterator
    + 'static
{
    fn event_name(&self) -> & str;
    fn table_name() -> &'static str;
}

pub trait IndexType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Send
    + Sync
    + Debug
    + Copy
    + strum::IntoEnumIterator
    + 'static
{
    fn index_name(&self) -> & str;
    fn table_name() -> &'static str;
}

pub trait RelationType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Send
    + Sync
    + Debug
    + Copy
    + strum::IntoEnumIterator
    + 'static
{
    fn relation_via(&self) -> & str;
    fn relation_from(&self) -> & str;
    fn relation_to(&self) -> &str;
}
