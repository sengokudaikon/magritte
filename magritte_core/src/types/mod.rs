//! Base types used throughout magritte.
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
mod field_type;
pub mod index;
pub mod operator;
pub mod order;
pub mod return_type;

pub mod permission;
pub mod projection;
pub mod range;
pub mod schema;
pub mod value;
pub mod record;
pub mod transaction;


pub use field_type::*;
pub use index::*;
pub use order::*;
pub use permission::*;
pub use projection::*;
pub use range::*;
pub use record::RecordRef;
pub use record::SurrealId;
pub use return_type::*;
pub use schema::SchemaType;


#[cfg(feature = "uuid")]
pub mod uuid;


#[cfg(feature = "uuid")]
pub use uuid::*;

pub trait NamedType {
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

pub trait HasId
where
    Self: RecordType,
{
    fn id(&self) -> SurrealId<Self>;
}
pub trait TableType: RecordType + HasId {
    fn schema_type() -> SchemaType;
}

pub trait ColumnType: ColumnTypeLite {
    fn table_name() -> &'static str;
    fn column_name(&self) -> &str;
    fn column_type(&self) -> &str;
}

pub trait EdgeType: RecordType + HasId {
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

pub trait Relations: Clone + Copy + Send + Sync + IntoEnumIterator + 'static {}

pub trait RelationType: Clone + Send + Sync + 'static {
    fn relation_via() -> String;
    fn relation_from() -> String;
    fn relation_to() -> String;
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

pub type Record = Arc<dyn RecordType>;
