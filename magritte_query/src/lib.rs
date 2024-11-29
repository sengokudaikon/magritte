#![allow(ambiguous_glob_reexports)]
#![allow(unused)]
pub(crate) mod backend;

pub(crate) mod func;
pub(crate) mod query;

pub(crate) mod define;


use std::sync::Arc;
pub use anyhow::Result;
pub use surrealdb::engine::any::Any;
pub use surrealdb::Surreal;
pub type SurrealDB = Arc<Surreal<Any>>;

pub use backend::value::SqlValue;
pub use backend::*;
pub use expr::*;
pub use func::*;
pub use query::*;
pub use define::*;