use magritte_query::{
    DeleteStatement, HasId, InsertStatement, Query, RecordType, SelectStatement, SurrealId,
    UpdateStatement, UpsertStatement,
};
use anyhow::{anyhow, Result};
/// for relations:
/// SELECT ->?->? AS relations FROM something FETCH relations
/// results in attached tables from the relation
///
/// SELECT
///     *,
///     ->? AS alias1,
///     <-? AS alias2,
///     <->? AS alias3
/// FROM something
/// FETCH alias1, alias2, alias3
/// results in just the keys for alias1,alias2,alias3 globbed
///
/// SELECT
///     *,
///     ->?->? AS alias1,
///     <-?<-? AS alias2,
///     <->?<->? AS alias3
/// FROM something
/// FETCH alias1, alias2, alias3
/// results in attached tables from all relations?

pub trait SurrealCrud<T>: Sized
where
    T: RecordType + HasId,
{
    fn insert(self) -> anyhow::Result<InsertStatement<T>>;
    fn insert_by_id<I: Into<SurrealId<T>>>(self, id: I) -> anyhow::Result<InsertStatement<T>>;
    fn get(self) -> anyhow::Result<SelectStatement<T>>;
    fn upsert(self) -> anyhow::Result<UpsertStatement<T>>;
    fn update(self) -> anyhow::Result<UpdateStatement<T>>;

    /// Finds a record by id.
    fn find_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<SelectStatement<T>>;
    /// Find all records.
    fn find_all() -> anyhow::Result<SelectStatement<T>>;

    /// Count filtered records.
    fn count() -> anyhow::Result<SelectStatement<T>>;

    /// Delete the current record by instance.
    fn delete(&self) -> anyhow::Result<DeleteStatement<T>>;
    fn delete_all() -> anyhow::Result<DeleteStatement<T>>;
    /// Deletes a record by id.
    fn delete_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<DeleteStatement<T>>;
    /// Fetches all edges connected to the record.
    fn fetch_all_edges(&self) -> SelectStatement<T>;

    /// Fetches edges along with related records.
    fn fetch_edges_with_related(&self) -> SelectStatement<T>;

    /// Fetches all relations (edges and related records) for the record.
    fn fetch_all_relations(&self) -> SelectStatement<T>;
}

impl<T> SurrealCrud<T> for T
where
    T: Sized + RecordType + HasId,
{
    fn insert(self) -> Result<InsertStatement<T>> {
        Ok(Query::insert().content(self).map_err( anyhow::Error::from)?)
    }
    fn insert_by_id<I: Into<SurrealId<T>>>(self, id: I) -> Result<InsertStatement<T>> {
        Ok(Query::insert().where_id(id.into()).content(self)?)
    }

    fn get(self) -> Result<SelectStatement<T>> {
        Ok(Query::select().where_id(self.id()))
    }

    fn upsert(self) -> anyhow::Result<UpsertStatement<T>> {
        Ok(Query::upsert().where_id(self.id()).content(self).map_err(anyhow::Error::from)?)
    }

    fn update(self) -> anyhow::Result<UpdateStatement<T>> {
        Ok(Query::update().where_id(self.id()).content(self).map_err(anyhow::Error::from)?)
    }

    fn find_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select().where_id(id.into()))
    }

    fn find_all() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select())
    }

    fn count() -> anyhow::Result<SelectStatement<T>> {
        Ok(Query::select().count())
    }

    fn delete(&self) -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete().where_id(self.id()))
    }

    fn delete_all() -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete())
    }

    fn delete_by_id<I: Into<SurrealId<T>>>(id: I) -> anyhow::Result<DeleteStatement<T>> {
        Ok(Query::delete().where_id(id.into()))
    }
    fn fetch_all_edges(&self) -> SelectStatement<T> {
        Query::select()
            .field("*", None)
            .field("->?", Some("edges")) // Fetch all outgoing edges
            .fetch(&["edges"])
            .where_id(self.id())
    }

    fn fetch_edges_with_related(&self) -> SelectStatement<T> {
        Query::select()
            .field("*", None)
            .relation_wildcard_as("related") // Fetch all outgoing edges
            .fetch(&["related"])
            .where_id(self.id())
    }

    fn fetch_all_relations(&self) -> SelectStatement<T> {
        Query::select()
            .field("*", None)
            .relation_wildcard_as("outbound")
            .relation_inverse_wildcard_as("inbound")
            .relation_bidirectional_wildcard_as("relations")
            .fetch(&["relations", "inbound", "outbound"])
            .where_id(self.id())
    }
}
