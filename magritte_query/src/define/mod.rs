pub(crate) mod access;
pub(crate) mod analyzer;
pub(crate) mod config;
pub(crate) mod database;
pub(crate) mod event;
pub(crate) mod field;
pub(crate) mod function;
pub(crate) mod index;
pub(crate) mod namespace;
pub(crate) mod param;
pub(crate) mod table;
pub(crate) mod token;
pub(crate) mod user;

use crate::define::define_table::DefineTableStatement;
use crate::{ColumnType, EdgeType, EventType, IndexType, RecordType};
pub use access::*;
pub use analyzer::*;
pub use config::*;
pub use database::*;
pub use event::*;
pub use field::*;
pub use function::*;
pub use index::*;
pub use namespace::*;
pub use param::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::marker::PhantomData;
use surrealdb::sql::statements::{DefineAnalyzerStatement, DefineDatabaseStatement, DefineFunctionStatement, DefineNamespaceStatement, DefineParamStatement, DefineUserStatement};
pub use table::*;
pub use token::*;
pub use user::*;
use crate::define_edge::DefineEdgeStatement;

#[derive(Debug, Clone)]
pub struct Define;
impl Define  {
    pub fn access() -> DefineAccessStatement {
        DefineAccessStatement::default()
    }
    pub fn analyzer() -> DefineAnalyzerStatement {
        DefineAnalyzerStatement::default()
    }
    pub fn config() -> DefineConfigStatement {
        DefineConfigStatement::default()
    }
    pub fn database() -> DefineDatabaseStatement {
        DefineDatabaseStatement::default()
    }
    pub fn event() -> DefineEventStatement {
        DefineEventStatement::default()
    }
    pub fn field () -> DefineFieldStatement {
        DefineFieldStatement::new()
    }
    pub fn function() -> DefineFunctionStatement {
        DefineFunctionStatement::default()
    }
    pub fn index() -> DefineIndexStatement
    {
        DefineIndexStatement::new()
    }
    pub fn edge<E>() -> DefineEdgeStatement<E> where E: EdgeType {
        DefineEdgeStatement::new()
    }
    pub fn namespace() -> DefineNamespaceStatement {
        DefineNamespaceStatement::default()
    }
    pub fn param() -> DefineParamStatement {
        DefineParamStatement::default()
    }
    pub fn table<T> () -> DefineTableStatement<T> where T: RecordType {
        DefineTableStatement::new()
    }
    pub fn token() -> DefineTokenStatement {
        DefineTokenStatement::default()
    }
    pub fn user() -> DefineUserStatement {
        DefineUserStatement::default()
    }
}
