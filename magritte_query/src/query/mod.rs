use std::marker::PhantomData;
use crate::backend::QueryBuilder;
use crate::query::info::InfoStatement;
use crate::query::insert::InsertStatement;
use crate::query::relate::RelateStatement;
use crate::query::select::SelectStatement;
use crate::query::update::UpdateStatement;
use crate::query::upsert::UpsertStatement;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::Surreal;
use crate::query::alter::AlterStatement;
use crate::query::create::CreateStatement;
use crate::query::delete::DeleteStatement;
use crate::types::{RecordType, TableType};

pub mod info;
pub mod insert;
pub mod relate;
pub mod alter;
pub mod create;
pub mod delete;
pub mod select;
pub mod update;
pub mod upsert;
/// Shorthand for constructing any Table query
#[derive(Debug, Clone)]
pub struct Query<T> {
    phantom_data: PhantomData<T>
}

/// All available types of Table query
#[derive(Debug, Clone)]
pub enum QueryStatement<T> where T:RecordType {
    Select(SelectStatement<T>),
    Create(CreateStatement<T>),
    Alter(AlterStatement),
    Insert(InsertStatement<T>),
    Update(UpdateStatement<T>),
    Delete(DeleteStatement<T>),
    Upsert(UpsertStatement<T>),
    Relate(RelateStatement<T>),
    Info(InfoStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubQueryStatement<T> where T:RecordType {
    SelectStatement(SelectStatement<T>),
    InsertStatement(InsertStatement<T>),
    UpdateStatement(UpdateStatement<T>),
    DeleteStatement(DeleteStatement<T>),
    UpsertStatement(UpsertStatement<T>),
    RelateStatement(RelateStatement<T>),
}

impl <T>Query<T>
where
    T: RecordType
{
    /// CREATE statement [`CreateStatement`]
    pub fn create() -> CreateStatement<T> {
        CreateStatement::new()
    }

    /// ALTER statement [`AlterStatement`]
    pub fn alter() -> AlterStatement { AlterStatement::new() }
    /// SELECT statement [`SelectStatement`]
    pub fn select() -> SelectStatement<T> {
        SelectStatement::new()
    }

    /// INSERT statement [`InsertStatement`]
    pub fn insert() -> InsertStatement<T> {
        InsertStatement::new()
    }

    /// UPDATE statement [`UpdateStatement`]
    pub fn update() -> UpdateStatement<T> {
        UpdateStatement::new()
    }

    /// DELETE statement [`DeleteStatement`]
    pub fn delete() -> DeleteStatement<T> {
        DeleteStatement::new()
    }
    /// UPSERT statement [`UpsertStatement`]
    pub fn upsert() -> UpsertStatement<T> {
        UpsertStatement::new()
    }
    /// INFO statement [`InfoStatement`]
    pub fn info() -> InfoStatement {
        InfoStatement::new(Arc::new(Surreal::init()))
    }
}
