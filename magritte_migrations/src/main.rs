use anyhow::Context;
use clap::{Parser, Subcommand};
use console::style;
use magritte_migrations::snapshot::{load_from_file, save_to_file};
use magritte_migrations::types::FlexibleDateTime;
use magritte_migrations::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use surrealdb::engine::any::connect;
use surrealdb::opt::auth::Root;
use magritte::{Query, SurrealDB};
use magritte_migrations::manager::MigrationManager;

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            init_migrations_dir(&cli.migrations_dir).await?;
        }
        Commands::Snapshot { action } => match action {
            SnapshotCommands::Save { name } => {
                save_snapshot(&cli.migrations_dir, name.clone()).await?;
            }
            SnapshotCommands::Restore { path } => {
                restore_snapshot(path.clone()).await?;
            }
        },
        Commands::Apply { snapshot, dry_run } => {
            handle_apply(&cli, snapshot.clone(), *dry_run).await?;
        },
        Commands::Rollback { snapshot, dry_run } => {
            handle_rollback(&cli, snapshot.clone(), *dry_run).await?;
        },
    }

    Ok(())
}

#[derive(Parser)]
#[command(
    name = "magritte",
    about = "SurrealDB schema migration tool",
    version,
    author
)]
pub(crate) struct Cli {
    #[arg(
        short,
        long,
        env = "MIGRATIONS_DIR",
        help = "Migrations directory",
        default_value = "./migrations"
    )]
    pub(crate) migrations_dir: PathBuf,

    #[arg(
        long,
        env = "SURREALDB_URL",
        help = "SurrealDB connection URL",
        default_value = "ws://localhost:8000"
    )]
    pub(crate) db_url: String,

    #[arg(
        long,
        env = "SURREALDB_NS",
        help = "SurrealDB namespace",
        default_value = "test"
    )]
    pub(crate) namespace: String,

    #[arg(
        long,
        env = "SURREALDB_DB",
        help = "SurrealDB database",
        default_value = "test"
    )]
    pub(crate) database: String,

    #[arg(
        long,
        env = "SURREALDB_USER",
        help = "SurrealDB root username"
    )]
    pub(crate) username: String,

    #[arg(
        long,
        env = "SURREALDB_PASS",
        help = "SurrealDB root password"
    )]
    pub(crate) password: String,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Initialize migrations directory
    Init,

    /// Create and manage schema snapshots
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },

    /// Apply schema changes
    Apply {
        /// Path to the snapshot file to apply
        #[arg(help = "Path to the snapshot file")]
        snapshot: PathBuf,
        
        /// Dry run - show what would be applied
        #[arg(long, help = "Show what would be applied without executing")]
        dry_run: bool,
    },

    /// Rollback to a previous snapshot
    Rollback {
        /// Path to the snapshot to rollback to
        #[arg(help = "Path to the snapshot file")]
        snapshot: PathBuf,
        
        /// Dry run - show what would be rolled back
        #[arg(long, help = "Show what would be rolled back without executing")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum SnapshotCommands {
    /// Create a new snapshot of the current code schema
    Save {
        #[arg(short, long, help = "Snapshot name")]
        name: Option<String>,
    },
    /// Restore from a snapshot
    Restore {
        #[arg(help = "Snapshot file path")]
        path: PathBuf,
    },
}

pub(crate) async fn init_migrations_dir(path: &PathBuf) -> Result<()> {
    if path.exists() {
        println!("{}", style("Migrations directory already exists").yellow());
        return Ok(());
    }

    std::fs::create_dir_all(path)?;
    println!("{}", style("Initialized migrations directory").green());
    Ok(())
}

pub(crate) async fn save_snapshot(migrations_dir: &Path, name: Option<String>) -> Result<()> {
    let manager = MigrationManager::new(PathBuf::from(migrations_dir));
    let current_schema = manager.current_schema_from_code()?;
    let filename = match name {
        Some(n) => format!("{}.json", n),
        None => format!("{}_schema.json", FlexibleDateTime::now()),
    };

    let path = migrations_dir.join(filename);
    save_to_file(&current_schema, &path)?;
    println!(
        "{} {}",
        style("Snapshot saved successfully:").green(),
        path.display()
    );
    Ok(())
}

pub(crate) async fn restore_snapshot(path: PathBuf) -> Result<()> {
    let snapshot = load_from_file(path.clone())
        .with_context(|| format!("Failed to load snapshot from {}", path.display()))?;

    println!(
        "{} {}",
        style("Loaded snapshot from:").green(),
        path.display()
    );
    println!("Tables:");
    for (name, table) in &snapshot.tables {
        println!(" - {}", name);
        for field_name in table.fields.keys() {
            println!("   Field: {}", field_name);
        }
    }

    println!("Edges:");
    for (name, edge) in &snapshot.edges {
        println!(" - {}", name);
        for field_name in edge.fields.keys() {
            println!("   Field: {}", field_name);
        }
    }

    Ok(())
}

pub(crate) async fn handle_apply(cli: &Cli, snapshot: PathBuf, dry_run: bool) -> Result<()> {
    let db = connect_db(cli).await?;
    let manager = MigrationManager::new(cli.migrations_dir.clone());
    
    println!("{} {}", style("Applying snapshot:").blue(), snapshot.display());
    
    // Load target snapshot
    let target_snapshot = snapshot::load_from_file(&snapshot)?;
    
    // Get current DB state
    let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;
    
    // Generate apply statements
    let statements = manager.generate_diff_migration(&db_snapshot, &target_snapshot)?;
    
    if statements.is_empty() {
        println!("{}", style("No changes to apply").green());
        return Ok(());
    }

    println!("\nChanges to apply:");
    for stmt in &statements {
        println!("  {}", stmt);
    }

    if dry_run {
        println!("\n{}", style("Dry run - no changes applied").yellow());
        return Ok(());
    }

    let mut transaction = Query::begin();
    for stmt in statements {
        transaction = transaction.raw(&stmt);
    }
    
    transaction.commit().execute(&db).await.map_err(Error::from)?;
    println!("{}", style("Apply completed successfully").green());
    Ok(())
}

pub(crate) async fn handle_rollback(cli: &Cli, snapshot: PathBuf, dry_run: bool) -> Result<()> {
    let db = connect_db(cli).await?;
    let manager = MigrationManager::new(cli.migrations_dir.clone());
    
    println!("{} {}", style("Rolling back to snapshot:").blue(), snapshot.display());
    
    // Load target snapshot
    let target_snapshot = snapshot::load_from_file(&snapshot)?;
    
    // Get current DB state
    let db_snapshot = introspection::create_snapshot_from_db(db.clone()).await?;
    
    // Generate rollback statements by swapping old/new in diff generation
    let statements = manager.generate_diff_migration(&db_snapshot, &target_snapshot)?;
    
    if statements.is_empty() {
        println!("{}", style("No changes to rollback").green());
        return Ok(());
    }

    println!("\nChanges to rollback:");
    for stmt in &statements {
        println!("  {}", stmt);
    }

    if dry_run {
        println!("\n{}", style("Dry run - no changes applied").yellow());
        return Ok(());
    }

    let mut transaction = Query::begin();
    for stmt in statements {
        transaction = transaction.raw(&stmt);
    }
    
    transaction.commit().execute(&db).await.map_err(Error::from)?;
    println!("{}", style("Rollback completed successfully").green());
    Ok(())
}

async fn connect_db(cli: &Cli) -> Result<SurrealDB> {
    let db_url = cli.db_url.clone();
    let db = connect(db_url).await?;
    db.use_db(cli.database.clone()).await?;
    db.use_ns(cli.namespace.clone()).await?;
    db.signin(
        Root {
            username: &cli.username,
            password: &cli.password,
        }
    ).await?;
    Ok(Arc::new(db))
}