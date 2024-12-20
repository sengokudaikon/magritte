use std::collections::HashMap;
use magritte::{SchemaSnapshot, TableSnapshot};
use magritte_migrations::introspection::{get_db_info, validate_migration};
use super::*;
use magritte::test_util::test_db;
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