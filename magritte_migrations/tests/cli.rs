use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// Helper to create a test command
fn magritte_cmd() -> Command {
    Command::cargo_bin("magritte_migrations").unwrap()
}

#[tokio::test]
async fn test_init_command() -> Result<()> {
    let temp = tempdir()?;
    let migrations_dir = temp.path().join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // Test initial creation
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created snapshot"));

    assert!(migrations_dir.exists());

    // Test idempotency
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created snapshot"));

    Ok(())
}

#[tokio::test]
async fn test_snapshot_command() -> Result<()> {
    let temp = tempdir()?;
    let migrations_dir = temp.path().join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // Test snapshot without DB (should succeed with registered schemas)
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created snapshot"));

    // Test snapshot with DB connection (should show diff statements)
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("mem://")
        .arg("snap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated statements"));

    Ok(())
}

#[tokio::test]
async fn test_apply_command() -> Result<()> {
    let temp = tempdir()?;
    let migrations_dir = temp.path().join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // Create initial snapshot
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .success();

    // Test apply with force
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("mem://")
        .arg("apply")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration applied successfully"));

    // Test apply with deviations (should show warning)
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("mem://")
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema deviations detected"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_command() -> Result<()> {
    let temp = tempdir()?;
    let migrations_dir = temp.path().join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // Create initial snapshot
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .success();

    // Test rollback with force
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("mem://")
        .arg("rollback")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rollback completed successfully"));

    // Test rollback with deviations (should show warning)
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("mem://")
        .arg("rollback")
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema deviations detected"));

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let temp = tempdir()?;
    let migrations_dir = temp.path().join("migrations");

    // Test missing migrations directory
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("snap")
        .assert()
        .failure();

    // Test invalid DB connection
    magritte_cmd()
        .arg("-m")
        .arg(&migrations_dir)
        .arg("--db-url")
        .arg("invalid://")
        .arg("snap")
        .assert()
        .failure();

    Ok(())
}
