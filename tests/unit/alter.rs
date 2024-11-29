use std::sync::Arc;

use anyhow::Result;
use magritte::builder::{operations::alter::Permission, QueryBuilder, QB};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::any::{connect, Any},
    Surreal,
};
use tokio::test;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealTable)]
#[table("user")]
struct User {
    name: String,
}

async fn mock_db() -> Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").await?;
    db.use_db("test").await?;
    Ok(Arc::new(db))
}

#[test]
async fn test_basic_alter() -> Result<()> {
    let db = mock_db().await?;

    // Basic ALTER TABLE
    let query = QB::<User>::alter(db.clone()).table("user").build()?;
    assert_eq!(query, "ALTER TABLE user");

    // ALTER TABLE with IF EXISTS
    let query = QB::<User>::alter(db.clone()).table("user").if_exists().build()?;
    assert_eq!(query, "ALTER TABLE IF EXISTS user");

    // ALTER TABLE with DROP
    let query = QB::<User>::alter(db.clone()).table("user").drop().build()?;
    assert_eq!(query, "ALTER TABLE user DROP");

    Ok(())
}

#[test]
async fn test_alter_schema_type() -> Result<()> {
    let db = mock_db().await?;

    // Set Table as SCHEMAFULL
    let query = QB::<User>::alter(db.clone()).table("user").schemafull().build()?;
    assert_eq!(query, "ALTER TABLE user SCHEMAFULL");

    // Set Table as SCHEMALESS
    let query = QB::<User>::alter(db.clone()).table("user").schemaless().build()?;
    assert_eq!(query, "ALTER TABLE user SCHEMALESS");

    Ok(())
}

#[test]
async fn test_alter_permissions() -> Result<()> {
    let db = mock_db().await?;

    // Set PERMISSIONS NONE
    let query = QB::<User>::alter(db.clone()).table("user").permissions(vec![Permission::None]).build()?;
    assert_eq!(query, "ALTER TABLE user PERMISSIONS NONE");

    // Set PERMISSIONS FULL
    let query = QB::<User>::alter(db.clone()).table("user").permissions(vec![Permission::Full]).build()?;
    assert_eq!(query, "ALTER TABLE user PERMISSIONS FULL");

    // Set specific permissions
    let query = QB::<User>::alter(db.clone())
        .table("user")
        .permissions(vec![
            Permission::Select("FULL".to_string()),
            Permission::Create("user = $auth.id".to_string()),
            Permission::Update("user = $auth.id".to_string()),
            Permission::Delete("user = $auth.id".to_string()),
        ])
        .build()?;
    assert_eq!(
        query,
        "ALTER TABLE user PERMISSIONS FOR select FULL FOR create user = $auth.id FOR update user = $auth.id FOR \
         delete user = $auth.id"
    );

    Ok(())
}

#[test]
async fn test_alter_with_comment() -> Result<()> {
    let db = mock_db().await?;

    // Add comment
    let query = QB::<User>::alter(db.clone()).table("user").comment("User Table for authentication").build()?;
    assert_eq!(query, "ALTER TABLE user COMMENT 'User Table for authentication'");

    // Complex ALTER with multiple clauses
    let query = QB::<User>::alter(db.clone())
        .table("user")
        .if_exists()
        .schemafull()
        .permissions(vec![Permission::Full])
        .comment("User Table with full permissions")
        .build()?;
    assert_eq!(
        query,
        "ALTER TABLE IF EXISTS user SCHEMAFULL PERMISSIONS FULL COMMENT 'User Table with full permissions'"
    );

    Ok(())
}
