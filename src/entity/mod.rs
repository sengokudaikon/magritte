use magritte_query::types::{EventType, IndexType, RelationType};
use crate::prelude::{ColumnTrait, EventTrait, IndexTrait, RelationTrait};

pub mod column;
pub mod edge;
pub mod event;
pub mod index;
pub mod relation;
pub mod table;


pub trait HasColumns {
    fn columns() -> impl IntoIterator<Item = impl ColumnTrait> where Self:Sized;
}

pub trait HasEvents {
    fn events() -> impl IntoIterator<Item = impl EventTrait> where Self:Sized;
}

pub trait HasIndexes {
    fn indexes() -> impl IntoIterator<Item = impl IndexTrait> where Self:Sized;
}

pub trait HasRelations {
    fn relations() -> impl IntoIterator<Item = impl RelationTrait> where Self:Sized;
}