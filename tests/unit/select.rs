use std::sync::Arc;

use anyhow::Result;
use magritte::{
    builder::{
        functions::{ArrayFunction, CountFunction},
        graph::{GraphTraversal, RecursiveDepth},
        CanCallFunctions, Lets, QueryBuilder, WhereClause, QB,
    },
    Operator, RelationType,
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
    age: i32,
}

async fn mock_db() -> Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").await?;
    db.use_db("test").await?;
    Ok(Arc::new(db))
}

#[test]
async fn test_basic_select() -> Result<()> {
    let db = mock_db().await?;

    // Select all fields
    let query = QB::<Person>::select(db.clone()).build()?;
    assert_eq!(query, "SELECT * FROM person;");

    // Select specific fields
    let query = QB::<Person>::select(db.clone()).fields(&["name", "address", "email"]).build()?;
    assert_eq!(query, "SELECT name, address, email FROM person;");

    // Select with alias
    let query = QB::<Person>::select(db.clone()).field("name.first", Some("user_name")).build()?;
    assert_eq!(query, "SELECT name.first AS user_name FROM person;");

    // Select VALUE
    let query = QB::<Person>::select(db.clone()).select_value().field("name", None).build()?;
    assert_eq!(query, "SELECT VALUE name FROM person;");
    Ok(())
}

#[test]
async fn test_edge_traversal() -> Result<()> {
    let db = mock_db().await?;

    // Basic edge traversal
    let query = QB::<Person>::select(db.clone()).traverse(RelationType::Out, "knows", "person").build()?;
    assert_eq!(query, "SELECT * FROM person->knows->person;");

    // Reverse edge traversal
    let query = QB::<Person>::select(db.clone()).traverse(RelationType::In, "knows", "person").build()?;
    assert_eq!(query, "SELECT * FROM person<-knows<-person;");

    // Both directions
    let query = QB::<Person>::select(db.clone()).traverse(RelationType::Both, "knows", "person").build()?;
    assert_eq!(query, "SELECT * FROM person<->knows<->person;");

    // Edge with conditions
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_edge_condition("since", Operator::Gt, "2020-01-01")
        .build()?;
    assert_eq!(query, "SELECT * FROM person->(knows WHERE since > $r1p0)->person;");

    // Edge with return fields
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .return_fields(vec!["strength", "since"])
        .build()?;
    assert_eq!(query, "SELECT * FROM person->knows[strength, since]->person;");

    // Multiple hops
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .traverse(RelationType::Out, "likes", "post")
        .build()?;
    assert_eq!(query, "SELECT * FROM person->knows->person->likes->post;");

    Ok(())
}

#[test]
async fn test_edge_subqueries() -> Result<()> {
    let db = mock_db().await?;

    // Edge with subquery
    let subquery = QB::<Person>::select(db.clone()).where_op("active", Operator::Eq, Some(true))?;

    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_edge_subquery(subquery)
        .build()?;
    assert_eq!(query, "SELECT * FROM person->(knows WHERE in (SELECT * FROM person WHERE active = $p0))->person;");

    Ok(())
}

#[test]
async fn test_complex_graph_traversal() -> Result<()> {
    let db = mock_db().await?;

    // Multiple relations with conditions
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_edge_condition("strength", Operator::Gt, 5)
        .traverse(RelationType::Out, "likes", "post")
        .with_edge_condition("since", Operator::Gt, "2023-01-01")
        .build()?;
    assert_eq!(
        query,
        "SELECT * FROM person->(knows WHERE strength > $r1p0)->person->(likes WHERE since > $r2p0)->post;"
    );

    // Nested subquery in edge condition
    let inner_query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "follows", "person")
        .with_edge_condition("active", Operator::Eq, true);

    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_edge_subquery(inner_query)
        .build()?;
    assert_eq!(
        query,
        "SELECT * FROM person->(knows WHERE in (SELECT * FROM person->(follows WHERE active = \
         $r1p0)->person))->person;"
    );

    // Using functions with graph traversal
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .return_fields(vec!["strength", "since"])
        .call_function(CountFunction::Count)
        .build()?;
    assert_eq!(query, "SELECT count() FROM person->knows[strength, since]->person;");

    // Multiple parallel traversals
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .traverse(RelationType::Out, "likes", "post")
        .parallel()
        .build()?;
    assert_eq!(query, "SELECT * FROM person->knows->person->likes->post PARALLEL;");

    // Return fields with array functions
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .return_fields(vec!["connections[]"])
        .call_function(ArrayFunction::Len("connections".to_string()))
        .build()?;
    assert_eq!(query, "SELECT array::len(connections) FROM person->knows[connections[]]->person;");

    Ok(())
}

#[test]
async fn test_graph_traversal_with_let() -> Result<()> {
    let db = mock_db().await?;

    // Using LET to store intermediate results
    let query = QB::<Person>::select(db.clone())
        .lets("friends", "(SELECT ->knows->person)")
        .traverse(RelationType::Out, "knows", "$friends")
        .build()?;
    assert_eq!(query, "LET $friends = (SELECT ->knows->person); SELECT * FROM person->knows->$friends;");

    // Using LET with complex subqueries
    let query = QB::<Person>::select(db.clone())
        .lets("active_friends", "(SELECT ->knows->person WHERE active = true)")
        .lets("recent_posts", "(SELECT ->likes->post WHERE created > time::now() - 24h)")
        .traverse(RelationType::Out, "knows", "$active_friends")
        .traverse(RelationType::Out, "likes", "$recent_posts")
        .build()?;
    assert_eq!(
        query,
        "LET $active_friends = (SELECT ->knows->person WHERE active = true); LET $recent_posts = (SELECT \
         ->likes->post WHERE created > time::now() - 24h); SELECT * FROM \
         person->knows->$active_friends->likes->$recent_posts;"
    );

    Ok(())
}

#[test]
async fn test_graph_traversal_with_aliases() -> Result<()> {
    let db = mock_db().await?;

    // Using AS to alias traversal results
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_alias("friends")
        .traverse(RelationType::Out, "likes", "post")
        .with_alias("liked_posts")
        .build()?;
    assert_eq!(query, "SELECT * FROM person->knows->person AS friends->likes->post AS liked_posts;");

    // Combining aliases with functions
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .with_alias("friends")
        .call_function(CountFunction::Count)
        .build()?;
    assert_eq!(query, "SELECT count() FROM person->knows->person AS friends;");

    Ok(())
}

#[test]
async fn test_recursive_traversal() -> Result<()> {
    let db = mock_db().await?;

    // Fixed depth recursive traversal
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .recursive(RecursiveDepth::Fixed(3))
        .build()?;
    assert_eq!(query, "SELECT * FROM person @.{3}->knows->person;");

    // Range depth recursive traversal
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .recursive(RecursiveDepth::Range(1, 5))
        .build()?;
    assert_eq!(query, "SELECT * FROM person @.{1..5}->knows->person;");

    // Open-ended recursive traversal
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .recursive(RecursiveDepth::OpenEnded(None))
        .build()?;
    assert_eq!(query, "SELECT * FROM person @.{..}->knows->person;");

    // Open-ended with max limit
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .recursive(RecursiveDepth::OpenEnded(Some(256)))
        .build()?;
    assert_eq!(query, "SELECT * FROM person @.{..256}->knows->person;");

    // With field collection
    let query = QB::<Person>::select(db.clone())
        .traverse(RelationType::Out, "knows", "person")
        .recursive(RecursiveDepth::Range(1, 5))
        .return_fields(vec!["id", "name", "->knows->person AS friends"])
        .build()?;
    assert_eq!(query, "SELECT * FROM person @.{1..5}.{id, name, friends: ->knows->person};");

    Ok(())
}

#[test]
async fn test_explain_and_version() -> Result<()> {
    let db = mock_db().await?;

    // Test EXPLAIN
    let query = QB::<Person>::select(db.clone()).explain(false).build()?;
    assert_eq!(query, "SELECT EXPLAIN * FROM person;");

    // Test EXPLAIN FULL
    let query = QB::<Person>::select(db.clone()).explain(true).build()?;
    assert_eq!(query, "SELECT EXPLAIN FULL * FROM person;");

    // Test VERSION
    let query = QB::<Person>::select(db.clone()).version("2024-01-01T00:00:00Z").build()?;
    assert_eq!(query, "SELECT * FROM person VERSION d'2024-01-01T00:00:00Z';");

    Ok(())
}

#[test]
async fn test_index_and_split() -> Result<()> {
    let db = mock_db().await?;

    // Test WITH INDEX
    let query = QB::<Person>::select(db.clone()).with_indexes(vec!["idx_email".to_string()]).build()?;
    assert_eq!(query, "SELECT * FROM person WITH INDEX idx_email;");

    // Test WITH NOINDEX
    let query = QB::<Person>::select(db.clone()).with_indexes(vec![]).build()?;
    assert_eq!(query, "SELECT * FROM person NOINDEX;");

    // Test SPLIT
    let query = QB::<Person>::select(db.clone()).split("tags").build()?;
    assert_eq!(query, "SELECT * FROM person SPLIT tags;");

    // Test multiple SPLIT fields
    let query = QB::<Person>::select(db.clone()).split_fields(vec!["tags", "categories"]).build()?;
    assert_eq!(query, "SELECT * FROM person SPLIT tags, categories;");

    Ok(())
}

#[test]
async fn test_range_queries() -> Result<()> {
    let db = mock_db().await?;

    // Basic range
    let query = QB::<Person>::select(db.clone()).range("1", "1000").build()?;
    assert_eq!(query, "SELECT * FROM person:1..1000;");

    Ok(())
}

#[test]
async fn test_complex_filtering() -> Result<()> {
    let db = mock_db().await?;

    // Array filtering
    let query = QB::<Person>::select(db.clone()).field_filter("address", "active = true")?.build()?;
    assert_eq!(query, "SELECT address[WHERE active = true] FROM person;");

    // Array filtering with parameterized condition
    let query = QB::<Person>::select(db.clone())
        .field_filter_with_condition("address", "active", Operator::Eq, true)?
        .build()?;
    assert_eq!(query, "SELECT address[WHERE active = $p0] FROM person;");

    // Graph edge filtering
    let query = QB::<Person>::select(db.clone()).raw("count(->experience->organisation) > 3").build()?;
    assert_eq!(query, "SELECT count(->experience->organisation) > 3 FROM person;");

    // Graph edge property filtering
    let query = QB::<Person>::select(db.clone()).raw("->(reaction WHERE type='celebrate')->post").build()?;
    assert_eq!(query, "SELECT ->(reaction WHERE type='celebrate')->post FROM person;");

    // Complex boolean logic
    let query = QB::<Person>::select(db.clone()).raw("(admin AND active) OR owner = true").build()?;
    assert_eq!(query, "SELECT (admin AND active) OR owner = true FROM person;");

    Ok(())
}
