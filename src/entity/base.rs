use crate::entity::table::TableTrait;
use crate::prelude::{ColumnTrait, EventTrait, IndexTrait, RelationTrait};

pub trait EntityTrait {
    type Table: TableTrait;
    #[allow(missing_docs)]
    type Columns: ColumnTrait;

    #[allow(missing_docs)]
    type Relations: RelationTrait;

    #[allow(missing_docs)]
    type Indexes: IndexTrait;
    #[allow(missing_docs)]
    type Events: EventTrait;
}
