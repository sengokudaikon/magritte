use crate::RelationTrait;
use anyhow::Result;
use magritte_query::{
    DeleteStatement, HasId, InsertStatement, Query, RecordType, SelectStatement, SurrealId,
    UpdateStatement, UpsertStatement,
};

pub trait SurrealCrud<T>: Sized
where
    T: RecordType + HasId,
{
    fn insert(self) -> anyhow::Result<InsertStatement<T>>;
    fn insert_by_id(id: SurrealId<T>, entity: T) -> Result<InsertStatement<T>>;
    fn find(self) -> anyhow::Result<SelectStatement<T>>;
    fn find_by_id(id: SurrealId<T>) -> Result<SelectStatement<T>>;
    fn find_all() -> anyhow::Result<SelectStatement<T>>;
    fn count() -> anyhow::Result<SelectStatement<T>>;
    fn upsert(self) -> anyhow::Result<UpsertStatement<T>>;
    fn upsert_by_id(id: SurrealId<T>, entity: T) -> Result<UpsertStatement<T>>;
    fn update(self) -> anyhow::Result<UpdateStatement<T>>;
    fn update_by_id(id: SurrealId<T>, entity: T) -> Result<UpdateStatement<T>>;
    fn delete(&self) -> anyhow::Result<DeleteStatement<T>>;
    fn delete_by_id(id: SurrealId<T>) -> Result<DeleteStatement<T>>;
    fn delete_all() -> anyhow::Result<DeleteStatement<T>>;

    /// Fetch the source `T` along with a single relation `R` that has `R::Source = T`.
    /// Returns `(T, Vec<R::Target>)` as the typed result.
    fn with_related<R>(relation: R) -> SelectStatement<(T, Vec<R::Target>)>
    where
        R: RelationTrait<Source = T>,
        R::Target: RecordType + HasId,
    {
        let def = relation.def_owned();
        Query::select()
            .field("*", None)
            .field(def.relation_to(), Some(def.relation_name()))
            .fetch(&[def.relation_name()])
    }
}

impl<T> SurrealCrud<T> for T
where
    T: Sized + RecordType + HasId,
{
    fn insert(self) -> Result<InsertStatement<T>> {
        Query::insert().content(self).map_err(anyhow::Error::from)
    }

    fn insert_by_id(id: SurrealId<T>, entity: T) -> Result<InsertStatement<T>> {
        Query::insert()
            .where_id(id)
            .content(entity)
            .map_err(anyhow::Error::from)
    }

    fn find(self) -> Result<SelectStatement<T>> {
        Ok(Query::select().only())
    }

    fn find_by_id(id: SurrealId<T>) -> Result<SelectStatement<T>> {
        Ok(Query::select().where_id(id))
    }

    fn find_all() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select())
    }

    fn count() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select().count())
    }

    fn upsert(self) -> anyhow::Result<UpsertStatement<T>> {
        Query::upsert().content(self).map_err(anyhow::Error::from)
    }

    fn upsert_by_id(id: SurrealId<T>, entity: T) -> Result<UpsertStatement<T>> {
        Query::upsert()
            .where_id(id)
            .content(entity)
            .map_err(anyhow::Error::from)
    }

    fn update(self) -> anyhow::Result<UpdateStatement<T>> {
        Query::update().content(self).map_err(anyhow::Error::from)
    }

    fn update_by_id(id: SurrealId<T>, entity: T) -> Result<UpdateStatement<T>> {
        Query::update()
            .where_id(id)
            .content(entity)
            .map_err(anyhow::Error::from)
    }

    fn delete(&self) -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }

    fn delete_by_id(id: SurrealId<T>) -> Result<DeleteStatement<T>> {
        Ok(Query::delete().where_id(id))
    }

    fn delete_all() -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }
}
