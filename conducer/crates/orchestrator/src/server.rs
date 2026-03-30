use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::llm::{ClaudeCodeClient, KiroCliClient};
use crate::pm_agent::PmAgent;
use crate::router::{SseEvent, create_router};

pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub event_tx: broadcast::Sender<SseEvent>,
    pub pm_agent: PmAgent,
}

pub async fn run_server(db_path: &Path, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let pool = conducer_state::db::init_pool(db_path).await?;
    let (event_tx, _) = broadcast::channel::<SseEvent>(256);

    // Auto-detect available LLM backend: prefer claude, fallback to kiro-cli
    let llm: Box<dyn crate::llm::LlmClient> = if which_exists("claude") {
        tracing::info!("Using Claude Code CLI as LLM backend");
        Box::new(ClaudeCodeClient::new())
    } else if which_exists("kiro-cli") {
        tracing::info!("Using Kiro CLI as LLM backend");
        Box::new(KiroCliClient::new())
    } else {
        tracing::warn!("No LLM CLI found (claude or kiro-cli). PM Agent will fail.");
        Box::new(ClaudeCodeClient::new()) // will error at runtime
    };

    let pm_agent = PmAgent::new(llm, pool.clone());

    let state = Arc::new(AppState { pool, event_tx, pm_agent });

    let app = create_router(state).layer(CorsLayer::permissive());

    tracing::info!("conducer orchestrator listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
