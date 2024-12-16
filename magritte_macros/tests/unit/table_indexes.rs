use magritte::*;
use serde::{Deserialize, Serialize};
use pretty_assertions::assert_eq;
use super::Product;
use super::User;

// Test index for Product table
#[derive(Index, Serialize, Deserialize,strum::EnumIter)]
pub enum ProductIndexes {
    #[index(
        name = "price_idx",
        fields = [price],
        comment = "Index on product price"
    )]
    PriceIdx,

    #[index(
        name = "name_idx",
        columns = [name],
        unique = true,
        concurrently = true
    )]
    NameIdx,
}

// Test index for UserModel table
#[derive(Index, Serialize, Deserialize,strum::EnumIter)]
pub enum UserIndexes {
    #[index(
        name = "email_idx",
        columns = [email],
        unique,
        comment = "Unique index on user email"
    )]
    EmailIdx,

    #[index(
        name = "name_idx",
        fields = [name],
        if_not_exists,
        comment = "IF NOT EXISTS index on user name"
    )]
    NameIdx,
}

#[test]
fn test_product_indexes_derive() {
    // Test PriceIdx
    let price_idx = ProductIndexes::PriceIdx;
    assert_eq!(ProductIndexes::table_name(), "products");
    assert_eq!(price_idx.index_name(), "price_idx");
    assert_eq!(price_idx.def().fields(), Some(vec!["price"]));
    assert_eq!(price_idx.def().columns(), None);
    assert_eq!(price_idx.def().is_unique(), false);
    assert_eq!(price_idx.def().comment(), Some("Index on product price"));
    assert_eq!(price_idx.def().is_concurrent(), false);

    // Test NameIdx
    let name_idx = ProductIndexes::NameIdx;
    assert_eq!(ProductIndexes::table_name(), "products");
    assert_eq!(name_idx.index_name(), "name_idx");
    assert_eq!(name_idx.def().fields(), None);
    assert_eq!(name_idx.def().columns(), Some(vec!["name"]));
    assert_eq!(name_idx.def().is_unique(), true);
    assert_eq!(name_idx.def().comment(), None);
    assert_eq!(name_idx.def().is_concurrent(), true);
}

#[test]
fn test_user_model_indexes_derive() {
    // Test EmailIdx
    let email_idx = UserIndexes::EmailIdx;
    assert_eq!(UserIndexes::table_name(), "users");
    assert_eq!(email_idx.index_name(), "email_idx");
    assert_eq!(email_idx.def().fields(), None);
    assert_eq!(email_idx.def().columns(), Some(vec!["email"]));
    assert_eq!(email_idx.def().is_unique(), true);
    assert_eq!(email_idx.def().comment(), Some("Unique index on user email"));
    assert_eq!(email_idx.def().is_concurrent(), false);

    // Test NameIdx
    let name_idx = UserIndexes::NameIdx;
    assert_eq!(UserIndexes::table_name(), "users");
    assert_eq!(name_idx.index_name(), "name_idx");
    assert_eq!(name_idx.def().fields(), Some(vec!["name"]));
    assert_eq!(name_idx.def().columns(), None);
    assert_eq!(name_idx.def().is_unique(), false);
    assert_eq!(name_idx.def().comment(), Some("IF NOT EXISTS index on user name"));
    assert_eq!(name_idx.def().is_concurrent(), false);
    assert_eq!(name_idx.def().if_not_exists(), true);
}

#[test]
fn test_index_statements() {
    // Test PriceIdx statement
    let price_idx_stmt = match ProductIndexes::PriceIdx.to_statement() {
        Ok(stmt) => {
            println!("{}", stmt);
            stmt
        },
        Err(e) => {
            eprintln!("Failed to get statement: {}", e);
            "".to_string()
        },
    };

    assert!(price_idx_stmt.contains("DEFINE INDEX price_idx ON products"));
    assert!(price_idx_stmt.contains("FIELDS price"));
    assert!(!price_idx_stmt.contains("UNIQUE"));
    assert!(price_idx_stmt.contains("COMMENT \"Index on product price\""));
    assert!(!price_idx_stmt.contains("CONCURRENTLY"));

    // Test NameIdx statement
    let name_idx_stmt = match ProductIndexes::NameIdx.to_statement() {
        Ok(stmt) => {
            println!("{}", stmt);
            stmt
        },
        Err(e) => {
            eprintln!("Failed to get statement: {}", e);
            "".to_string()
        },
    };
    assert!(name_idx_stmt.contains("DEFINE INDEX name_idx ON products"));
    assert!(name_idx_stmt.contains("COLUMNS name"));
    assert!(name_idx_stmt.contains("UNIQUE"));
    assert!(!name_idx_stmt.contains("COMMENT"));
    assert!(name_idx_stmt.contains("CONCURRENTLY"));

    // Test EmailIdx statement
    let email_idx_stmt = match UserIndexes::EmailIdx.to_statement() {
        Ok(stmt) => {
            println!("{}", stmt);
            stmt
        },
        Err(e) => {
            eprintln!("Failed to get statement: {}", e);
            "".to_string()
        },
    };
    assert!(email_idx_stmt.contains("DEFINE INDEX email_idx ON users"));
    assert!(email_idx_stmt.contains("COLUMNS email"));
    assert!(email_idx_stmt.contains("UNIQUE"));
    assert!(email_idx_stmt.contains("COMMENT \"Unique index on user email\""));
    assert!(!email_idx_stmt.contains("CONCURRENTLY"));

    // Test NameIdx statement
    let name_idx_stmt = match UserIndexes::NameIdx.to_statement() {
        Ok(stmt) => {
            println!("{}", stmt);
            stmt
        },
        Err(e) => {
            eprintln!("Failed to get statement: {}", e);
            "".to_string()
        },
    };
    assert!(name_idx_stmt.contains("DEFINE INDEX IF NOT EXISTS name_idx ON users"));
    assert!(name_idx_stmt.contains("FIELDS name"));
    assert!(!name_idx_stmt.contains("UNIQUE"));
    assert!(name_idx_stmt.contains("COMMENT \"IF NOT EXISTS index on user name\""));
    assert!(!name_idx_stmt.contains("CONCURRENTLY"));
}

#[test]
fn test_index_enum_iteration() {
    // Test ProductIndexes iteration
    let product_indexes: Vec<_> = Product::indexes().collect();
    assert_eq!(product_indexes.len(), 2);
    assert!(product_indexes.contains(&ProductIndexes::PriceIdx));
    assert!(product_indexes.contains(&ProductIndexes::NameIdx));

    // Test UserIndexes iteration
    let user_model_indexes: Vec<_> = User::indexes().collect();
    assert_eq!(user_model_indexes.len(), 2);
    assert!(user_model_indexes.contains(&UserIndexes::EmailIdx));
    assert!(user_model_indexes.contains(&UserIndexes::NameIdx));
}