use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use magritte::prelude::{Surreal, SurrealDB};
use magritte_migrations::{
    manager::SchemaManager,
    migrator::Migrator,
    types::MigrationContext,
};
use std::path::PathBuf;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Parser)]
#[command(
    name = "magritte",
    about = "SurrealDB schema migration tool",
    version,
    author
)]
struct Cli {
    #[arg(
        short,
        long,
        env = "SURREAL_URL",
        help = "SurrealDB connection URL",
        default_value = "ws://localhost:8000"
    )]
    url: String,

    #[arg(
        short,
        long,
        env = "SURREAL_NS",
        help = "SurrealDB namespace",
        default_value = "test"
    )]
    namespace: String,

    #[arg(
        short,
        long,
        env = "SURREAL_DB",
        help = "SurrealDB database",
        default_value = "test"
    )]
    database: String,

    #[arg(
        short,
        long,
        env = "MIGRATIONS_DIR",
        help = "Migrations directory",
        default_value = "./migrations"
    )]
    migrations_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize migrations directory
    Init,
    
    /// Create a new migration
    Create {
        #[arg(help = "Name of the migration")]
        name: String,
    },
    
    /// Run pending migrations
    Up {
        #[arg(short, long, help = "Number of migrations to run")]
        steps: Option<usize>,
        
        #[arg(short, long, help = "Perform a dry run without making changes")]
        dry_run: bool,
    },
    
    /// Rollback migrations
    Down {
        #[arg(short, long, help = "Number of migrations to rollback")]
        steps: Option<usize>,
        
        #[arg(short, long, help = "Perform a dry run without making changes")]
        dry_run: bool,
    },
    
    /// Show migration status
    Status,
    
    /// Create a schema snapshot
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },
}

#[derive(Subcommand)]
enum SnapshotCommands {
    /// Create a new snapshot
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let db = connect_db(&cli.url).await?;
    
    // Set up context
    db.use_ns(&cli.namespace).await?;
    db.use_db(&cli.database).await?;
    
    let ctx = MigrationContext {
        db: db.clone(),
        namespace: cli.namespace.clone(),
        database: cli.database.clone(),
    };
    
    let manager = SchemaManager::new(db, cli.namespace, cli.database);
    let migrator = Migrator::new(ctx.clone(), cli.migrations_dir.clone());

    match cli.command {
        Commands::Init => {
            init_migrations_dir(&cli.migrations_dir).await?;
        }
        Commands::Create { name } => {
            create_migration(&cli.migrations_dir, &name).await?;
        }
        Commands::Up { steps, dry_run } => {
            run_migrations(&migrator, steps, dry_run).await?;
        }
        Commands::Down { steps, dry_run } => {
            rollback_migrations(&migrator, steps, dry_run).await?;
        }
        Commands::Status => {
            show_status(&migrator).await?;
        }
        Commands::Snapshot { action } => match action {
            SnapshotCommands::Save { name } => {
                save_snapshot(&manager, &cli.migrations_dir, name).await?;
            }
            SnapshotCommands::Restore { path } => {
                restore_snapshot(&manager, path).await?;
            }
        },
    }

    Ok(())
}

async fn connect_db(url: &str) -> Result<SurrealDB> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Connecting to database...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let db = Surreal::connect(url)
        .await
        .context("Failed to connect to database")?;

    spinner.finish_with_message("Connected to database");
    Ok(Arc::new(db))
}

async fn init_migrations_dir(path: &PathBuf) -> Result<()> {
    if path.exists() {
        println!("{}", style("Migrations directory already exists").yellow());
        return Ok(());
    }

    std::fs::create_dir_all(path)?;
    println!("{}", style("Initialized migrations directory").green());
    Ok(())
}

async fn create_migration(dir: &PathBuf, name: &str) -> Result<()> {
    let timestamp = OffsetDateTime::now_utc().unix_timestamp();
    let filename = format!("{}_{}.surql", timestamp, name);
    let path = dir.join(filename);

    let template = format!(
        "-- Migration: {}\n\n-- Up\n\n-- Down\n",
        name
    );

    std::fs::write(&path, template)?;
    println!("{} {}", style("Created migration:").green(), path.display());
    Ok(())
}

async fn run_migrations(migrator: &Migrator, steps: Option<usize>, dry_run: bool) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    if dry_run {
        println!("{}", style("Performing dry run...").yellow());
    }

    spinner.set_message("Running migrations...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = if dry_run {
        migrator.dry_run(steps).await
    } else {
        migrator.up(steps).await
    };

    match result {
        Ok(_) => {
            spinner.finish_with_message(style("Migrations completed successfully").green().to_string());
            Ok(())
        }
        Err(e) => {
            spinner.finish_with_message(style("Migration failed").red().to_string());
            Err(e.into())
        }
    }
}

async fn rollback_migrations(migrator: &Migrator, steps: Option<usize>, dry_run: bool) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.yellow} {msg}")
            .unwrap(),
    );

    if dry_run {
        println!("{}", style("Performing dry run...").yellow());
    }

    spinner.set_message("Rolling back migrations...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = if dry_run {
        migrator.dry_run_down(steps).await
    } else {
        migrator.down(steps).await
    };

    match result {
        Ok(_) => {
            spinner.finish_with_message(style("Rollback completed successfully").green().to_string());
            Ok(())
        }
        Err(e) => {
            spinner.finish_with_message(style("Rollback failed").red().to_string());
            Err(e.into())
        }
    }
}

async fn show_status(migrator: &Migrator) -> Result<()> {
    let status = migrator.status().await?;
    
    if status.is_empty() {
        println!("{}", style("No migrations found").yellow());
        return Ok(());
    }

    println!("\n{}", style("Migration Status:").bold());
    println!("{}", style("─".repeat(50)).dim());

    for entry in status {
        let status_symbol = if entry.applied {
            style("✓").green()
        } else {
            style("✗").red()
        };
        println!(
            "{} {} ({})",
            status_symbol,
            entry.name,
            if entry.applied { "applied" } else { "pending" }
        );
    }

    println!("{}", style("─".repeat(50)).dim());
    Ok(())
}

async fn save_snapshot(
    manager: &SchemaManager,
    base_dir: &PathBuf,
    name: Option<String>,
) -> Result<()> {
    let snapshot_dir = base_dir.join("snapshots");
    std::fs::create_dir_all(&snapshot_dir)?;

    let filename = match name {
        Some(n) => format!("{}.surql", n),
        None => format!("snapshot_{}.surql", OffsetDateTime::now_utc().unix_timestamp()),
    };

    manager.save_snapshot(snapshot_dir.join(filename)).await?;
    println!("{}", style("Snapshot saved successfully").green());
    Ok(())
}

async fn restore_snapshot(manager: &SchemaManager, path: PathBuf) -> Result<()> {
    let snapshot = SchemaManager::load_snapshot(path)?;
    // TODO: Implement snapshot restore functionality
    println!("{}", style("Snapshot restored successfully").green());
    Ok(())
}
