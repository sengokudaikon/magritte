//! Base types used throughout magritte_query.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use surrealdb::sql::{Thing, Value};
use surrealdb::{sql, Object, RecordId, RecordIdKey};

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

pub trait NamedType: Sized {
    fn table_name() -> &'static str;
}

pub trait RecordType:
    NamedType
    + Display
    + AsRef<str>
    + Debug
    + Serialize
    + DeserializeOwned
    + Clone
    + Send
    + Sync
    + 'static
{
}

pub trait HasId where Self:RecordType{
    fn id(&self) -> SurrealId<Self>;
}
pub trait TableType: RecordType {
    fn schema_type() -> SchemaType;
}
pub trait ColumnType: ColumnTypeLite {
    fn table_name() -> &'static str;
    fn column_name(&self) -> &str;
    fn column_type(&self) -> &str;
}
pub trait EdgeType: RecordType {
    fn edge_from(&self) -> &str;
    fn edge_to(&self) -> &str;
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
    fn event_name(&self) -> &str;
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
    fn index_name(&self) -> &str;
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
    fn relation_via(&self) -> &str;
    fn relation_from(&self) -> &str;
    fn relation_to(&self) -> &str;
}

pub trait ColumnTypeLite:
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRef<T>(String, PhantomData<T>)
where
    T: RecordType;
impl<T> RecordRef<T>
where
    T: RecordType,
{
    pub fn new() -> Self {
        Self(format!("record<{}>", T::table_name()), PhantomData)
    }
}

impl<T> From<RecordRef<T>> for sql::Value
where
    T: RecordType,
{
    fn from(value: RecordRef<T>) -> Self {
        sql::Value::from(value.0)
    }
}
#[derive(Debug, Clone)]
pub struct SurrealId<T>(RecordId, PhantomData<T>)
where
    T: RecordType;
impl<T> SurrealId<T>
where
    T: RecordType,
{
    pub fn new<I: Into<RecordIdKey>>(id: I) -> Self {
        let record_id = RecordId::from((T::table_name().to_string(), id.into()));
        Self(record_id, PhantomData)
    }

    pub fn to_record_id(&self) -> &RecordId {
        &self.0
    }
    pub fn table(&self) -> &str {
        self.0.table()
    }
    pub fn id(&self) -> &RecordIdKey {
        self.0.key()
    }
}
impl<T> From<String> for SurrealId<T>
where
    T: RecordType,
{
    fn from(value: String) -> Self {
        let record_id = RecordId::from((String::from(T::table_name()), RecordIdKey::from(value)));
        Self(record_id, PhantomData)
    }
}

impl<T> std::fmt::Display for SurrealId<T>
where
    T: RecordType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let table = self.0.table();
        let id = self.0.key().to_string();
        write!(f, "{}:{}", table, id)
    }
}
impl<T> std::ops::Deref for SurrealId<T>
where
    T: RecordType,
{
    type Target = RecordId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq for SurrealId<T>
where
    T: RecordType,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for SurrealId<T> where
    T: RecordType
{
}

impl<T> std::hash::Hash for SurrealId<T>
where
    T: RecordType,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Serialize for SurrealId<T>
where
    T: RecordType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for SurrealId<T>
where
    T: RecordType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let record_id = RecordId::deserialize(deserializer)?;
        Ok(Self(record_id, PhantomData))
    }
}

impl<T> From<serde_json::Value> for SurrealId<T> where T: RecordType {
    fn from(value: serde_json::Value) -> Self {
        SurrealId::new(value.to_string())
    }
}

#[cfg(feature = "uuid")]
pub mod uuid;

pub mod index;

