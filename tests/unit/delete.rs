use std::sync::Arc;

use anyhow::Result;
use magritte::{
    builder::{base::ReturnType, QueryBuilder, Returns, WhereClause, QB},
    Operator,
};
use magritte_derive::SurrealTable;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::{
    engine::any::{connect, Any},
    Surreal,
};
use tokio::test;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealTable)]
#[table("person")]
struct Person {
    name: String,
    age: Option<i32>,
    interests: Option<Vec<String>>,
}

async fn mock_db() -> Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").await?;
    db.use_db("test").await?;
    Ok(Arc::new(db))
}

#[test]
async fn test_basic_delete() -> Result<()> {
    let db = mock_db().await?;

    // Delete all records from a Table
    let query = QB::<Person>::delete(db.clone()).build()?;
    assert_eq!(query, "DELETE person;");

    // Delete specific record by ID
    let query = QB::<Person>::delete(db.clone()).with_id("tobie").build()?;
    assert_eq!(query, "DELETE person:tobie;");

    // Delete with ONLY and RETURN BEFORE
    let query = QB::<Person>::delete(db.clone()).with_id("tobie").only().return_(ReturnType::Before).build()?;
    assert_eq!(query, "DELETE ONLY person:tobie RETURN BEFORE;");

    Ok(())
}

#[test]
async fn test_delete_with_conditions() -> Result<()> {
    let db = mock_db().await?;

    // Delete with simple condition
    let query = QB::<Person>::delete(db.clone()).where_op("name", Operator::Eq, Some("London"))?.build()?;
    assert_eq!(query, "DELETE person WHERE name = $p0;");

    // Delete with age condition
    let query = QB::<Person>::delete(db.clone()).where_op("age", Operator::Lt, Some(18))?.build()?;
    assert_eq!(query, "DELETE person WHERE age < $p0;");

    // Delete with array contains condition
    let query = QB::<Person>::delete(db.clone()).where_op("interests", Operator::Contains, Some("reading"))?.build()?;
    assert_eq!(query, "DELETE person WHERE interests CONTAINS $p0;");
    Ok(())
}

#[test]
async fn test_delete_return_values() -> Result<()> {
    let db = mock_db().await?;

    // Return NONE (default)
    let query = QB::<Person>::delete(db.clone()).return_(ReturnType::None).build()?;
    assert_eq!(query, "DELETE person RETURN NONE;");

    // Return BEFORE
    let query = QB::<Person>::delete(db.clone()).return_(ReturnType::Before).build()?;
    assert_eq!(query, "DELETE person RETURN BEFORE;");

    // Return AFTER
    let query = QB::<Person>::delete(db.clone()).return_(ReturnType::After).build()?;
    assert_eq!(query, "DELETE person RETURN AFTER;");

    // Return DIFF
    let query = QB::<Person>::delete(db.clone()).return_(ReturnType::Diff).build()?;
    assert_eq!(query, "DELETE person RETURN DIFF;");

    Ok(())
}

#[test]
async fn test_delete_with_timeout_and_parallel() -> Result<()> {
    let db = mock_db().await?;

    // With TIMEOUT
    let query = QB::<Person>::delete(db.clone()).timeout(std::time::Duration::from_secs(5)).build()?;
    assert_eq!(query, "DELETE person TIMEOUT 5;");

    // With PARALLEL
    let query = QB::<Person>::delete(db.clone()).parallel().build()?;
    assert_eq!(query, "DELETE person PARALLEL;");

    // Complex example with timeout and conditions
    let query = QB::<Person>::delete(db.clone())
        .where_op::<Value>("->knows->person->(knows WHERE influencer = false)", Operator::Raw, None)?
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    assert_eq!(query, "DELETE person WHERE ->knows->person->(knows WHERE influencer = false)   TIMEOUT 5;");

    Ok(())
}

#[test]
async fn test_edge_deletion() -> Result<()> {
    let db = mock_db().await?;

    // Basic edge deletion
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .to("iphone")
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought->person:iphone;");

    // Edge deletion with only from
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought;");

    // Edge deletion with only to
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .to("iphone")
        .build()?;
    assert_eq!(query, "DELETE person->bought->person:iphone;");

    // Edge deletion with WHERE condition
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .where_op("out", Operator::Eq, Some("product:iphone"))?
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought WHERE out = $p0;");

    // Edge deletion with RETURN clause
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .to("iphone")
        .return_(ReturnType::Before)
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought->person:iphone RETURN BEFORE;");

    // Edge deletion with TIMEOUT
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .to("iphone")
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought->person:iphone TIMEOUT 5;");

    // Edge deletion with PARALLEL
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .to("iphone")
        .parallel()
        .build()?;
    assert_eq!(query, "DELETE person:tobie->bought->person:iphone PARALLEL;");

    // Complex edge deletion with multiple clauses
    let query = QB::<Person>::delete(db.clone())
        .edge_of("bought")
        .from("tobie")
        .where_op("out", Operator::Eq, Some("product:iphone"))?
        .return_(ReturnType::Before)
        .timeout(std::time::Duration::from_secs(5))
        .parallel()
        .build()?;
    assert_eq!(
        query,
        "DELETE person:tobie->bought WHERE out = $p0 RETURN BEFORE TIMEOUT 5 PARALLEL;"
    );

    Ok(())
}
