//! Index definition & alternations statements.
//!
//! //DEFINE INDEX [ OVERWRITE | IF NOT EXISTS ] @name ON [ TABLE ] @Table [ FIELDS | COLUMNS ]
//! @fields
//!	[ UNIQUE
//!        | SEARCH ANALYZER @analyzer [ BM25 [(@k1, @b)] ] [ HIGHLIGHTS ]
//!        | MTREE DIMENSION @dimension [ TYPE @type ] [ DIST @distance ] [ CAPACITY @capacity]
//!        | HNSW DIMENSION @dimension [ TYPE @type ] [DIST @distance] [ EFC @efc ] [ M @m ]
//!    ]
//!    [ COMMENT @string ]
//!    [ CONCURRENTLY ]

mod define;
mod delete;

pub use define::*;
pub use delete::*;

/// Shorthand for constructing any index statement
#[derive(Debug, Clone)]
pub struct Index;

/// All available types of index statement
#[derive(Debug, Clone)]
pub enum IndexStatement {
    Define(DefineIndexStatement),
    Delete(IndexDeleteStatement),
}

impl Index {
    /// Define index [`IndexDefineStatement`]
    pub fn define() -> DefineIndexStatement {
        DefineIndexStatement::new()
    }

    /// Delete index [`IndexDeleteStatement`]
    pub fn delete() -> IndexDeleteStatement {
        IndexDeleteStatement::new()
    }
}

