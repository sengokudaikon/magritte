use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use pretty_assertions::assert_eq;
use magritte::*;
use super::User;
use super::Product;
use super::Order;
use super::edge::OrderProduct;
use super::edge::UserOrder;
// Test relations for Order and Product
#[derive(Relation, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderRelations {
    #[relate(
        to = Product,
        edge = OrderProduct,
        content = "order_product_content"
    )]
    OrderToProduct,
}

// Test relations for User and Order
#[derive(Relation,Serialize, Deserialize,strum::EnumIter)]
pub enum UserRelations {
    #[relate(
        to = Order,
        edge = UserOrder,
        content = "user_order_content",
        eager
    )]
    UserToOrder,
}

#[test]
fn test_order_product_relations_derive() {
    // Test OrderProductRelations enum
    let relation_def = OrderRelations::OrderToProduct.relation_def();
    assert_eq!(relation_def.relation_from(), "orders:id");
    assert_eq!(relation_def.relation_to(), "products:id");
    assert_eq!(relation_def.relation_name(), "order_product");
    assert_eq!(relation_def.content().unwrap(), "order_product_content");
}

#[test]
fn test_user_order_relations_derive() {
    // Test UserOrderRelations enum
    let relation_def =  UserRelations::UserToOrder.relation_def();
    assert_eq!(relation_def.relation_from(), "users:id");
    assert_eq!(relation_def.relation_to(), "orders:id");
    assert_eq!(relation_def.relation_name(), "user_order");
    assert_eq!(relation_def.content().unwrap(), "user_order_content");
}

#[test]
fn test_relation_statements() -> anyhow::Result<()> {
    // Test OrderProductRelations statement
    let order_product_stmt = OrderRelations::OrderToProduct.relation_def().relate("id", "id2")?.build().map_err(anyhow::Error::from)?;
    assert!(order_product_stmt.contains("RELATE orders:id->order_product->products:id2 CONTENT order_product_content"));

    // Test UserOrderRelations statement
    let user_order_stmt = UserRelations::UserToOrder.relation_def().relate("id", "id2")?.build()?;
    assert!(user_order_stmt.contains("RELATE users:id->user_order->orders:id2 CONTENT user_order_content"));
    Ok(())
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