#[cfg(test)]
use magritte::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

// Test basic table derive
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
#[table(name = "users")]
pub struct UserModel {
    id: String,
    name: String,
}

impl HasId for UserModel {
    fn id(&self) -> SurrealId<Self> {
        SurrealId::new(&self.id)
    }
}

// Test table with all possible attributes
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
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
    id: String,
}

impl HasId for Posts {
    fn id(&self) -> SurrealId<Self> {
        SurrealId::new(&self.id)
    }
}

// Test table with columns and all possible column attributes
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
#[table(name = "products", schema = "SCHEMALESS")]
pub struct Product {
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
impl HasId for Product {
    fn id(&self) -> SurrealId<Self> {
        SurrealId::gen_v6()
    }
}
// Test table with as_select
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
#[table(
    name = "active_users",
    as_select = {
        from = users,
        where = active == true
    }
)]
pub struct ActiveUsers {
    #[column(ignore)]
    id: String,
    name: String,
}

// Test table with generics
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
#[table(name = "generic_items")]
pub struct GenericItem {
    #[column(type = "string")]
    id: String,
    data: RecordRef<Product>,
}

impl HasId for GenericItem {
    fn id(&self) -> SurrealId<Self> {
        SurrealId::new(&self.id)
    }
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, Display)]
pub enum EnumStatus {
    Pending,
    Shipped,
    Delivered
}

// Test table with nested columns and relationships
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
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
    #[column(value ="pending|processing|shipped|delivered")]
    status: String,
}

impl HasId for Order {
    fn id(&self) -> SurrealId<Self> {
        SurrealId::new(&self.id)
    }
}

#[test]
fn test_table_derives() {
    // Basic table
    let user = UserModel { id: "1".to_string(), name: "John".to_string() };
    assert_eq!(user.to_string(), "users");

    // Full table
    let post = Posts { id: "1".to_string() };
    assert_eq!(Posts::table_name(), "posts");
    let def = post.def();
    assert!(def.is_overwrite());
    assert_eq!(def.if_not_exists(), false);
    assert_eq!(def.is_drop(), false);
    assert_eq!(def.comment(), Some("Posts table with all attributes"));
    assert_eq!(def.permissions().len(), 1);
    assert!(def.changefeed().is_some());

    // Table with as_select
    let active_users = ActiveUsers { id: "1".to_string(), name: "John".to_string() };
    let def = active_users.def();
    assert_eq!(
        def.as_select(),
        Some("* FROM users WHERE active == true")
    );
    let product = Product {
        name: "".to_string(),
        quantity: 0,
        price: 0.0,
        sku: "".to_string(),
        metadata: Default::default(),
    };
    // Generic table
    GenericItem {
        id: "1".to_string(),
        data: product.as_record()
    };
    assert_eq!(
        GenericItem::table_name(),
        "generic_items"
    );
}

#[test]
fn test_column_derives() {
    // Test generated column enum
    assert_eq!(ProductColumns::Name.to_string(), "name");
    assert_eq!(ProductColumns::Quantity.to_string(), "quantity");
    assert_eq!(ProductColumns::Price.to_string(), "price");
    assert_eq!(ProductColumns::Sku.to_string(), "sku");
    assert_eq!(ProductColumns::Metadata.to_string(), "metadata");

    // Test column definitions
    let name_def = ProductColumns::Name.def();
    assert_eq!(name_def.name(), "name");
    assert_eq!(name_def.table_name(), "products");
    assert_eq!(name_def.column_type(), "string");
    assert!(!name_def.is_nullable());

    let price_def = ProductColumns::Price.def();
    assert!(price_def.is_nullable());
    assert!(price_def.is_readonly());
    assert!(price_def.is_flexible());
    assert_eq!(price_def.default(), Some("0.0"));
    assert_eq!(price_def.assert(), Some("value >= 0"));
    assert_eq!(price_def.comment(), Some("Product price with validation"));
    assert_eq!(price_def.permissions().len(), 1);

    let metadata_def = ProductColumns::Metadata.def();
    assert!(metadata_def.is_flexible());
    assert_eq!(metadata_def.column_type(), "object");
}

#[test]
fn test_complex_column_derives() {
    // Test relationship column definitions
    let user_def = OrderColumns::User.def();
    assert_eq!(user_def.column_type(), "record<users>");
    assert!(user_def.assert().is_some());
    assert!(!user_def.is_nullable());

    // Test array relationship column
    let items_def = OrderColumns::Items.def();
    assert_eq!(items_def.column_type(), "array<record<products>>");
    
    // Test enum-like value constraints
    let status_def = OrderColumns::Status.def();
    assert_eq!(status_def.column_type(), "string");
    let status_stmt = status_def.to_statement();
    assert!(status_stmt.contains("VALUE"));
    assert!(status_stmt.contains("pending"));
    assert!(status_stmt.contains("delivered"));

    // Test nested object column
    let shipping_def = OrderColumns::ShippingInfo.def();
    assert_eq!(shipping_def.column_type(), "object");
    assert!(shipping_def.is_flexible());

    // Test decimal type column
    let total_def = OrderColumns::Total.def();
    assert_eq!(total_def.column_type(), "decimal");
    assert_eq!(total_def.assert(), Some("value >= 0"));

    // Test datetime column
    let created_def = OrderColumns::CreatedAt.def();
    assert_eq!(created_def.column_type(), "datetime");
}

#[test]
fn test_statement_generation() {
    // Test basic column statement
    let name_stmt = ProductColumns::Name.def().to_statement();
    assert!(name_stmt.contains("DEFINE FIELD name ON TABLE products"));
    assert!(name_stmt.contains("TYPE string"));

    // Test full column statement
    let price_stmt = ProductColumns::Price.def().to_statement();
    assert!(price_stmt.contains("DEFINE FIELD price ON TABLE products"));
    assert!(price_stmt.contains("FLEXIBLE"));
    assert!(price_stmt.contains("TYPE float|null"));
    assert!(price_stmt.contains("DEFAULT 0.0"));
    assert!(price_stmt.contains("ASSERT value >= 0"));
    assert!(price_stmt.contains("PERMISSIONS"));
    assert!(price_stmt.contains("READONLY"));
    assert!(price_stmt.contains("COMMENT"));
}

#[test]
fn test_column_statement_variations() {
    // Test relationship field statement
    let user_stmt = OrderColumns::User.def().to_statement();
    assert!(user_stmt.contains("DEFINE FIELD user ON TABLE orders"));
    assert!(user_stmt.contains("TYPE record<users>"));
    assert!(user_stmt.contains("ASSERT value != NONE"));

    // Test array relationship statement
    let items_stmt = OrderColumns::Items.def().to_statement();
    assert!(items_stmt.contains("DEFINE FIELD items ON TABLE orders"));
    assert!(items_stmt.contains("TYPE array<record<products>>"));

    // Test enum-like value constraint statement
    let status_stmt = OrderColumns::Status.def().to_statement();
    assert!(status_stmt.contains("DEFINE FIELD status ON TABLE orders"));
    let expected_value = "pending|processing|shipped|delivered";
    assert!(status_stmt.contains(format!("VALUE {}",expected_value).as_str()));
}
