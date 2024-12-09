use crate::define::table::define_table::DefineTableStatement;
use crate::query::alter::AlterStatement;
use crate::query::delete::DeleteStatement;
use crate::query::update::UpdateStatement;
use crate::types::{RecordType, TableType};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub mod define_table;
pub mod define_edge;

/// Helper for constructing any Table statement
#[derive(Debug)]
pub struct Table<T> {
    phantom_data: PhantomData<T>,
}

/// All available types of Table statement
#[derive(Debug, Clone)]
pub enum TableStatement<T> where T:TableType{
    Define(DefineTableStatement<T>),
    Alter(AlterStatement),
    Delete(DeleteStatement<T>),
    Update(UpdateStatement<T>),
}

impl<T> Table<T>
where T:TableType
{
    pub fn define() -> DefineTableStatement<T> {
        DefineTableStatement::new()
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
where T:TableType
{
    /// Build corresponding SQL statement for certain database backend and return SQL string
    pub fn build(&self) -> anyhow::Result<String> {
        match self {
            Self::Define(stat) => stat.build(),
            Self::Alter(stat) => stat.build(),
            Self::Delete(stat) => stat.build(),
            Self::Update(stat) => stat.build(),
        }
    }
}
