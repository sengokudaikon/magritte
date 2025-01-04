# Magritte Migrations

A schema migration tool for SurrealDB that provides safe, version-controlled schema management.

## Features

- Schema snapshots for tables and edges
- Migration versioning with timestamps
- Schema validation and deviation reporting
- Safe schema updates with OVERWRITE semantics
- Transaction support for atomic changes
- Edge table schema management
- Comprehensive validation reporting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
magritte_migrations = "0.1.0"
```

## Usage

### Basic Example

```rust
use magritte_migrations::{manager::MigrationManager, Result};
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::engine::any::connect;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the migration manager
    let manager = MigrationManager::new(PathBuf::from("./migrations"));
    
    // Create a new migration from current schema
    let (path, statements) = manager.create_snapshot(None, None).await?;
    
    // Connect to database and apply migration
    let db = Arc::new(connect(None).await?);
    manager.apply_migration(&db, None).await?;
    
    Ok(())
}
```

### Managing Migrations

The migration manager supports:

1. Creating new migrations:
```rust, ignore
let (path, statements) = manager.create_snapshot(Some(db.clone()), None).await?;
```

2. Applying migrations:
```rust, ignore
manager.apply_migration(&db, None).await?;
```

3. Rolling back migrations:
```rust, ignore
manager.rollback(&db, None).await?;
```

### Schema Validation

The tool provides comprehensive schema validation:

```rust, ignore
let report = manager.check_deviations(&db, &snapshot_path).await?;
if report.has_issues() {
    println!("Schema deviations found:");
    if let Some(schema_diff) = &report.schema_deviations {
        println!("Schema deviations:\n{}", schema_diff.join("\n"));
    }
    if let Some(db_diff) = &report.db_deviations {
        println!("Database deviations:\n{}", db_diff.join("\n"));
    }
}
```

## Note on Relations

While the crate handles edge table schemas, actual relation data migrations (`RELATE` statements) must be handled manually as they depend on application-specific record IDs.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details. 
This project is licensed under the Apache License, Version 2.0 - see the [LICENSE-APACHE](LICENSE-APACHE) file for details.