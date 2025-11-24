use magritte::{
    table_snapshot, ColumnTrait, Event, HasId, Index, SchemaSnapshot, SurrealId, Table,
    TableRegistration, TableSnapshot, TableTrait,
};
use magritte_migrations::manager::MigrationManager;
use magritte_migrations::table::TableDiff;
use magritte_migrations::test_models::{UserV1, UserV2};
use magritte_migrations::Diff;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[tokio::test]
async fn test_generate_diff_from_snapshots() -> anyhow::Result<()> {
    let manager = MigrationManager::new(tempdir()?.path().into());
    // 1. Create a snapshot from UserV1 schema
    let old_schema = {
        let mut schema = SchemaSnapshot::new();
        let define_table = <UserV1 as TableTrait>::to_statement()
            .build()
            .map_err(anyhow::Error::from)?;
        println!("Old: {}", define_table);
        let mut fields = std::collections::HashMap::new();
        for col in <UserV1 as TableTrait>::columns() {
            let def = ColumnTrait::def(&col);
            let field_def = ColumnTrait::to_statement(&col).build()?;
            println!("{}", field_def);
            fields.insert(def.name().to_string(), field_def);
        }
        let table_snap = TableSnapshot {
            name: "users".into(),
            define_table_statement: define_table,
            fields,
            indexes: Default::default(),
            events: Default::default(),
        };
        schema.add_table(table_snap);
        schema
    };

    // 2. Create a snapshot from UserV2 schema
    let new_schema = {
        let mut schema = SchemaSnapshot::new();
        let define_table = <UserV2 as TableTrait>::to_statement().overwrite().build()?;
        println!("New: {}", define_table);
        let mut fields = std::collections::HashMap::new();
        for col in <UserV2 as TableTrait>::columns() {
            let def = ColumnTrait::def(&col);
            let field_def = ColumnTrait::to_statement(&col).build()?;
            println!("{}", field_def);
            fields.insert(def.name().to_string(), field_def);
        }
        let table_snap = TableSnapshot {
            name: "users".into(),
            define_table_statement: define_table,
            fields,
            indexes: Default::default(),
            events: Default::default(),
        };
        schema.add_table(table_snap);
        schema
    };

    // 3. Generate the diff migration
    let statements = manager.diff(&old_schema, &new_schema)?;
    println!("Total: {}", statements.join("\n"));
    assert!(
        !statements.is_empty(),
        "Expected some statements for schema change"
    );

    // We should not see a DEFINE TABLE users ... OVERWRITE
    assert!(statements
        .iter()
        .any(|s| !s.as_str().contains("DEFINE TABLE OVERWRITE users")));

    // We should see the new field being added or modified (email)
    assert!(statements.iter().any(|s| s
        .as_str()
        .contains("DEFINE FIELD OVERWRITE email ON TABLE users")));

    // 4. Simulate creating a migration file using MigrationManager (just a dry-run)
    let temp_dir = tempfile::tempdir()?;
    let manager = MigrationManager::new(temp_dir.path().into());
    let migration_name = manager.new_migration(&new_schema)?;

    // Check that files were created
    let json_path = temp_dir
        .path()
        .join(format!("{}_schema.json", &migration_name));
    assert!(json_path.exists(), "Snapshot JSON file not created");

    let json_contents = std::fs::read_to_string(&json_path)?;
    let loaded_schema: SchemaSnapshot = serde_json::from_str(&json_contents)?;
    assert_eq!(
        loaded_schema.tables.len(),
        1,
        "Expected one table in snapshot"
    );

    Ok(())
}

#[tokio::test]
async fn test_reverse_migration() -> anyhow::Result<()> {
    // This test ensures we can reverse a diff (down migration)
    // Start with UserV2 as the "new" schema and UserV1 as "old".

    let v1_table_def = <UserV1 as TableTrait>::to_statement()
        .build()
        .map_err(anyhow::Error::from)?;
    let v2_table_def = <UserV2 as TableTrait>::to_statement().overwrite().build()?;

    let mut old_schema = SchemaSnapshot::new();
    {
        let mut fields = std::collections::HashMap::new();
        for col in <UserV1 as TableTrait>::columns() {
            let def = ColumnTrait::def(&col);
            let field_def = ColumnTrait::to_statement(&col).build()?;
            fields.insert(def.name().to_string(), field_def);
        }
        let table_snap = TableSnapshot {
            name: "users".into(),
            define_table_statement: v1_table_def,
            fields,
            ..Default::default()
        };
        old_schema.add_table(table_snap);
    }

    let mut new_schema = SchemaSnapshot::new();
    {
        let mut fields = std::collections::HashMap::new();
        for col in <UserV2 as TableTrait>::columns() {
            let def = ColumnTrait::def(&col);
            let field_def = ColumnTrait::to_statement(&col).build()?;
            fields.insert(def.name().to_string(), field_def);
        }
        let table_snap = TableSnapshot {
            name: "users".into(),
            define_table_statement: v2_table_def,
            fields,
            indexes: Default::default(),
            events: Default::default(),
        };
        new_schema.add_table(table_snap);
    }

    // Diff from old -> new
    let diff = TableDiff::from_snapshots(
        old_schema.tables.get("users").unwrap(),
        new_schema.tables.get("users").unwrap(),
    )?;

    let up_statements = diff.generate_statements("users")?;
    assert!(
        !up_statements.is_empty(),
        "Should have up migration statements"
    );

    let down_statements = diff.reverse("users")?;
    assert!(
        !down_statements.is_empty(),
        "Should have down migration statements"
    );
    assert!(
        down_statements
            .iter()
            .any(|s| s.contains("REMOVE FIELD email")),
        "Down migration should remove the email field"
    );

    Ok(())
}

#[tokio::test]
async fn test_conditional_features() -> anyhow::Result<()> {
    let manager = MigrationManager::new(tempdir()?.path().into());
    // Get schema with all models
    let schema = manager.current_schema()?;

    // Test OrderV1 (has indexes)
    let order_snap = schema.tables.get("orders").expect("Order table not found");
    assert!(!order_snap.indexes.is_empty(), "Order should have indexes");
    assert!(order_snap.events.is_empty(), "Order should not have events");

    // Test ProductV1 (has events)
    let product_snap = schema
        .tables
        .get("products")
        .expect("Product table not found");
    assert!(
        product_snap.indexes.is_empty(),
        "Product should not have indexes"
    );
    assert!(
        !product_snap.events.is_empty(),
        "Product should have events"
    );

    // Verify index content
    let order_indexes = order_snap.indexes.keys().collect::<Vec<_>>();
    assert!(
        order_indexes.contains(&&"status_idx".to_string()),
        "Missing expected index"
    );

    // Verify event content
    let product_events = product_snap.events.keys().collect::<Vec<_>>();
    assert!(
        product_events.contains(&&"created".to_string()),
        "Missing expected event"
    );

    Ok(())
}

#[tokio::test]
async fn test_migration_with_features() -> anyhow::Result<()> {
    let manager = MigrationManager::new(tempdir()?.path().into());
    let old_schema = SchemaSnapshot::new(); // Empty schema
    let new_schema = {
        let mut schema = SchemaSnapshot::new();
        let table_snap = table_snapshot::<UserV1>()?;
        schema.add_table(table_snap);
        schema
    };

    let statements = manager.diff(&old_schema, &new_schema)?;

    // Verify that index and event statements are generated
    let statements_str = statements.join("\n");
    println!("{}", statements_str);
    assert!(
        statements_str.contains("DEFINE TABLE"),
        "Should generate table statements"
    );
    assert!(
        statements_str.contains("DEFINE FIELD"),
        "Should generate field statements"
    );

    // Verify order of statements (tables should be defined before fields)
    let table_pos = statements_str
        .find("DEFINE TABLE")
        .expect("Should have table definition");
    let field_pos = statements_str
        .find("DEFINE FIELD")
        .expect("Should have field definition");

    assert!(
        table_pos < field_pos,
        "Tables should be defined before fields"
    );

    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "test_inventory_table")]
pub struct TestInventoryTable {
    id: SurrealId<Self>,
    #[column(type = "string")]
    title: String,
}

#[derive(strum::EnumIter, Serialize, Deserialize, Index)]
pub enum TestInventoryTableIndexes {}

#[derive(strum::EnumIter, Serialize, Deserialize, Event)]
pub enum TestInventoryTableEvents {}

#[tokio::test]
async fn test_inventory_collection() -> anyhow::Result<()> {
    // At compile time, the `Table` macro for `TestInventoryTable` will have invoked
    // `inventory::submit!` registering this table's builder.

    // Iterate over all table registrations in the inventory
    let registrations: Vec<&TableRegistration> =
        inventory::iter::<TableRegistration>().collect::<Vec<_>>();
    assert!(!registrations.is_empty(), "Inventory should not be empty");

    // Optionally, check if "test_inventory_table" is among the registered tables
    let mut found = false;
    for reg in &registrations {
        let snapshot = (reg.builder)().map_err(anyhow::Error::from)?;
        if snapshot.name == "test_inventory_table" {
            found = true;
            // You can also verify fields, indexes, events here if desired
            assert!(
                snapshot.fields.contains_key("title"),
                "Expected 'title' field in snapshot"
            );
        }
    }

    assert!(
        found,
        "Expected to find 'test_inventory_table' in the inventory registry"
    );

    Ok(())
}
