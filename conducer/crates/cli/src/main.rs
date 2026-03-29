use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "conducer", about = "Autonomous coding agent orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Port to listen on
    #[arg(short, long, default_value = "7700")]
    port: u16,

    /// Path to the project directory
    #[arg(short = 'd', long, default_value = ".")]
    project_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the orchestrator server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "7700")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let port = match &cli.command {
        Some(Commands::Start { port }) => *port,
        None => cli.port,
    };

    let project_dir = cli.project_dir.canonicalize().unwrap_or(cli.project_dir);
    let conducer_dir = project_dir.join(".conducer");
    std::fs::create_dir_all(&conducer_dir)?;

    let db_path = conducer_dir.join("state.db");
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();

    tracing::info!("conducer starting in {}", project_dir.display());
    tracing::info!("state db: {}", db_path.display());

    conducer_orchestrator::server::run_server(&db_path, addr).await?;

    Ok(())
}
