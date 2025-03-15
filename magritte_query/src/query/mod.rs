use crate::{HasParams, RecordType};
use surrealdb::{Response, Surreal};

pub mod alter;
pub mod create;
pub mod delete;
pub mod info;
pub mod insert;
pub mod relate;
pub mod select;
pub mod update;
pub mod upsert;

use magritte_db::{QueryType, SurrealDB};
pub use alter::*;
pub use create::*;
pub use delete::*;
pub use info::*;
pub use insert::*;
pub use relate::*;
pub use select::*;
use serde_json::Value;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Database;
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
    RelateStatement(Box<RelateStatement>),
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

    pub fn info(db: Surreal<Any>) -> InfoStatement {
        InfoStatement::new(db)
    }

    /// Begin a transaction
    pub fn begin() -> TransactionStatement {
        TransactionStatement::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransactionStatement {
    statements: Vec<(String, Vec<(String, Value)>)>,
}

impl TransactionStatement {
    pub fn new() -> Self {
        Self {
            statements: vec![("BEGIN TRANSACTION;".to_string(), vec![])],
        }
    }

    pub fn raw(mut self, query: &str, params: Vec<(String, Value)>) -> Self {
        self.statements.push((query.to_string(), params));
        self
    }

    pub fn commit(mut self) -> Self {
        self.statements.push(("COMMIT TRANSACTION;".to_string(), vec![]));
        self
    }

    pub fn rollback(mut self) -> Self {
        self.statements.push(("CANCEL TRANSACTION;".to_string(), vec![]));
        self
    }

    pub fn then<S: StatementBuilder>(mut self, statement: S) -> Self {
        self.statements.push((statement.build().unwrap(), statement.with_params()));
        self
    }
    pub fn build(&self) -> (String, Vec<(String, Value)>) {
        let mut query_string = String::new();
        let mut all_params = Vec::new();
        let mut param_counter = 0;

        for (i, (query, params)) in self.statements.iter().enumerate() {
            let mut formatted_query = query.trim().to_string();
            // Ensure proper semicolon usage: add only between statements, not after the last one
            if i < self.statements.len() - 1 && !formatted_query.ends_with(';') {
                formatted_query.push(';');
            }

            // Rename parameters to avoid conflicts across statements
            let mut renamed_params = Vec::new();
            for (param_name, value) in params {
                let new_param_name = format!("p{}", param_counter);
                formatted_query = formatted_query.replace(&format!("${}", param_name), &format!("${}", new_param_name));
                renamed_params.push((new_param_name, value.clone()));
                param_counter += 1;
            }
            all_params.extend(renamed_params);

            if i > 0 {
                query_string.push(' ');
            }
            query_string.push_str(&formatted_query);
        }

        // Clean up extra delimiters
        query_string = query_string.replace(";;", ";").replace("; ;", ";").trim().to_string();

        (query_string, all_params)
    }

    pub async fn execute(self, db: &Surreal<Any>) -> surrealdb::Result<Response> {
        let (query, params) = self.build();
        let mut q = db.query(&query);
        if !params.is_empty() {
            q = q.bind(params);
        }
        q.await?.check()
    }
}

pub trait StatementBuilder {
    fn build(&self) -> anyhow::Result<String>;
    fn with_params(&self) -> Vec<(String, Value)> {
        vec![]
    }
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
    fn with_params(&self) -> Vec<(String, Value)> {
        self.params().clone()
    }
}

impl<T: RecordType> StatementBuilder for InsertStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }
    fn with_params(&self) -> Vec<(String, Value)> {
        self.params().clone()
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

    fn with_params(&self) -> Vec<(String, Value)> {
        self.params().clone()
    }
}

impl<T: RecordType> StatementBuilder for UpsertStatement<T> {
    fn build(&self) -> anyhow::Result<String> {
        self.build()
    }

    fn with_params(&self) -> Vec<(String, Value)> {
        self.params().clone()
    }
}
