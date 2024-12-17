use magritte_macros::{Edge, Event, Index, Relation, Table};
use magritte_query::types::{EventType, IndexType, RelationType};
use magritte_query::{HasId, RecordType, Relations, SurrealId};
use serde::{Deserialize, Serialize};

pub mod column;
pub mod edge;
pub mod event;
pub mod index;
pub mod manager;
pub mod relation;
pub mod table;

// Re-export main types
pub use manager::{cache::EntityCache, EntityManager};
pub use relation::{LoadStrategy, RelationDef, RelationTrait};

pub trait HasColumns {
    fn columns() -> Vec<impl ColumnTrait>;

    fn column_defs() -> Vec<ColumnDef>;
}

#[derive(Table, Clone, Serialize, Deserialize)]
pub struct Dummy {
    id: SurrealId<Self>,
}

impl HasId for Dummy {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum DummyEvents {
    #[event(when = "Never", then = "Never")]
    None,
}

pub trait HasEvents {
    type Event: EventTrait = DummyEvents;

    fn events() -> Vec<Self::Event>;
    fn event_defs() -> Vec<EventDef>;
}

impl<T: RecordType> HasEvents for T {
    default fn events() -> Vec<Self::Event> {
        vec![]
    }
    default fn event_defs() -> Vec<EventDef> {
        vec![]
    }
}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum DummyIndexes {
    None,
}

pub trait HasIndexes {
    type Index: IndexTrait = DummyIndexes;

    fn indexes() -> Vec<Self::Index>;
    fn index_defs() -> Vec<IndexDef>;
}

impl<T: RecordType> HasIndexes for T {
    default fn indexes() -> Vec<Self::Index> {
        vec![]
    }

    default fn index_defs() -> Vec<IndexDef> {
        vec![]
    }
}
#[derive(Edge, Serialize, Deserialize, Clone)]
#[edge(from=Dummy, to=Dummy)]
pub struct DummyEdge {
    id: SurrealId<Self>,
}
impl HasId for DummyEdge {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}
#[derive(Relation, Serialize, Deserialize, strum::EnumIter)]
pub enum DummyRelations {
    #[relate(to=Dummy, edge=DummyEdge)]
    None,
}

pub trait HasRelations {
    type Relation: Relations = DummyRelations;

    fn relations() -> Vec<Self::Relation>;
    fn relation_defs() -> Vec<RelationDef>;
}

impl<T: RecordType> HasRelations for T {
    default fn relations() -> Vec<Self::Relation> {
        vec![]
    }
    default fn relation_defs() -> Vec<RelationDef> {
        vec![]
    }
}

// Re-export common traits
use crate::{ColumnDef, EventDef, IndexDef};
pub use crate::{ColumnTrait, EventTrait, IndexTrait, TableTrait};

// Export type aliases for common patterns
pub type EntityResult<T> = Result<T, anyhow::Error>;
pub type EntityVec<T> = EntityResult<Vec<T>>;
pub type EntityOption<T> = EntityResult<Option<T>>;
