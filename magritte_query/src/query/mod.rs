use crate::{RecordType, SurrealDB};
use serde::de::DeserializeOwned;
use serde::Serialize;
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
pub struct Query;

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
    Relate(RelateStatement),
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
    RelateStatement(RelateStatement),
}

impl Query {
    /// CREATE statement [`CreateStatement`]
    pub fn create<T: RecordType>() -> CreateStatement<T> {
        CreateStatement::new()
    }

    /// ALTER statement [`AlterStatement`]
    pub fn alter() -> AlterStatement {
        AlterStatement::new()
    }
    /// SELECT statement [`SelectStatement`]
    pub fn select<T: RecordType>() -> SelectStatement<T> {
        SelectStatement::new()
    }

    /// INSERT statement [`InsertStatement`]
    pub fn insert<T: RecordType>() -> InsertStatement<T> {
        InsertStatement::new()
    }

    /// RELATE statement [`RelateStatement`]
    pub fn relate() -> RelateStatement {
        RelateStatement::new()
    }

    /// UPDATE statement [`UpdateStatement`]
    pub fn update<T: RecordType>() -> UpdateStatement<T> {
        UpdateStatement::new()
    }

    /// DELETE statement [`DeleteStatement`]
    pub fn delete<T: RecordType>() -> DeleteStatement<T> {
        DeleteStatement::new()
    }
    /// UPSERT statement [`UpsertStatement`]
    pub fn upsert<T: RecordType>() -> UpsertStatement<T> {
        UpsertStatement::new()
    }

    pub fn info(db: SurrealDB) -> InfoStatement {
        InfoStatement::new(db)
    }

    /// Begin a transaction
    pub fn begin() -> TransactionStatement {
        TransactionStatement::new()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionStatement {
    statements: Vec<String>,
}

impl TransactionStatement {
    pub fn new() -> Self {
        Self {
            statements: vec!["BEGIN TRANSACTION".to_string()],
        }
    }

    pub fn then<S: StatementBuilder>(mut self, statement: S) -> Self {
        self.statements.push(statement.build().unwrap());
        self
    }

    pub fn raw(mut self, statement: String) -> Self {
        self.statements.push(statement);
        self
    }

    pub fn commit(mut self) -> Self {
        self.statements.push("COMMIT TRANSACTION".to_string());
        self
    }

    pub fn rollback(mut self) -> Self {
        self.statements.push("CANCEL TRANSACTION".to_string());
        self
    }

    pub fn build(&self) -> String {
        self.statements.join("; ") + ";"
    }

    pub async fn execute(self, db: &SurrealDB) -> anyhow::Result<()> {
        db.query(self.build()).await?;
        Ok(())
    }
}

pub trait StatementBuilder {
    fn build(&self) -> anyhow::Result<String>;
}

impl StatementBuilder for AlterStatement {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}

impl<T: RecordType> StatementBuilder for SelectStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}

impl<T: RecordType> StatementBuilder for InsertStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}

impl<T: RecordType> StatementBuilder for UpdateStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}
impl StatementBuilder for RelateStatement {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}
impl<T: RecordType> StatementBuilder for DeleteStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}

impl<T: RecordType> StatementBuilder for UpsertStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
}