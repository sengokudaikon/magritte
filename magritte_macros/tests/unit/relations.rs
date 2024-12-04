use magritte::prelude::*;
use serde::{Deserialize, Serialize};

// Test table with nested columns and relationships
#[derive(Table, Clone, Serialize, Deserialize)]
#[table(name = "orders")]
pub struct Order {
    #[column(type = "string")]
    id: String,

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

#[derive(Table, Clone, Serialize, Deserialize)]
#[table(name = "users")]
pub struct User {
    #[column(type = "string")]
    id: String,

    #[column(type = "string")]
    name: String,

    #[column(type = "string")]
    email: String,
}

#[derive(Table, Clone, Serialize, Deserialize)]
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
#[derive(Edge, Clone, Serialize, Deserialize)]
#[edge(name = "order_product", from = Order, to = Product, schema = "SCHEMALESS", enforced)]
pub struct OrderProduct {
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    quantity: String,
}

// Test edge between User and Order
#[derive(Edge, Clone, Serialize, Deserialize)]
#[edge(name = "user_order", from = User, to = Order, schema = "SCHEMAFULL")]
pub struct UserOrder {
    #[column(type = "datetime")]
    created_at: String,

    #[column(type = "string")]
    note: String,
}

// Test relations for Order and Product
#[derive(Relation, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderRelations {
    #[relate(
        in = "id",
        to = Product,
        out = "id",
        edge = OrderProduct,
        content = "order_product_content"
    )]
    OrderToProduct,
}

// Test relations for User and Order
#[derive(Relation,Serialize, Deserialize,strum::EnumIter)]
pub enum UserRelations {
    #[relate(
        in = "id",
        to = Order,
        out = "id",
        edge = UserOrder,
        content = "user_order_content"
    )]
    UserToOrder,
}

#[test]
fn test_order_product_relations_derive() {
    // Test OrderProductRelations enum
    let relation = OrderRelations::OrderToProduct;
    let relation_def = relation.def();
    assert_eq!(relation_def.relation_from(), "orders:id");
    assert_eq!(relation_def.relation_to(), "products:id");
    assert_eq!(relation_def.relation_name(), "order_product");
    assert_eq!(relation_def.content().unwrap(), "order_product_content");
}

#[test]
fn test_user_order_relations_derive() {
    // Test UserOrderRelations enum
    let relation = UserRelations::UserToOrder;
    let relation_def = relation.def();
    assert_eq!(relation_def.relation_from(), "users:id");
    assert_eq!(relation_def.relation_to(), "orders:id");
    assert_eq!(relation_def.relation_name(), "user_order");
    assert_eq!(relation_def.content().unwrap(), "user_order_content");
}

#[test]
fn test_relation_statements() {
    // Test OrderProductRelations statement
    let order_product_stmt = OrderRelations::OrderToProduct.to_relate_statement();
    assert!(order_product_stmt.contains("RELATE orders:id->order_product->products:id CONTENT order_product_content"));

    // Test UserOrderRelations statement
    let user_order_stmt = UserRelations::UserToOrder.to_relate_statement();
    assert!(user_order_stmt.contains("RELATE users:id->user_order->orders:id CONTENT user_order_content"));
}

impl OrderRelations {
    fn new() -> Self {
        Self::OrderToProduct
    }
}

impl UserRelations {
    fn new() -> Self {
        Self::UserToOrder
    }
}