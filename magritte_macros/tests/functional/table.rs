use crate::User;
use magritte::entity_crud::{BasicCrud, SurrealCrud};
use magritte::test_util::test_db;
use magritte::{ColumnTrait, TableTrait};
use magritte_query::types::HasId;
use magritte_query::SurrealId;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_create_and_query_records() -> anyhow::Result<()> {
    let db = test_db().await?;
    let stmt = User::to_statement().build().map_err(anyhow::Error::from)?;
    println!("{}", stmt.as_str());

    db.query(stmt).await?.check()?;
    for col in User::columns() {
        let stmt = ColumnTrait::to_statement(&col)
            .build()
            .map_err(anyhow::Error::from)?;
        println!("{}", stmt.as_str());
        db.query(stmt).await?.check()?;
    }
    // Create User records
    let user1 = User::new("Alice", "Alice".to_string(), "alice@me.com".to_string());
    let user_stmt = user1.clone().insert()?.build()?;
    println!("{}", user_stmt.as_str());

    let result = user1
        .insert()?
        .execute(db.clone())
        .await
        .map_err(anyhow::Error::from)?;
    assert_eq!(result.len(), 1);
    let alice = result.first();
    assert!(alice.is_some());
    let alice = alice.unwrap();
    assert_eq!(alice.id().to_string(), "users:Alice");
    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.email, "alice@me.com");

    let result = User::find_by_id(SurrealId::from("Alice"))?
        .execute(db.clone())
        .await
        .map_err(anyhow::Error::from)?;
    let user2: Option<&User> = result.first();
    assert!(user2.is_some());
    let user2 = user2.unwrap();
    assert_eq!(user2.id().to_string(), "users:Alice");
    assert_eq!(user2.name, "Alice");
    assert_eq!(user2.email, "alice@me.com");

    Ok(())
}
