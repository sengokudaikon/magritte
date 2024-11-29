use std::sync::Arc;

use anyhow::Result;
use magritte::builder::{
    base::{RangeTarget, ReturnType},
    QueryBuilder, Returns, QB,
};
use magritte_derive::SurrealTable;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::any::{connect, Any},
    Surreal,
};
use tokio::test;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealTable)]
#[table("person")]
struct Person {
    name: String,
    company: Option<String>,
    skills: Option<Vec<String>>,
}

async fn mock_db() -> Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").await?;
    db.use_db("test").await?;
    Ok(Arc::new(db))
}

#[test]
async fn test_basic_create() -> Result<()> {
    let db = mock_db().await?;

    // Basic CREATE with random ID
    let query = QB::<Person>::create(db.clone()).build()?;
    assert_eq!(query, "CREATE person;");

    // CREATE with specific ID
    let query = QB::<Person>::create(db.clone()).with_id("tobie").build()?;
    assert_eq!(query, "CREATE person:tobie;");

    // CREATE with SET clause
    let query = QB::<Person>::create(db.clone())
        .set("name", "Tobie")?
        .set("company", "SurrealDB")?
        .set("skills", vec!["Rust", "Go", "JavaScript"])?
        .build()?;
    assert_eq!(
        query,
        "CREATE person SET name = \"Tobie\", company = \"SurrealDB\", skills = [\"Rust\",\"Go\",\"JavaScript\"];"
    );

    // CREATE with CONTENT clause
    let content = Person {
        name: "Tobie".into(),
        company: Some("SurrealDB".into()),
        skills: Some(vec!["Rust".into(), "Go".into(), "JavaScript".into()]),
    };
    let query = QB::<Person>::create(db.clone()).content(content)?.build()?;
    assert_eq!(
        query,
        "CREATE person CONTENT \
         {\"name\":\"Tobie\",\"company\":\"SurrealDB\",\"skills\":[\"Rust\",\"Go\",\"JavaScript\"]};"
    );

    Ok(())
}

#[test]
async fn test_create_multiple() -> Result<()> {
    let db = mock_db().await?;

    // Create multiple records with range
    let query = QB::<Person>::create(db.clone()).range(RangeTarget::Range("1".into(), "3".into())).build()?;
    assert_eq!(query, "CREATE |person:1..3|;");

    // Create multiple records with count
    let query = QB::<Person>::create(db.clone()).range(RangeTarget::Count("3".into())).build()?;
    assert_eq!(query, "CREATE |person:3|;");

    // Create multiple records with SET
    let query = QB::<Person>::create(db.clone())
        .range(RangeTarget::Range("1".into(), "3".into()))
        .set("name", "Just a person")?
        .build()?;
    assert_eq!(query, "CREATE |person:1..3| SET name = \"Just a person\";");

    Ok(())
}

#[test]
async fn test_create_with_only() -> Result<()> {
    let db = mock_db().await?;

    // CREATE with ONLY
    let query = QB::<Person>::create(db.clone()).with_id("tobie").only().build()?;
    assert_eq!(query, "CREATE ONLY person:tobie;");

    Ok(())
}

#[test]
async fn test_create_return_values() -> Result<()> {
    let db = mock_db().await?;

    // RETURN NONE
    let query = QB::<Person>::create(db.clone()).return_(ReturnType::None).build()?;
    assert_eq!(query, "CREATE person RETURN NONE;");

    // RETURN BEFORE (same as NONE for CREATE)
    let query = QB::<Person>::create(db.clone()).return_(ReturnType::Before).build()?;
    assert_eq!(query, "CREATE person RETURN BEFORE;");

    // RETURN AFTER (default)
    let query = QB::<Person>::create(db.clone()).return_(ReturnType::After).build()?;
    assert_eq!(query, "CREATE person RETURN AFTER;");

    // RETURN DIFF
    let query = QB::<Person>::create(db.clone()).return_(ReturnType::Diff).build()?;
    assert_eq!(query, "CREATE person RETURN DIFF;");

    // RETURN specific fields
    let query = QB::<Person>::create(db.clone()).return_fields(["age", "interests"].as_ref()).build()?;
    assert_eq!(query, "CREATE person RETURN age, interests;");

    Ok(())
}

#[test]
async fn test_create_with_timeout_and_parallel() -> Result<()> {
    let db = mock_db().await?;

    // With TIMEOUT
    let query = QB::<Person>::create(db.clone()).timeout(std::time::Duration::from_millis(500)).build()?;
    assert_eq!(query, "CREATE person TIMEOUT 0;");

    // With PARALLEL
    let query = QB::<Person>::create(db.clone()).parallel().build()?;
    assert_eq!(query, "CREATE person PARALLEL;");

    // With VERSION
    let query = QB::<Person>::create(db.clone()).version("2024-01-01T00:00:00Z").build()?;
    assert_eq!(query, "CREATE person VERSION d'2024-01-01T00:00:00Z';");

    Ok(())
}
