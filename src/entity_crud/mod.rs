use std::process::id;
use surrealdb::sql;
use magritte_query::delete::DeleteStatement;
use magritte_query::insert::InsertStatement;
use magritte_query::Query;
use magritte_query::select::SelectStatement;
use magritte_query::types::{HasId, RecordType, SurrealId};
use magritte_query::upsert::UpsertStatement;
use crate::prelude::TableTrait;

pub trait SurrealCrud<T> : Sized where T: RecordType + HasId {
    fn insert(self) -> anyhow::Result<InsertStatement<T>>;
    /// Creates or updates a model/table in the database.
    fn upsert(self) -> anyhow::Result<UpsertStatement<T>>;

    /// Finds a record by id.
    fn find_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<SelectStatement<T>>;

    /// Finds records by filtering.
    fn find_where() -> anyhow::Result<SelectStatement<T>>;

    /// Count filtered records.
    fn count_where() -> anyhow::Result<SelectStatement<T>>;
    /// Count all records.
    fn count_all() -> anyhow::Result<SelectStatement<T> >;

    /// Delete the current record by instance.
    fn delete(&self) -> anyhow::Result<DeleteStatement<T>>;

    /// Deletes a record by id.
    fn delete_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<DeleteStatement<T>>;

    /// Deletes records by filtering.
    fn delete_where() -> anyhow::Result<DeleteStatement<T>>;
}

impl<T> SurrealCrud<T> for T where T: Sized + RecordType + HasId {
    fn insert(self) -> anyhow::Result<InsertStatement<T>> {
        Ok(Query::insert().content(self)?)
    }

    fn upsert(self) -> anyhow::Result<UpsertStatement<T>> {
        Ok(Query::upsert().content(self)?)
    }

    fn find_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select().where_id(id.into()))
    }

    fn find_where() -> anyhow::Result<SelectStatement<T>> {
        todo!()
    }

    fn count_where() -> anyhow::Result<SelectStatement<T>> {
        todo!()
    }

    fn count_all() -> anyhow::Result<SelectStatement<T>> {
        todo!()
    }

    fn delete(&self) -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }

    fn delete_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<DeleteStatement<T>>  {
        Ok(Query::delete().where_id(id.into()))
    }

    fn delete_where() -> anyhow::Result<DeleteStatement<T>> {
        todo!()
    }
}
