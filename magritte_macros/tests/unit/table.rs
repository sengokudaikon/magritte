#[cfg(test)]
use magritte::prelude::*;
use serde::{Deserialize, Serialize};

// Test basic table derive
#[derive(Table, Clone, Serialize, Deserialize, Debug)]
#[table(name = "users")]
pub struct UserModel {
    pub(crate) id: String,
    pub(crate) name: String,
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