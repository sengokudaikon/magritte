use anyhow::Result;
use clap::{Parser, Subcommand};
use magritte::SurrealDB;
use magritte_migrations::manager::MigrationManager;
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::engine::any::connect;
use surrealdb::Surreal;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    db_url: Option<String>,

    #[arg(short, long)]
    migrations_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new migration snapshot
    Snap {
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Apply migrations
    Apply {
        #[arg(short, long)]
        version: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
    /// Rollback migrations
    Rollback {
        #[arg(short, long)]
        version: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
}

async fn connect_db(url: &str) -> Result<SurrealDB> {
    let db: Surreal<surrealdb::engine::any::Any> = connect(url).await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let migrations_dir = cli.migrations_dir.unwrap_or_else(|| PathBuf::from("migrations"));
    let manager = MigrationManager::new(migrations_dir);

    match cli.command {
        Commands::Snap { name } => {
            println!("Creating migration snapshot...");
            let db = if let Some(url) = cli.db_url {
                Some(connect_db(&url).await?)
            } else {
                None
            };

            let (snapshot_path, statements) = manager.create_snapshot(db, name).await?;
            println!("Created snapshot: {}", snapshot_path.display());
            if !statements.is_empty() {
                println!("\nGenerated statements:");
                for stmt in statements {
                    println!("{}", stmt);
                }
            }
        }
        Commands::Apply { version, force } => {
            let db = if let Some(url) = cli.db_url {
                connect_db(&url).await?
            } else {
                anyhow::bail!("Database URL is required for apply command");
            };

            if !force {
                let (snapshot_path, _) = manager.create_snapshot(Some(db.clone()), None).await?;
                let report = manager.check_deviations(&db, &snapshot_path).await?;
                if report.db_deviations.is_some() || report.schema_deviations.is_some() {
                    println!("\nSchema deviations detected:");
                    if let Some(db_devs) = report.db_deviations {
                        println!("Database deviations:");
                        for dev in db_devs {
                            println!("  {}", dev);
                        }
                    }
                    if let Some(schema_devs) = report.schema_deviations {
                        println!("Schema deviations:");
                        for dev in schema_devs {
                            println!("  {}", dev);
                        }
                    }
                    println!("\nUse --force to apply migration despite deviations");
                    return Ok(());
                }
            }

            println!("Applying migration...");
            manager.apply_migration(&db, version).await?;
            println!("Migration applied successfully");
        }
        Commands::Rollback { version, force } => {
            let db = if let Some(url) = cli.db_url {
                connect_db(&url).await?
            } else {
                anyhow::bail!("Database URL is required for rollback command");
            };

            if !force {
                let (snapshot_path, _) = manager.create_snapshot(Some(db.clone()), None).await?;
                let report = manager.check_deviations(&db, &snapshot_path).await?;
                if report.db_deviations.is_some() || report.schema_deviations.is_some() {
                    println!("\nSchema deviations detected:");
                    if let Some(db_devs) = report.db_deviations {
                        println!("Database deviations:");
                        for dev in db_devs {
                            println!("  {}", dev);
                        }
                    }
                    if let Some(schema_devs) = report.schema_deviations {
                        println!("Schema deviations:");
                        for dev in schema_devs {
                            println!("  {}", dev);
                        }
                    }
                    println!("\nUse --force to rollback despite deviations");
                    return Ok(());
                }
            }

            println!("Rolling back migration...");
            manager.rollback(&db, version).await?;
            println!("Rollback completed successfully");
        }
    }

    Ok(())
}
