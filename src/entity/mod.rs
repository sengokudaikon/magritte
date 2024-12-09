use crate::{ColumnTrait, EventTrait, IndexDef, IndexTrait, RelationTrait, TableTrait};
use magritte_macros::{Edge, Event, Index, Relation, Table};
use magritte_query::types::{EventType, IndexType, RelationType};
use magritte_query::{HasId, RecordType, SurrealId};
use serde::{Deserialize, Serialize};
use tracing::Event;

pub mod column;
pub mod edge;
pub mod event;
pub mod index;
pub mod relation;
pub mod table;
pub trait HasColumns {
    fn columns() -> impl IntoIterator<Item = impl ColumnTrait>
    where
        Self: Sized;
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
}

impl<T: RecordType> HasEvents for T {
    default fn events() -> Vec<Self::Event>
    where
        Self: Sized,
    {
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
}

impl<T: RecordType> HasIndexes for T {
    default fn indexes() -> Vec<Self::Index>
    where
        Self: Sized,
    {
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
}

impl<T: RecordType> HasRelations for T {
    default fn relations() -> Vec<Self::Relation>
    where
        Self: Sized,
    {
        vec![]
    }
}
