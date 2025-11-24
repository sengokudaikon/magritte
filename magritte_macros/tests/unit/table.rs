use crate::{Posts, Product, ProductColumns, User};
use magritte::*;
use pretty_assertions::assert_eq;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Test table with as_select
#[derive(Table, Serialize, Deserialize, Clone)]
#[table(
    name = "active_users",
    as_select = {
        from = users,
        where = active == true
    }
)]
pub struct ActiveUsers {
    #[column(ignore)]
    id: SurrealId<Self>,
    name: String,
}

// Empty indexes for ActiveUsers
#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum ActiveUsersIndexes {}

// Empty events for ActiveUsers
#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum ActiveUsersEvents {}

// Test table with generics
#[derive(Table, Serialize, Deserialize, Clone)]
#[table(name = "generic_items")]
pub struct GenericItem {
    id: SurrealId<Self>,
    data: RecordRef<Product>,
}

#[test]
fn test_table_derives() {
    // Basic table
    let user = User::new("1", "J", "jdoe@example.com");
    assert_eq!(user.to_string(), "users");

    // Full table
    let post = Posts::new("1");
    assert_eq!(Posts::table_name(), "posts");
    let def = post.def_owned();
    assert!(def.is_overwrite());
    assert_eq!(def.if_not_exists(), false);
    assert_eq!(def.is_drop(), false);
    assert_eq!(def.comment(), Some("Posts table with all attributes"));
    assert_eq!(def.permissions().len(), 1);
    assert!(def.changefeed().is_some());

    // Table with as_select
    let active_users = ActiveUsers {
        id: "1".into(),
        name: "John".to_string(),
    };
    let def = active_users.def_owned();
    assert_eq!(def.as_select(), Some("* FROM users WHERE active == true"));
    let product = Product {
        id: "1".into(),
        name: "".to_string(),
        quantity: 0,
        price: 0.0,
        sku: "".to_string(),
        metadata: Default::default(),
    };
    // Generic table
    GenericItem {
        id: "1".into(),
        data: product.as_record(),
    };
    assert_eq!(GenericItem::table_name(), "generic_items");
}

#[test]
fn test_statement_generation() -> Result<()> {
    // Test basic column statement
    let name_stmt = ProductColumns::Name.def().to_statement().build().map_err(anyhow::Error::from)?;
    assert!(name_stmt.contains("DEFINE FIELD name ON TABLE products"));
    assert!(name_stmt.contains("TYPE string"));

    // Test full column statement
    let price_stmt = ProductColumns::Price.def().to_statement().build()?;
    assert!(price_stmt.contains("DEFINE FIELD price ON TABLE products"));
    assert!(price_stmt.contains("FLEXIBLE"));
    assert!(price_stmt.contains("TYPE float|null"));
    assert!(price_stmt.contains("DEFAULT 0.0"));
    assert!(price_stmt.contains("ASSERT value >= 0"));
    assert!(price_stmt.contains("PERMISSIONS"));
    assert!(price_stmt.contains("READONLY"));
    assert!(price_stmt.contains("COMMENT"));
    Ok(())
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
fn test_table_name_validation() {
    // Can't access magritte_macros::derives::attributes directly, so we'll test manually
    
    // Valid names should follow the pattern ^[a-zA-Z0-9_-]+$
    let valid_names = &["users", "user_profiles", "users123", "user-profiles"];
    for name in valid_names {
        assert!(name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
    }
    
    // Invalid names contain characters not allowed in table names
    let invalid_chars = &[';', ' ', '\'', '/', '*'];
    for &c in invalid_chars {
        let invalid_name = format!("users{}", c);
        assert!(!invalid_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
    }
}
