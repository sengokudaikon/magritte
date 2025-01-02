use magritte::{Query, SchemaSnapshot, Snapshot, SurrealDB, TableSnapshot, TableTrait};
use magritte_migrations::introspection::{get_db_info, validate_migration};
use magritte_migrations::test_models::UserV1;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;

async fn test_db() -> anyhow::Result<SurrealDB> {
    let db: Surreal<Any> = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}
#[tokio::test]
async fn test_get_db_info() -> anyhow::Result<()>{
    let db = test_db().await?;
    let info = get_db_info(db).await?;
    assert!(info.tables.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_validation_with_empty_db() -> anyhow::Result<()>{
    let db = test_db().await?;
    let mut expected = SchemaSnapshot::new();
    let mut table = TableSnapshot {
        name: "test".to_string(),
        define_table_statement: "".to_string(),
        fields: HashMap::new(),
        indexes: HashMap::new(),
        events: HashMap::new(),
    };
    table.define_table_statement = "DEFINE TABLE test SCHEMALESS".to_string();
    expected.add_table(table);

    let report = validate_migration(db, &expected).await?;
    assert!(report.has_issues());
    assert_eq!(report.missing.len(), 1);
    assert!(report.missing[0].contains("test"));
    Ok(())
}

#[tokio::test]
async fn test_transaction_execution() -> anyhow::Result<()> {
    let db = test_db().await?;
    
    // Setup initial state with UserV1
    let table_stmt = <UserV1 as TableTrait>::to_statement().build().map_err(anyhow::Error::from)?;
    Query::begin().raw(&table_stmt).commit().execute(&db).await?;
    
    // Try to apply invalid migration (should rollback)
    let mut transaction = Query::begin();
    transaction = transaction
        .raw("DEFINE FIELD invalid ON users TYPE string".into())
        .raw("THIS IS INVALID SQL");
    
    // Should fail and rollback
    let result = transaction.commit().execute(&db).await;
    assert!(result.is_err(), "Invalid transaction should fail");
    
    // Verify DB state is unchanged
    let info = Query::info(db.clone()).info_table("users").await?;
    assert!(!info.fields.contains_key("invalid"), "Invalid field should not exist");
    
    Ok(())
}

#[tokio::test]
async fn test_validation_with_events_and_indexes() -> anyhow::Result<()> {
    let db = test_db().await?;
    let qb = Query::begin();
    // Create table with event and index
    qb
        .raw("DEFINE TABLE test SCHEMALESS")
        .raw("DEFINE INDEX idx_test ON test FIELDS name")
        .raw("DEFINE EVENT evt_test ON test WHEN $event = 'CREATE' THEN CREATE log:entry")
        .commit()
        .execute(&db.clone()).await.map_err(anyhow::Error::from)?;
    
    // Create expected schema
    let mut expected = SchemaSnapshot::new();
    let mut table = TableSnapshot::new("test".into(), "DEFINE TABLE test TYPE ANY SCHEMALESS PERMISSIONS NONE".into());
    table.add_index("idx_test".into(), "DEFINE INDEX idx_test ON test FIELDS name".into());
    table.add_event("evt_test".into(), "DEFINE EVENT evt_test ON test WHEN $event = 'CREATE' THEN (CREATE log:entry)".into());
    expected.add_table(table);
    
    // Validate
    let report = validate_migration(db, &expected).await?;
    println!("{}", report.missing.join("\n"));
    println!("{}", report.mismatches.join("\n"));
    println!("{}", report.unexpected.join("\n"));
    assert!(!report.has_issues(), "Validation should pass");
    
    Ok(())
}