use crate::RecordType;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
pub mod alter;
pub mod create;
pub mod delete;
pub mod info;
pub mod insert;
pub mod relate;
pub mod select;
pub mod update;
pub mod upsert;

pub use alter::*;
pub use create::*;
pub use delete::*;
pub use info::*;
pub use insert::*;
pub use relate::*;
pub use select::*;
pub use update::*;
pub use upsert::*;

/// Shorthand for constructing any Table query
#[derive(Debug, Clone)]
pub struct Query<T> {
    phantom_data: PhantomData<T>,
}

/// All available types of Table query
#[derive(Debug, Clone)]
pub enum QueryStatement<T>
where
    T: RecordType,
{
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
pub enum SubQueryStatement<T>
where
    T: RecordType,
{
    SelectStatement(SelectStatement<T>),
    InsertStatement(InsertStatement<T>),
    UpdateStatement(UpdateStatement<T>),
    DeleteStatement(DeleteStatement<T>),
    UpsertStatement(UpsertStatement<T>),
    RelateStatement(RelateStatement<T>),
}

impl<T> Query<T>
where
    T: RecordType,
{
    /// CREATE statement [`CreateStatement`]
    pub fn create() -> CreateStatement<T> {
        CreateStatement::new()
    }

    /// ALTER statement [`AlterStatement`]
    pub fn alter() -> AlterStatement {
        AlterStatement::new()
    }
    /// SELECT statement [`SelectStatement`]
    pub fn select() -> SelectStatement<T> {
        SelectStatement::new()
    }

    /// INSERT statement [`InsertStatement`]
    pub fn insert() -> InsertStatement<T> {
        InsertStatement::new()
    }

    /// RELATE statement [`RelateStatement`]
    pub fn relate() -> RelateStatement<T> {
        RelateStatement::new()
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
}
