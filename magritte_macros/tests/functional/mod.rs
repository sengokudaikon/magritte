use magritte::*;
use serde::{Deserialize, Serialize};

mod table;

// Test table with nested columns and relationships
#[derive(Table, Serialize, Deserialize, Clone)]
#[table(name = "orders")]
pub struct Order {
    id: SurrealId<Self>,

    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "record<users>", assert = "value != NONE")]
    user: RecordRef<User>,

    #[column(type = "array<record<products>>")]
    items: Vec<RecordRef<Product>>,

    #[column(type = "decimal", assert = "value >= 0")]
    total: f64,

    #[column(type = "object", flexible)]
    shipping_info: serde_json::Value,
    #[column(value = "pending|processing|shipped|delivered")]
    status: String,
}

impl Order {
    pub fn new(
        id: impl Into<SurrealId<Self>>,
        created_at: impl Into<String>,
        user: impl Into<RecordRef<User>>,
        items: impl Into<Vec<RecordRef<Product>>>,
        total: impl Into<f64>,
        shipping_info: impl Into<serde_json::Value>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at: created_at.into(),
            user: user.into(),
            items: items.into(),
            total: total.into(),
            shipping_info: shipping_info.into(),
            status: status.into(),
        }
    }
}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderIndexes {
    #[index(
        name = "status_idx",
        fields = [status],
        comment = "Index on order status"
    )]
    StatusIdx,
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderEvents {}

#[derive(Table, Serialize, Deserialize, Clone)]
#[table(name = "products", schema = "SCHEMALESS")]
pub struct Product {
    id: SurrealId<Self>,
    // Basic column
    #[column(type = "string")]
    name: String,

    // Nullable column with default
    #[column(type = "int", nullable, default = "0")]
    quantity: i32,

    // Column with all attributes
    #[column(
            type = "float",
            nullable,
            default = "0.0",
            assert = "value >= 0",
            permissions = ["full".to_string()],
            readonly,
            flexible,
            comment = "Product price with validation"
    )]
    price: f64,

    // Required column with assertion
    #[column(type = "string", assert = "value != NONE")]
    sku: String,

    // Flexible column
    #[column(type = "object", flexible = true)]
    metadata: serde_json::Value,
}

impl Product {
    pub fn new(
        id: impl Into<SurrealId<Self>>,
        name: impl Into<String>,
        quantity: impl Into<i32>,
        price: impl Into<f64>,
        sku: impl Into<String>,
        metadata: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            quantity: quantity.into(),
            price: price.into(),
            sku: sku.into(),
            metadata: metadata.into(),
        }
    }
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum ProductEvents {
    #[event(
        name = "created",
        when = "var:before==NONE",
        then = "UPDATE products SET status = 'pending';
            CREATE log SET
            order = var:value.id,
            action     = 'product' + ' ' + var:event.lowercase(),
            old_status  = '',
            new_status  = var:after.status ?? 'pending',
            at         = time::now()
            "
    )]
    ProductCreated,
}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum ProductIndexes {}

#[derive(Table, Clone, Serialize, Deserialize)]
#[table(name = "users")]
pub struct User {
    #[column(type = "string")]
    id: SurrealId<Self>,

    #[column(type = "string")]
    name: String,

    #[column(type = "string")]
    email: String,
}

impl User {
    pub fn new(
        id: impl Into<SurrealId<Self>>,
        name: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            email: email.into(),
        }
    }
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum UserEvents {}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum UserIndexes {}
// Test table with all possible attributes
#[derive(Table, Serialize, Deserialize, Clone)]
#[table(
        name = "posts",
        schema = "SCHEMAFULL",
        permissions = ["full".to_string()],
        overwrite,
        comment = "Posts table with all attributes",
        changefeed = "1",
        include_original
)]
pub struct Posts {
    id: SurrealId<Self>,
    content: serde_json::Value,
}

impl Posts {
    pub fn new(id: impl Into<SurrealId<Self>>, content: impl Into<serde_json::Value>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}
#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum PostsEvents {}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum PostsIndexes {}
