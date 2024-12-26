# magritte Derive

Procedural macros for generating SurrealDB schema definitions and operations.

## Core Requirements

**Important**: For any table/edge definition, you MUST define corresponding empty Events and Indexes enums, even if you don't use them:

```rust, ignore
#[derive(Table)]
struct User { ... }

#[derive(Index)]
enum UserIndexes {} // Required, even if empty

#[derive(Event)]
enum UserEvents {} // Required, even if empty
```

This is necessary because we generate schema definitions at compile time for migrations, even if you don't use them.
Sad, but true. You're welcome to contribute to the library to make it more flexible.

## Type System

The macro provides automatic type conversion from Rust types to SurrealDB types. Types are resolved in the following order:

1. Explicit type override via `#[column(type = "...")]`
2. Automatic conversion based on Rust type:
   - Primitive types (bool, integers, floats, strings)
   - Optional types via `Option<T>`
   - Collections (Vec, Array, Set, HashSet)
   - Record references via `RecordRef<T>` Note: it converts to `record<T>`, where T is the name of the struct deriving Table/Edge, snake_case'd. This means that your structs HAVE to match the expected record name. Otherwise, override the type with `#[column(type = "record<T::table_name>")]`.
   - Feature-gated types:
     - `chrono::DateTime` -> datetime
     - `rust_decimal::Decimal` -> decimal
     - `geo` types -> geometry
     - `uuid::Uuid` -> uuid
3. Falls back to `any` type if unknown

Special cases:

- Computed fields: Use `#[column(value = "...")]`
- Default values: Use `#[column(default = "...")]`
- Future values: Use `#[column(type = "future")]`
- Flexible objects: Use `#[column(flexible)]`

## Derive Macros

The crate provides five main derive macros:

### Table

`#[derive(Table)]` generates implementations for:

- `TableTrait` - Core table functionality
- `HasColumns` - Column definitions and metadata
- `HasEvents` - Event handling
- `HasIndexes` - Index management
- `HasRelations` - Relationship management
- `HasId` - Record ID handling
- `RecordType` - Base record functionality

### Edge

`#[derive(Edge)]` generates implementations for:

- `EdgeTrait` - Edge-specific functionality
- `HasColumns` - Column definitions for edge properties
- `HasEvents` - Event handling
- `HasIndexes` - Index management
- `HasId` - Edge ID handling
- `RecordType` - Base record functionality

### Event

`#[derive(Event)]` generates implementations for:

- `EventTrait` - Event trigger definitions
- `EventType` - Event type information

### Index

`#[derive(Index)]` generates implementations for:

- `IndexTrait` - Index definitions
- `IndexType` - Index type information

### Relation

`#[derive(Relation)]` generates implementations for:

- `RelationTrait` - High-level relationship definitions
- `RelationType` - Relationship type information

Relations are generated as empty structs which contain the definitions, while the base enum defines the possible relations.

## Trait Requirements

Each derive macro requires certain traits to be implemented. Most common ones are automatically derived, but some need manual implementation:

### For Tables and Edges

```rust
pub trait HasId: RecordType {
    fn id(&self) -> &SurrealId;
    fn set_id(&mut self, id: SurrealId);
}
```

The HasID trait is required to be derived because we currently need to expose the ID in the struct to be able to create relations, and hold table:id info.

You **MUST** derive Clone, Serialize, Deserialize, as these traits cannot be currently derived introspectively.

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

The Columns enum is generated automatically, so you shouldn't need to implement this.

### For Indexes

Serialize, Deserialize, strum::EnumIter must be derived on the enum, since these traits cannot be derived at compile time currently.

```rust
#[derive(Serialize, Deserialize, strum::EnumIter)]
pub enum UserIndexes {
    #[index(fields = [name], comment = "Index on user name")]
    NameIdx,
}
```

```rust
pub trait IndexType:
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
    fn index_name(&self) -> & str;
    fn table_name() -> &'static str;
}
```

### For Events

Serialize, Deserialize, strum::EnumIter must be derived on the enum, since these traits cannot be derived at compile time currently.

```rust
#[derive(Serialize, Deserialize, strum::EnumIter)]
pub enum UserEvents {
    #[event(when = "created", then = "UPDATE created_at = now()", comment = "User created event")]
    UserCreated,
}
```

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

### For Relations

Relations are currently unstable, but you can define them similarly to indexes and events.

```rust
#[derive(Relation, Serialize, Deserialize, strum::EnumIter)]
pub enum UserRelations {
    #[relation(from = User, to = Order, via= UserOrders, comment = "User has many orders")]
    HasOrders,
}
```

```rust
pub trait RelationType:
    FromStr
    + Display
    + AsRef<str>
    + Clone
    + Debug
    + Send
    + Sync
    + Copy
    + 'static
{
    fn relation_via() -> String; // this is the edge table name
    fn relation_from() -> String; // this is the source table name
    fn relation_to() -> String; // this is the target table name
}
```

The return types are not static due to complicated nature of relation generation. We create empty structs for each relation, and their definitions hold the values of table/edge names. Therefore tI haven't been able to make them accessible at compile time as static strings.

To actually execute the RELATE statement for tables, you need to either call .relate() on the relation definition, or use relate! macro. You'd need to provide IDs for the source and target records.