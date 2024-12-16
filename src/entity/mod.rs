use magritte_macros::{Edge, Event, Index, Relation, Table};
use magritte_query::types::{EventType, IndexType, RelationType};
use magritte_query::{HasId, RecordType, SurrealId};
use serde::{Deserialize, Serialize};

pub mod cache;
pub mod column;
pub mod edge;
pub mod event;
pub mod index;
pub mod manager;
pub mod relation;
pub mod table;


// Re-export main types
pub use cache::EntityCache;
pub use manager::EntityManager;
pub use relation::{LoadStrategy, RelationDef, RelationTrait};

pub trait HasColumns {
    fn columns() -> impl IntoIterator<Item = impl ColumnTrait>
    where
        Self: Sized;

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
    
    fn events() -> Vec<Self::Event>
    where
        Self: Sized,
    {
        vec![]
    }
    fn event_defs() -> Vec<EventDef> {
        vec![]
    }
}

impl<T: RecordType> HasEvents for T {
    default fn events() -> Vec<Self::Event>
    where
        Self: Sized,
    {
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
    
    fn indexes() -> Vec<Self::Index>
    where
        Self: Sized,
    {
        vec![]
    }
    fn index_defs() -> Vec<IndexDef> {
        vec![]
    }
}

impl<T: RecordType> HasIndexes for T {
    default fn indexes() -> Vec<Self::Index>
    where
        Self: Sized,
    {
        vec![]
    }
    default fn index_defs() -> Vec<IndexDef> {
        vec![]
    }
}

#[derive(Relation, Serialize, Deserialize, strum::EnumIter)]
pub enum DummyRelations {
    #[relate(in="", out="", to=Dummy, edge=Dummy)]
    None,
}

pub trait HasRelations {
    type Relation: RelationTrait = DummyRelations;
    
    fn relations() -> Vec<Self::Relation>
    where
        Self: Sized,
    {
        vec![]
    }
    fn relation_defs() -> Vec<RelationDef> {
        vec![]
    }
}

impl<T: RecordType> HasRelations for T {
    default fn relations() -> Vec<Self::Relation>
    where
        Self: Sized,
    {
        vec![]
    }
    default fn relation_defs() -> Vec<RelationDef> {
        vec![]
    }
}

// Re-export common traits
pub use crate::{
    ColumnTrait,
    EventTrait,
    IndexTrait,
    TableTrait,
};
use crate::{ColumnDef, EventDef, IndexDef};

// Export type aliases for common patterns
pub type EntityResult<T> = Result<T, anyhow::Error>;
pub type EntityVec<T> = EntityResult<Vec<T>>;
pub type EntityOption<T> = EntityResult<Option<T>>;
