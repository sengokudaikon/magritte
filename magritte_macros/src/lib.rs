//! magritte Macros - Type-safe SurrealDB schema definitions
//!
//! This crate provides a set of derive macros for defining SurrealDB schemas in a type-safe way.
//! The macros generate the necessary implementations for tables, edges, events, and indexes.
//!
//! # Core Requirements
//!
//! For any table definition, you MUST define corresponding Events and Indexes enums, even if empty:
//!
//! ```rust,ignore
//! #[derive(Table)]
//! struct User { ... }
//!
//! #[derive(Index)]
//! enum UserIndexes {} // Required, even if empty
//!
//! #[derive(Event)]
//! enum UserEvents {} // Required, even if empty
//! ```
//!
//! # Real-World Examples
//!
//! ## Complex Table with Relations and Validations
//!
//! ```rust,ignore
//! #[derive(Table, Serialize, Deserialize, Clone)]
//! #[table(name = "orders")]
//! pub struct Order {
//!     id: SurrealId<Self>,
//!
//!     #[column(type = "datetime")]
//!     created_at: String,
//!
//!     #[column(type = "record<users>", assert = "value != NONE")]
//!     user: RecordRef<User>,
//!
//!     #[column(type = "array<record<products>>")]
//!     items: Vec<RecordRef<Product>>,
//!
//!     #[column(type = "decimal", assert = "value >= 0")]
//!     total: f64,
//!
//!     #[column(type = "object", flexible)]
//!     shipping_info: serde_json::Value,
//!
//!     #[column(value = "pending|processing|shipped|delivered")]
//!     status: String,
//! }
//!
//! // Required trait implementation
//! impl HasId for Order {
//!     fn id(&self) -> SurrealId<Self> {
//!         self.id.clone()
//!     }
//! }
//! ```
//!
//! ## Table with Rich Column Attributes
//!
//! ```rust,ignore
//! #[derive(Table, Serialize, Deserialize, Clone)]
//! #[table(name = "products", schema = "SCHEMALESS")]
//! pub struct Product {
//!     id: SurrealId<Self>,
//!
//!     #[column(type = "string")]
//!     name: String,
//!
//!     #[column(type = "int", nullable, default = "0")]
//!     quantity: Option<i32>,
//!
//!     #[column(
//!         type = "float",
//!         nullable,
//!         default = "0.0",
//!         assert = "value >= 0",
//!         permissions = ["full"],
//!         readonly,
//!         flexible,
//!         comment = "Product price with validation"
//!     )]
//!     price: Option<f64>,
//!
//!     #[column(type = "string", assert = "value != NONE")]
//!     sku: String,
//!
//!     #[column(type = "object", flexible = true)]
//!     metadata: serde_json::Value,
//! }
//! ```
//!
//! ## Complex Event with Business Logic
//!
//! ```rust,ignore
//! #[derive(Event, Serialize, Deserialize, strum::EnumIter)]
//! pub enum ProductEvents {
//!     #[event(
//!         name = "created",
//!         when = "var:before==NONE",
//!         then = "UPDATE products SET status = 'pending';
//!             CREATE log SET
//!             order = var:value.id,
//!             action = 'product' + ' ' + var:event.lowercase(),
//!             old_status = '',
//!             new_status = var:after.status ?? 'pending',
//!             at = time::now()
//!             "
//!     )]
//!     ProductCreated,
//! }
//! ```
//!
//! ## Table with All Available Attributes
//!
//! ```rust,ignore
//! #[derive(Table, Serialize, Deserialize, Clone)]
//! #[table(
//!     name = "posts",
//!     schema = "SCHEMAFULL",
//!     permissions = ["full"],
//!     overwrite,
//!     comment = "Posts table with all attributes",
//!     changefeed = "1", // in minutes
//!     include_original
//! )]
//! pub struct Posts {
//!     id: SurrealId<Self>,
//!     content: serde_json::Value,
//! }
//! ```
//!
//! # Usage with EntityManager
//!
//! ```rust,ignore
//! use magritte::*;
//!
//! // Create a new record
//! let user = User::new("Alice", "Alice", "alice@me.com");
//! let result = user.insert(/** implicit self */)?.execute(db.clone()).await?;
//!
//! // Find by ID
//! let user = User::find_by_id(SurrealId::from("Alice") /** any type that can be converted to a SurrealDB id */)?
//!     .execute(db.clone())
//!     .await?;
//! ```
//!
//! # Type System
//!
//! Types are resolved in the following order:
//! 1. Explicit override via `#[column(type = "...")]`
//! 2. Automatic conversion from Rust types
//! 3. Fallback to `any` type
//!
//! Special cases:
//! - Computed fields: `#[column(value = "...")]`
//! - Default values: `#[column(default = "...")]`
//! - Future values: `#[column(type = "future")]`
//! - Flexible objects: `#[column(flexible)]`
//!
//! # Best Practices
//!
//! 1. Always implement `HasId` for your tables and edges
//! 2. Always derive `Serialize`, `Deserialize`, and `Clone`
//! 3. Use `RecordRef<T>` for relations between tables
//! 4. Use assertions to validate data integrity
//! 5. Use events for business logic and audit logging
//! 6. Use indexes for frequently queried fields
//! 7. Group related tables, edges, and their relationships in modules
//!
//! # Type Safety
//!
//! The macros provide compile-time guarantees for:
//! - Field types and nullability
//! - Relationship validity
//! - Event and index configurations
//! - Schema consistency
//!
//! # Experimental Features enabled in the crate
//!
//! #![feature(duration_constructors)] - necessary for Duration parsing from minutes. Might be removed in the future if we opt for seconds-based parsing.
//! #![feature(const_type_id)] - TypeId::of is used in the proc macros to write definitions into a global registry.

#![feature(duration_constructors)]
#![feature(const_type_id)]
#![allow(unused)]
extern crate proc_macro;
mod conversion;
mod derives;
mod strum;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    derives::expand_derive_table(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Column, attributes(column))]
pub fn derive_column(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    derives::expand_derive_column(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    derives::expand_derive_event(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Index, attributes(index))]
pub fn derive_index(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    derives::expand_derive_index(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Relation, attributes(relate))]
pub fn derive_relation(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    derives::expand_derive_relation(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Edge, attributes(edge, column))]
pub fn derive_edge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_edge(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Reimported from strum crate to allow for enum iteration.
#[proc_macro_derive(EnumIter, attributes(strum))]
pub fn enum_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    strum::enum_iter::enum_iter_inner(&ast)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
