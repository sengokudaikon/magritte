use super::Order;
use super::Product;
use super::User;
use anyhow::Result;
use magritte::*;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

// Test edge between Order and Product
#[derive(Edge, Serialize, Deserialize, Clone)]
#[edge(name = "order_product", from = Order, to = Product, schema = "SCHEMALESS", enforced)]
pub struct OrderProduct {
    id: SurrealId<Self>,
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    quantity: String,
}

impl HasId for OrderProduct {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

// Test edge between User and Order
#[derive(Edge, Serialize, Deserialize, Clone)]
#[edge(from = User, to = Order, if_not_exists)]
pub struct UserOrder {
    id: SurrealId<Self>,
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    note: String,
}

impl HasId for UserOrder {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

#[test]
fn test_order_product_edge_derive() {
    // Test OrderProductEdge
    let edge = OrderProduct {
        id: "1".into(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        quantity: "1".to_string(),
    };

    let edge_def = edge.def_owned();
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
        id: "1".into(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        note: "Special order".to_string(),
    };

    let edge_def = edge.def_owned();
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
fn test_edge_statements() -> Result<()> {
    // Test OrderProductEdge statement
    let order_product_edge_stmt = OrderProduct::new()
        .to_statement_owned()
        .build()
        .map_err(anyhow::Error::from)?;
    println!("{}", order_product_edge_stmt);
    assert!(order_product_edge_stmt.contains(
        "DEFINE TABLE order_product TYPE RELATION SCHEMALESS FROM orders TO products ENFORCED"
    ));

    // Test UserOrderEdge statement
    let user_order_edge_stmt = UserOrder::new().to_statement_owned().build()?;
    assert!(user_order_edge_stmt.contains(
        "DEFINE TABLE IF NOT EXISTS user_order TYPE RELATION SCHEMAFULL FROM users TO orders"
    ));
    Ok(())
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
            id: "1".into(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            quantity: "1".to_string(),
        }
    }
}

impl UserOrder {
    fn new() -> Self {
        Self {
            id: "1".into(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            note: "Special order".to_string(),
        }
    }
}
