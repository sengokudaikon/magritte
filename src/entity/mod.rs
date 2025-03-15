use inventory::collect;
use magritte_macros::{Edge, Event, Index, Relation, Table};
use magritte_query::types::{EventType, IndexType, RelationType};
use magritte_query::{HasId, RecordType, Relations, SurrealId};
use serde::{Deserialize, Serialize};

pub mod column;
pub mod edge;
pub mod event;
pub mod index;
pub mod relation;
pub mod table;
pub mod cache;

pub use relation::{LoadStrategy, RelationDef, RelationTrait};

pub trait HasColumns {
    fn columns() -> Vec<impl ColumnTrait>;

    fn column_defs() -> Vec<ColumnDef>;
}

pub trait HasEvents {

    fn events() -> Vec<impl EventTrait>;
    fn event_defs() -> Vec<EventDef>;
}

pub trait HasIndexes {
    fn indexes() -> Vec<impl IndexTrait>;
    fn index_defs() -> Vec<IndexDef>;
}
pub trait HasRelations {

    fn relations() -> Vec<impl Relations>;
    fn relation_defs() -> Vec<RelationDef>;
}

// Re-export common traits
use crate::{ColumnDef, EventDef, EventRegistration, IndexDef, IndexRegistration};
pub use crate::{ColumnTrait, EventTrait, IndexTrait, TableTrait};

// Export type aliases for common patterns
pub type EntityResult<T> = Result<T, anyhow::Error>;
pub type EntityVec<T> = EntityResult<Vec<T>>;
pub type EntityOption<T> = EntityResult<Option<T>>;
