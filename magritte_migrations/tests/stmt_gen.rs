use magritte::{
    table_snapshot, ColumnTrait, HasId, SchemaSnapshot, SurrealId, Table, TableRegistration,
    TableSnapshot, TableTrait,
};
use magritte_migrations::generate_diff_migration;
use magritte_migrations::manager::MigrationManager;
use magritte_migrations::table::TableDiff;
use magritte_migrations::test_models::{UserV1, UserV2};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[tokio::test]
async fn test_generate_diff_from_snapshots() -> anyhow::Result<()> {
    // 1. Create a snapshot from UserV1 schema
    let old_schema = {
        let mut schema = SchemaSnapshot::new();
        let define_table = <UserV1 as TableTrait>::to_statement()
            .build()
            .map_err(anyhow::Error::from)?;
        println!("{}", define_table);
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
        println!("{}", define_table);
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
    let statements = generate_diff_migration(&old_schema, &new_schema)?;
    println!("{}", statements.join("\n"));
    assert!(
        !statements.is_empty(),
        "Expected some statements for schema change"
    );

    // We should see a DEFINE TABLE users ... OVERWRITE
    assert!(statements
        .iter()
        .any(|s| s.as_str().contains("DEFINE TABLE OVERWRITE users")));

    // We should see the new field being added or modified (email)
    assert!(statements.iter().any(|s| s
        .as_str()
        .contains("DEFINE OVERWRITE FIELD email ON TABLE users")));

    // 4. Simulate creating a migration file using MigrationManager (just a dry-run)
    let temp_dir = tempfile::tempdir()?;
    let manager = MigrationManager::new(temp_dir.path().into());
    let migration_name = manager.create_new_migration(&new_schema)?;

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
            indexes: Default::default(),
            events: Default::default(),
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

// Fake current_schema_from_code implementations
// In a real scenario, current_schema_from_code() would rely on inventory
// and see which tables are compiled in. For demonstration, we mimic two scenarios.

fn schema_with_user_v1() -> anyhow::Result<SchemaSnapshot> {
    // The inventory would have one entry from UserV1. For this test, we just build it manually.
    let mut schema = SchemaSnapshot::new();
    let table_snap = table_snapshot::<UserV1>()?;
    schema.add_table(table_snap);
    Ok(schema)
}

fn schema_with_user_v2() -> anyhow::Result<SchemaSnapshot> {
    // The inventory would now have UserV2 registered instead of UserV1.
    // For testing, we just build directly.
    let mut schema = SchemaSnapshot::new();
    let table_snap = table_snapshot::<UserV2>()?;
    schema.add_table(table_snap);
    Ok(schema)
}

#[tokio::test]
async fn test_inventory_registration_and_migration() -> anyhow::Result<()> {
    // Simulate initial state: UserV1 schema
    let old_schema = schema_with_user_v1()?;
    assert_eq!(old_schema.tables.len(), 1, "Should have one table (UserV1)");

    // Simulate new state: UserV2 schema
    let new_schema = schema_with_user_v2()?;
    assert_eq!(new_schema.tables.len(), 1, "Should have one table (UserV2)");

    // Generate diff
    let diff_statements = generate_diff_migration(&old_schema, &new_schema)?;
    assert!(!diff_statements.is_empty(), "Expected diff statements");
    // Check that a new field (email) is introduced
    let email_field = diff_statements.iter().any(|s| s.contains("email"));
    assert!(email_field, "Should have added an 'email' field");

    // Use MigrationManager to create a new migration
    let tmp = tempdir()?;
    let manager = MigrationManager::new(tmp.path().into());

    // Create migration file for new_schema
    let migration_name = manager.create_new_migration(&new_schema)?;
    let json_path = tmp.path().join(format!("{}_schema.json", &migration_name));

    assert!(json_path.exists(), "JSON snapshot file not created");

    // Now, we can simulate applying the migration:
    // In a real scenario, you'd connect to SurrealDB and run the statements.
    // For testing, just ensure that we can parse and generate them.
    // If connected to a test instance of SurrealDB, you could run them:
    //
    // let ctx = magritte_migrations::types::MigrationContext::new(...) // connect to SurrealDB
    // for stmt in diff_statements {
    //     ctx.db.execute(&stmt).await?;
    // }

    Ok(())
}

#[tokio::test]
async fn test_reverse_migration_from_inventory() -> anyhow::Result<()> {
    // Create snapshots again
    let old_schema = schema_with_user_v1()?;
    let new_schema = schema_with_user_v2()?;

    // Generate diff and reverse statements
    let diff_statements = generate_diff_migration(&old_schema, &new_schema)?;
    assert!(
        !diff_statements.is_empty(),
        "Expected up migration statements"
    );

    // Create a TableDiff from snapshots directly if needed
    let old_table = old_schema.tables.get("users").unwrap();
    let new_table = new_schema.tables.get("users").unwrap();
    let table_diff = TableDiff::from_snapshots(old_table, new_table)?;
    let down_statements = table_diff.reverse("users")?;
    assert!(
        !down_statements.is_empty(),
        "Expected down migration statements"
    );

    // Check that it removes the email field
    assert!(
        down_statements
            .iter()
            .any(|s| s.contains("REMOVE FIELD email")),
        "Down migration should remove the email field"
    );

    // If connected to SurrealDB, you would run these down statements to revert changes.

    Ok(())
}
#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "test_inventory_table")]
pub struct TestInventoryTable {
    id: SurrealId<Self>,
    #[column(type = "string")]
    title: String,
}

impl HasId for TestInventoryTable {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

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
