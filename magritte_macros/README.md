# magritte Derive

Procedural macros for generating SurrealDB schema definitions and operations.

## Current Implementation

The macro currently provides two main derive macros:

- `#[derive(Entity, Clone, Debug, Serialize, Deserialize)]` - generates the necessary implementations for a SurrealDB table.
- `#[derive(Edge, Clone, Debug, Serialize, Deserialize)]` - generates the necessary implementations for a SurrealDB edge.

## Usage
### Entity
`Entity` derive macro automatically derives the following traits:

- `#[derive(TableTrait)]` - a foundation for table records
- `#[derive(ColumnTrait)] pub enum EntityColumns { ... }` - a list of table columns
- `#[derive(RelationTrait)] pub enum EntityRelations { ... }` - a list of table relations to edges
- `#[derive(IndexTrait)] pub enum EntityIndexes { ... }` - a list of table indexes
- `#[derive(EventTrait)] pub enum EntityEvents { ... }` - a list of table events

Each `TableTrait` implementor fulfills a contract of 
```rust
pub trait TableType: NamedType +
Display
+ AsRef<str>
+ Debug
+ Serialize
+ DeserializeOwned
+ Clone
+ Send
+ Sync
+ 'static
{
    fn schema_type() -> SchemaType;
}
```

An implementor **MUST** derive Debug, Clone, Serialize, Deserialize, as these traits cannot be currently derived introspectively.

Each `ColumnTrait` implementor fulfills a contract of
```rust
pub trait ColumnType:
FromStr
+ Display
+ AsRef<str>
+ Debug
+ Copy
+ Serialize
+ DeserializeOwned
+ Clone
+ Send
+ Sync
+ strum::IntoEnumIterator
+ 'static
{
    fn table_name() -> &'static str;
    fn column_name(&self) -> & str;
    fn column_type(&self) -> & str;
}
```

The traits are derived automatically for a generated `{}Columns` entry enum.

Each `RelationTrait` implementor fulfills a contract of
```rust

pub trait RelationType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Send
    + Sync
    + Debug
    + Copy
    + strum::IntoEnumIterator
    + 'static
{
    fn relation_via(&self) -> & str;
    fn relation_from(&self) -> & str;
    fn relation_to(&self) -> &str;
}
```

The traits are derived automatically for a generated `{}Relations` entry enum.

Each `IndexTrait` implementor fulfills a contract of
```rust
pub trait IndexType:
FromStr
+ Display
+ AsRef<str>
+ Clone
+ Send
+ Sync
+ Debug
+ Copy
+ strum::IntoEnumIterator
+ 'static
{
    fn index_name(&self) -> & str;
    fn table_name() -> &'static str;
}
```

The traits are derived automatically for a generated `{}Indexes` entry enum.

Each `EventTrait` implementor fulfills a contract of
```rust

pub trait EventType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Debug
    + Send
    + Sync
    + Copy
    + strum::IntoEnumIterator
    + 'static
{
    fn event_name(&self) -> & str;
    fn table_name() -> &'static str;
}
```

The traits are derived automatically for a generated `{}Events` entry enum.

### Edge

`Edge` derive macro automatically derives the following traits:

- `#[derive(ColumnTrait)] pub enum EdgeColumns { ... }` - a list of edge columns

```rust
pub trait EdgeType: NamedType +
Display
+ AsRef<str>
+ Debug
+ Serialize
+ DeserializeOwned
+ Clone
+ Send
+ Sync
+ 'static
{
    fn edge_from(&self) -> & str;
    fn edge_to(&self) -> & str;
    fn is_enforced(&self) -> bool;
}
```

An implementor **MUST** derive Debug, Clone, Serialize, Deserialize, as these traits cannot be currently derived introspectively.

Currently, there is no inverse `RelationTrait` for edges to tables.

The traits are derived automatically for a generated `{}Columns` entry enum, as described above.

Both `Entity` and `Edge` rely on `NamedType`, which is a marker trait that provides a static name for an implementor.

## Planned features

- Migration script generation

Compile-time generated scripts that are then executed at runtime.
