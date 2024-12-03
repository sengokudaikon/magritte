use crate::backend::QueryBuilder;
use crate::define::table::define::DefineStatement;
use crate::query::alter::AlterStatement;
use crate::query::delete::DeleteStatement;
use crate::query::update::UpdateStatement;
use crate::types::{RecordType, TableType};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub mod define;

/// Helper for constructing any Table statement
#[derive(Debug)]
pub struct Table<T> {
    phantom_data: PhantomData<T>,
}

/// All available types of Table statement
#[derive(Debug, Clone)]
pub enum TableStatement<T>  where T:RecordType{
    Define(DefineStatement),
    Alter(AlterStatement),
    Delete(DeleteStatement<T>),
    Update(UpdateStatement<T>),
}

impl<T> Table<T>
where T:RecordType
{
    pub fn define() -> DefineStatement {
        DefineStatement::new()
    }
    pub fn alter() -> AlterStatement {
        AlterStatement::new()
    }
    pub fn delete() -> DeleteStatement<T> {
        DeleteStatement::new()
    }
    pub fn update() -> UpdateStatement<T> {
        UpdateStatement::new()
    }
}

impl<T> TableStatement<T>
where T:RecordType
{
    /// Build corresponding SQL statement for certain database backend and return SQL string
    pub fn build(&self) -> anyhow::Result<String> {
        match self {
            Self::Define(stat) => <DefineStatement as QueryBuilder<T>>::build(stat),
            Self::Alter(stat) => stat.build(),
            Self::Delete(stat) => stat.build(),
            Self::Update(stat) => stat.build(),
        }
    }
}
