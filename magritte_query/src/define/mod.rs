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

use crate::{EdgeType, TableType};
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
pub use table::*;
pub use token::*;
pub use user::*;
pub use define_edge::*;
pub use define_table::*;

#[derive(Debug, Clone)]
pub struct Define;
impl Define {
    pub fn access() -> DefineAccessStatement {
        DefineAccessStatement::new()
    }
    pub fn analyzer() -> DefineAnalyzerStatement {
        DefineAnalyzerStatement::new()
    }
    pub fn config() -> DefineConfigStatement {
        DefineConfigStatement::new()
    }
    pub fn database() -> DefineDatabaseStatement {
        DefineDatabaseStatement::new()
    }
    pub fn event() -> DefineEventStatement {
        DefineEventStatement::new()
    }
    pub fn field() -> DefineFieldStatement {
        DefineFieldStatement::new()
    }
    pub fn function() -> DefineFunctionStatement {
        DefineFunctionStatement::new()
    }
    pub fn index() -> DefineIndexStatement {
        DefineIndexStatement::new()
    }
    pub fn edge<E>() -> DefineEdgeStatement<E>
    where
        E: EdgeType,
    {
        DefineEdgeStatement::new()
    }
    pub fn namespace() -> DefineNamespaceStatement {
        DefineNamespaceStatement::new()
    }
    pub fn param() -> DefineParamStatement {
        DefineParamStatement::new()
    }
    pub fn table<T>() -> DefineTableStatement<T>
    where
        T: TableType,
    {
        DefineTableStatement::new()
    }
    pub fn token() -> DefineTokenStatement {
        DefineTokenStatement::new()
    }
    pub fn user() -> DefineUserStatement {
        DefineUserStatement::new()
    }
}
