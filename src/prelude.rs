pub use magritte_macros::{
    Edge, Event, Index, Relation, Column, Table
};
pub use super::entity::column::ColumnTrait;
pub use super::entity::edge::EdgeTrait;
pub use super::entity::event::EventTrait;
pub use super::entity::index::IndexTrait;
pub use super::entity::relation::RelationTrait;
pub use super::entity::table::TableTrait;
pub use magritte_query;
pub use magritte_macros::EnumIter;
pub use strum;
// Re-exports for convenience
pub use magritte_macros::*;
pub use surrealdb::RecordId;
pub use magritte_query::types::*;
pub use magritte_query::types::index::*;
pub use super::defs::*;
pub use super::ColumnFromStrErr;
pub use super::TableFromStrErr;
pub use super::EventFromStrErr;
pub use super::IndexFromStrErr;
pub use super::RelationFromStrErr;
pub use super::EdgeFromStrErr;


#[cfg(feature = "uuid")]
pub use magritte_query::types::uuid::*;
