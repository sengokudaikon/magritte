use magritte::prelude::*;
use serde::{Deserialize, Serialize};

// Test table with nested columns and relationships
#[derive(Table, Serialize, Deserialize, Clone)]
#[table(name = "orders")]
pub struct Order {
    #[column(type = "string")]
    id: String,

    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "record<users>", assert = "value != NONE")]
    user: RecordRef<UserModel>,

    #[column(type = "array<record<products>>")]
    items: Vec<RecordRef<Product>>,

    #[column(type = "decimal", assert = "value >= 0")]
    total: f64,

    #[column(type = "object", flexible)]
    shipping_info: serde_json::Value,
    #[column(value = "pending|processing|shipped|delivered")]
    status: String,
}

#[derive(Table, Serialize, Deserialize,  Clone)]
#[table(name = "users")]
pub struct UserModel {
    #[column(type = "string")]
    id: String,

    #[column(type = "string")]
    name: String,

    #[column(type = "string")]
    email: String,
}

#[derive(Table, Serialize, Deserialize,  Clone)]
#[table(name = "products")]
pub struct Product {
    #[column(type = "string")]
    id: String,

    #[column(type = "string")]
    name: String,

    #[column(type = "decimal", assert = "value >= 0")]
    price: f64,
}

// Test edge between Order and Product
#[derive(Edge, Serialize, Deserialize,  Clone)]
#[edge(name = "order_product", from = Order, to = Product, schema = "SCHEMALESS", enforced = true)]
pub struct OrderProduct {
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    quantity: String,
}

// Test edge between User and Order
#[derive(Edge, Serialize, Deserialize,  Clone)]
#[edge(from = UserModel, to = Order, if_not_exists)]
pub struct UserOrder {
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    note: String,
}

#[test]
fn test_order_product_edge_derive() {
    // Test OrderProductEdge
    let edge = OrderProduct {
        created_at: "2023-01-01T00:00:00Z".to_string(),
        quantity: "1".to_string(),
    };

    let edge_def = edge.def();
    assert_eq!(edge_def.edge_name(), "order_product");
    assert_eq!(edge_def.schema_type().to_string(), "SCHEMALESS");
    assert_eq!(edge_def.permissions().len(), 0);
    assert_eq!(edge_def.edge_from(), "orders");
    assert_eq!(edge_def.edge_to(), "products");
    assert_eq!(edge_def.is_enforced(), true);
    assert_eq!(edge_def.is_overwrite(), false);
    assert_eq!(edge_def.is_drop(), false);
    assert_eq!(edge_def.if_not_exists(), false);
}

#[test]
fn test_user_order_edge_derive() {
    // Test UserOrderEdge
    let edge = UserOrder {
        created_at: "2023-01-01T00:00:00Z".to_string(),
        note: "Special order".to_string(),
    };

    let edge_def = edge.def();
    assert_eq!(edge_def.edge_name(), "user_order");
    assert_eq!(edge_def.schema_type().to_string(), "SCHEMAFULL");
    assert_eq!(edge_def.permissions().len(), 0);
    assert_eq!(edge_def.edge_from(), "users");
    assert_eq!(edge_def.edge_to(), "orders");
    assert_eq!(edge_def.is_enforced(), false);
    assert_eq!(edge_def.is_overwrite(), false);
    assert_eq!(edge_def.is_drop(), false);
    assert_eq!(edge_def.if_not_exists(), true);
}

#[test]
fn test_edge_statements() {
    // Test OrderProductEdge statement
    let order_product_edge_stmt = OrderProduct::new().to_statement();
    assert!(order_product_edge_stmt.contains("DEFINE TABLE order_product SCHEMALESS TYPE RELATION FROM orders TO products ENFORCED"));

    // Test UserOrderEdge statement
    let user_order_edge_stmt = UserOrder::new().to_statement();
    assert!(user_order_edge_stmt.contains("DEFINE TABLE IF NOT EXISTS user_order SCHEMAFULL TYPE RELATION FROM users TO orders"));
}

#[test]
fn test_edge_columns() {
    // Test OrderProductEdge columns
    let created_at_def = OrderProductColumns::CreatedAt.def();
    assert_eq!(created_at_def.column_type(), "datetime");
    assert!(created_at_def.assert().is_none());
    assert!(!created_at_def.is_nullable());

    let quantity_def = OrderProductColumns::Quantity.def();
    assert_eq!(quantity_def.column_type(), "string");
    assert!(quantity_def.assert().is_none());
    assert!(!quantity_def.is_nullable());

    // Test UserOrderEdge columns
    let created_at_def = UserOrderColumns::CreatedAt.def();
    assert_eq!(created_at_def.column_type(), "datetime");
    assert!(created_at_def.assert().is_none());
    assert!(!created_at_def.is_nullable());

    let note_def = UserOrderColumns::Note.def();
    assert_eq!(note_def.column_type(), "string");
    assert!(note_def.assert().is_none());
    assert!(!note_def.is_nullable());
}

impl OrderProduct {
    fn new() -> Self {
        Self {
            created_at: "2023-01-01T00:00:00Z".to_string(),
            quantity: "1".to_string(),
        }
    }
}

impl UserOrder {
    fn new() -> Self {
        Self {
            created_at: "2023-01-01T00:00:00Z".to_string(),
            note: "Special order".to_string(),
        }
    }
}
