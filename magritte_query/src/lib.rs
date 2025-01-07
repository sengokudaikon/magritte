#![allow(clippy::wrong_self_convention)]
#![allow(unused)]
#![allow(ambiguous_glob_reexports)]
pub mod backend;
pub mod define;
pub mod func;
pub mod query;
pub type SurrealDB = std::sync::Arc<surrealdb::Surreal<surrealdb::engine::any::Any>>;

pub use backend::*;
pub use define::*;
pub use func::*;
pub use query::*;
