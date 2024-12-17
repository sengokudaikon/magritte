use anyhow::Context;
use clap::{Parser, Subcommand};
use console::style;
use magritte_migrations::current_schema_from_code;
use magritte_migrations::snapshot::{load_from_file, save_to_file};
use magritte_migrations::types::FlexibleDateTime;
use magritte_migrations::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init_migrations_dir(&cli.migrations_dir).await?;
        }
        Commands::Snapshot { action } => match action {
            SnapshotCommands::Save { name } => {
                save_snapshot(&cli.migrations_dir, name).await?;
            }
            SnapshotCommands::Restore { path } => {
                restore_snapshot(path).await?;
            }
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

pub(crate) async fn save_snapshot(migrations_dir: &PathBuf, name: Option<String>) -> Result<()> {
    let current_schema = current_schema_from_code()?;

    let snapshot_dir = migrations_dir.join("snapshots");
    std::fs::create_dir_all(&snapshot_dir)?;

    let filename = match name {
        Some(n) => format!("{}.json", n),
        None => format!("snapshot_{}.json", FlexibleDateTime::now().to_string()),
    };

    let path = snapshot_dir.join(filename);
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
