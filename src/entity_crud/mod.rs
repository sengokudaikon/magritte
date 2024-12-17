use anyhow::Result;
use magritte_query::{
    DeleteStatement, HasId, InsertStatement, Query, RecordType, SelectStatement, SurrealId,
    UpdateStatement, UpsertStatement,
};
use std::fmt::{Debug, Display};

pub trait BasicCrud<T>
where
    T: RecordType + HasId,
{
    fn insert_by_id(id: SurrealId<T>, entity: T) -> Result<InsertStatement<T>> {
        Query::insert()
            .where_id(id)
            .content(&entity)
            .map_err(anyhow::Error::from)
    }
    fn find_by_id(id: SurrealId<T>) -> Result<SelectStatement<T>> {
        Ok(Query::select().where_id(id))
    }

    fn upsert_by_id(id: SurrealId<T>, entity: T) -> Result<UpsertStatement<T>> {
        Query::upsert()
            .where_id(id)
            .content(&entity)
            .map_err(anyhow::Error::from)
    }

    fn find_all() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select())
    }
    fn count() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select().count())
    }

    fn update_by_id(id: SurrealId<T>, entity: T) -> Result<UpdateStatement<T>> {
        Query::update()
            .where_id(id)
            .content(&entity)
            .map_err(anyhow::Error::from)
    }

    fn delete_by_id(id: SurrealId<T>) -> Result<DeleteStatement<T>> {
        Ok(Query::delete().where_id(id))
    }
    fn delete_all() -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }
}

impl<T> BasicCrud<T> for T where T: HasId + RecordType + Sized {}
pub trait SurrealCrud<T>
where
    T: HasId + RecordType,
{
    fn insert(&self) -> anyhow::Result<InsertStatement<T>>;
    fn find(&self) -> anyhow::Result<SelectStatement<T>>;
    fn upsert(&self) -> anyhow::Result<UpsertStatement<T>>;
    fn update(&self) -> anyhow::Result<UpdateStatement<T>>;
    fn delete(&self) -> anyhow::Result<DeleteStatement<T>>;
}

impl<T> SurrealCrud<T> for T
where
    T: Sized + RecordType + HasId,
{
    fn insert(&self) -> Result<InsertStatement<T>> {
        Query::insert().content(self).map_err(anyhow::Error::from)
    }

    fn find(&self) -> Result<SelectStatement<T>> {
        Ok(Query::select().only())
    }
    fn upsert(&self) -> anyhow::Result<UpsertStatement<T>> {
        Query::upsert().content(self).map_err(anyhow::Error::from)
    }
    fn update(&self) -> anyhow::Result<UpdateStatement<T>> {
        Query::update().content(self).map_err(anyhow::Error::from)
    }

    fn delete(&self) -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }
}
