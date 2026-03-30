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
    /// Start the orchestrator server (headless, no GUI)
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

    let project_dir = cli.project_dir.canonicalize().unwrap_or(cli.project_dir);
    let conducer_dir = project_dir.join(".conducer");
    std::fs::create_dir_all(&conducer_dir)?;
    let db_path = conducer_dir.join("state.db");

    match &cli.command {
        Some(Commands::Start { port }) => {
            // Headless mode: just run the orchestrator server
            let addr: SocketAddr = ([127, 0, 0, 1], *port).into();
            tracing::info!("conducer (headless) starting in {}", project_dir.display());
            conducer_orchestrator::server::run_server(&db_path, addr).await?;
        }
        None => {
            // Default: launch GUI app (Tauri)
            // The GUI binary is `conducer-gui`, built alongside this CLI
            tracing::info!("conducer starting in {}", project_dir.display());
            let gui_bin = std::env::current_exe()?
                .parent()
                .unwrap()
                .join("conducer-gui");

            if gui_bin.exists() {
                let status = std::process::Command::new(&gui_bin)
                    .current_dir(&project_dir)
                    .status()?;
                std::process::exit(status.code().unwrap_or(1));
            } else {
                // Fallback: run headless if GUI binary not found
                tracing::warn!("GUI binary not found at {:?}, falling back to headless mode", gui_bin);
                let addr: SocketAddr = ([127, 0, 0, 1], cli.port).into();
                conducer_orchestrator::server::run_server(&db_path, addr).await?;
            }
        }
    }

    Ok(())
}
