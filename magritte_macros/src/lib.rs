#![feature(duration_constructors)]
#![feature(structural_match)]
#![allow(unused)]
extern crate proc_macro;
mod conversion;
mod derives;
mod strum;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};
use thiserror::Error;


/// magritte Macros - Type-safe SurrealDB schema definitions
///
/// This crate provides a set of derive macros for defining SurrealDB schemas in a type-safe way.
/// The macros generate the necessary implementations for tables, edges, events, and indexes.
///
/// # Core Concepts
///
/// - **Tables**: Main data structures defined using the `Table` derive macro
/// - **Edges**: Relationships between tables defined using the `Edge` derive macro
/// - **Events**: Database triggers defined using the `Event` derive macro
/// - **Indexes**: Search and lookup optimizations defined using the `Index` derive macro
/// - **Relations**: High-level relationship definitions using the `Relation` derive macro
///
/// # Example
///
/// ```rust,ignore
/// use magritte::prelude::*;
/// use chrono::{DateTime, Utc};
/// use serde::{Deserialize, Serialize};
///
/// // Define a table
/// #[derive(Table, Debug, Serialize, Deserialize)]
/// #[table(
///     name = "users",
///     schema = "schemafull",
///     permissions = ["for select where user = $auth.id"]
/// )]
/// pub struct User {
///     id: RecordId,                    // automatically handled
///     name: String,                    // automatically handled
///     email: Option<String>,           // automatically nullable
///     #[column(unique, type = "text")] // explicit attributes when needed
///     username: String,
/// }
///
/// // Define an edge
/// #[derive(Edge)]
/// #[edge(from = "User", to = "Post", enforced)]
/// struct Authored {
///     #[column(type = "datetime")]
///     created_at: DateTime<Utc>,
/// }
///
/// // Define events
/// #[derive(Event)]
/// enum UserEvents {
///     #[event(
///         table = "users",
///         when = "CREATE",
///         then = "CREATE audit::log SET user = $after.id, action = 'created'"
///     )]
///     Created,
/// }
///
/// // Define indexes
/// #[derive(Index)]
/// enum UserIndexes {
///     #[index(table = "users",fields = ["email"], unique)]
///     Email,
///
///     #[index(
///         table = "users",
///         fields = ["bio"],
///         search(analyzer = "english", highlights = true)
///     )]
///     Bio,
/// }
///
/// // Define relations
/// #[derive(Relation)]
/// enum UserRelations {
///     #[relate(
///         from = "users",
///         in_id = "id",
///         to = "Post",
///         out_id = "id",
///         edge = "Authored",
///         content = "{ created_at: time::now() }"
///     )]
///     Posts,
/// }
/// ```
///
/// # Features
///
/// ## Table Derive
///
/// The `Table` derive macro automatically:
/// - Generates a Column enum for type-safe field access
/// - Handles field types and nullability automatically
/// - Supports custom column attributes
/// - Generates table definitions with permissions and schema settings
///
/// ## Edge Derive
///
/// The `Edge` derive macro:
/// - Defines type-safe relationships between tables
/// - Supports enforced relationships
/// - Allows custom edge fields
/// - Handles schema and permissions
///
/// ## Event Derive
///
/// The `Event` derive macro:
/// - Creates type-safe database triggers
/// - Supports all SurrealDB event types
/// - Allows complex trigger conditions and actions
/// - Automatically links events to tables
///
/// ## Index Derive
///
/// The `Index` derive macro supports:
/// - Regular indexes with uniqueness constraints
/// - Full-text search indexes with analyzers and BM25
/// - Vector indexes (MTREE, HNSW) for similarity search
/// - Automatic index naming and configuration
///
/// ## Relation Derive
///
/// The `Relation` derive macro:
/// - Creates high-level relationship definitions
/// - Handles record ID formatting
/// - Supports custom edge content
/// - Provides type-safe relationship traversal
///
/// # Best Practices
///
/// 1. Use automatic field handling when possible, only add explicit attributes when needed
/// 2. Group related tables, edges, and their relationships in modules
/// 3. Use enforced edges for required relationships
/// 4. Add appropriate indexes for frequently queried fields
/// 5. Use events for audit logging and data consistency
///
/// # Type Safety
///
/// The macros provide compile-time guarantees for:
/// - Field types and nullability
/// - Relationship validity
/// - Event and index configurations
/// - Schema consistency
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

#[proc_macro_derive(EnumIter, attributes(strum))]
pub fn enum_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    strum::enum_iter::enum_iter_inner(&ast)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
