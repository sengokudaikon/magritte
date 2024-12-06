use std::sync::Arc;
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;

use magritte_query::types::{HasId, SurrealId};
use pretty_assertions::assert_eq;
use magritte::entity_crud::SurrealCrud;
use magritte::migration::MigrationDef;
use magritte::prelude::TableTrait;
use magritte_query::QueryBuilder;
use crate::{Posts, User};

pub async fn init_db() -> anyhow::Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}

#[tokio::test]
async fn test_create_and_query_records() -> anyhow::Result<()> {
    let db = init_db().await?;
    let stmt = User::up();
    let res = db.query(stmt).await?.check();
    assert!(res.is_ok());
    // Create User records
    let user1 = User::new("Alice", "Alice".to_string(), "alice@me.com".to_string());

    println!("{}", stmt);
    let id = user1.id.clone();
    let alice = user1.insert_by_id(id)?.execute(db.clone()).await.map_err(|e|e.into())?.take_first();
    assert!(alice.is_some());
    let alice = alice.unwrap();
    assert_eq!(alice.id().to_string(), "users:Alice");
    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.email, "alice@me.com");

    let user2:Option<User> = db.query("SELECT * FROM users").await?.take(0)?;
    assert!(user2.is_some());
    let user2 = user2.unwrap();
    assert_eq!(user2.id().to_string(), "users:Alice");
    assert_eq!(user2.name, "Alice");
    assert_eq!(user2.email, "alice@me.com");

    Ok(())
}

