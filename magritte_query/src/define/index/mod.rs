//! Index definition & alternations statements.
//!
//! # Usage
//!
//! - Table Index Define, see [`IndexDefineStatement`]
//! - Table Index Delete, see [`IndexDeleteStatement`]

mod common;
mod define;
mod delete;

pub use common::*;
pub use define::*;
pub use delete::*;

/// Shorthand for constructing any index statement
#[derive(Debug, Clone)]
pub struct Index;

/// All available types of index statement
#[derive(Debug, Clone)]
pub enum IndexStatement {
    Define(IndexDefineStatement),
    Delete(IndexDeleteStatement),
}

impl Index {
    /// Define index [`IndexDefineStatement`]
    pub fn define() -> IndexDefineStatement {
        IndexDefineStatement::new()
    }

    /// Delete index [`IndexDeleteStatement`]
    pub fn delete() -> IndexDeleteStatement {
        IndexDeleteStatement::new()
    }
}

pub trait Indexable {
    fn with_index(&self) -> &Option<Vec<String>>;
    fn with_index_mut(&mut self) -> &mut Option<Vec<String>>;
}