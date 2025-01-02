use anyhow::bail;
use magritte::{ColumnTrait, Query, SchemaSnapshot, SurrealDB, TableTrait};
use magritte_migrations::manager::MigrationManager;
use magritte_migrations::test_models::{UserV1, UserV2};
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;
use tempfile::tempdir;
async fn test_db() -> anyhow::Result<SurrealDB> {
    let db: Surreal<Any> = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}
#[tokio::test]
async fn test_full_migration_flow() -> anyhow::Result<()> {
    let db = test_db().await?;
    let temp_dir = tempdir()?;
    let manager = MigrationManager::new(temp_dir.path().to_path_buf());
    
    // Create initial schema in DB (UserV1)
    let table_stmt = <UserV1 as TableTrait>::to_statement().build().map_err(anyhow::Error::from)?;
    let mut qb = Query::begin();
    qb = qb.raw(&table_stmt);
    for col in <UserV1 as TableTrait>::columns() {
        let field_stmt = ColumnTrait::to_statement(&col).build()?;
        qb = qb.raw(&field_stmt)
    }
    qb.commit().execute(&db.clone()).await?;
    
    // Get current schema from code (UserV2)
    let code_snapshot = manager.current_schema_from_code()?;
    // Generate migration considering DB state
    let validated_snapshot = manager.generate_migration_with_db_check(
        db.clone(),
        &SchemaSnapshot::new(), // Empty snapshot since this is first migration
        &code_snapshot,
    ).await?;
    // Create migration files
    let migration_name = manager.create_empty_migration()?;
    
    // Apply migration
    let res = manager.apply_migration(&db, &migration_name).await;
    match res {
        Ok(_) => {
            println!("Migration applied successfully");
        },
        Err(e) => {
            println!("Migration failed: {}", e);
            bail!("Migration failed");
        }
    }

    // since User is already defined, it won't autogenerate a overwrite diff, so we have to do it manually
    let userv2 = <UserV2 as TableTrait>::to_statement().build().map_err(anyhow::Error::from)?;
    let mut qb = Query::begin();
    qb = qb.raw(&userv2);
    for col in <UserV2 as TableTrait>::columns() {
        let field_stmt = ColumnTrait::to_statement(&col).overwrite().build()?;
        qb = qb.raw(&field_stmt)
    }
    let intermediary = qb.clone().commit().build();
    println!("Before second: {}", intermediary);
    let res = qb.commit().execute(&db.clone()).await;
    match res {
        Ok(_) => {
            println!("Second Migration applied successfully");
        },
        Err(e) => {
            println!("Migration failed: {}", e);
            bail!("Migration failed");
        }
    }
    
    // Verify final state
    let info = Query::info(db.clone()).info_table("users").await?;
    println!("{}", serde_json::to_string_pretty(&info)?);
    assert!(info.fields.contains_key("email"), "Email field should exist");
    
    Ok(())
}