//! Standard SurrealDB functions implementation

pub mod array;
pub mod bytes;
pub mod count;
pub mod crypto;
pub mod duration;
pub mod geometry;
pub mod http;
pub mod math;
pub mod meta;
pub mod not;
pub mod object;
pub mod parse;
pub mod rand;
pub mod record;
pub mod search;
pub mod session;
pub mod sleep;
pub mod string;
pub mod time;
pub mod typ_e;
pub mod value;
pub mod vector;

use std::fmt::Display;

/// Function call.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall<T: Callable> {
    pub(crate) func: T,
    pub(crate) args: Vec<String>,
}
/// Trait for standard SurrealDB functions
pub trait Callable: Display + Send + Sync + Clone + 'static {
    /// Returns the function name with namespace (e.g., "array", "math",
    /// "string")
    fn namespace() -> &'static str;

    /// Returns the function category (e.g., "basic", "distance", "semver")
    fn category(&self) -> &'static str;

    /// Returns true if function can be used in WHERE clause
    fn can_filter(&self) -> bool;
}

// Re-export implemented functions
pub use array::*;
pub use bytes::*;
pub use count::*;
pub use crypto::*;
pub use duration::*;
pub use geometry::*;
pub use http::*;
pub use math::*;
pub use meta::*;
pub use not::*;
pub use object::*;
pub use parse::*;
pub use rand::*;
pub use record::*;
pub use search::*;
pub use session::*;
pub use sleep::*;
pub use string::*;
pub use time::*;
pub use typ_e::*;
pub use value::*;
pub use vector::*;

pub trait CanCallFunctions {
    fn call_function<F: Callable>(self, func: F) -> Self;
}
