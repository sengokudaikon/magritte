#![allow(ambiguous_glob_reexports)]
#![allow(unused)]
pub mod backend;
pub mod func;
pub mod query;
pub mod define;


use std::sync::Arc;
pub use anyhow::Result;
pub use surrealdb::engine::any::Any;
pub use surrealdb::Surreal;
pub type SurrealDB = Arc<Surreal<Any>>;

pub use backend::*;
pub use expr::*;
pub use func::*;
pub use query::*;
pub use define::*;