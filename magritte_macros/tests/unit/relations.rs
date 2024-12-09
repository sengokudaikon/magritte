use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use pretty_assertions::assert_eq;
use magritte::RelationTrait;
use magritte_macros::Relation;

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
fn test_relation_statements() -> anyhow::Result<()> {
    // Test OrderProductRelations statement
    let order_product_stmt = OrderRelations::OrderToProduct.to_statement()?.build().map_err(anyhow::Error::from)?;
    assert!(order_product_stmt.contains("RELATE orders:id->order_product->products:id CONTENT order_product_content"));

    // Test UserOrderRelations statement
    let user_order_stmt = UserRelations::UserToOrder.to_statement()?.build()?;
    assert!(user_order_stmt.contains("RELATE users:id->user_order->orders:id CONTENT user_order_content"));
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