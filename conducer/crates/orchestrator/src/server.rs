use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::llm::ClaudeCodeClient;
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

    // Default to Claude Code CLI backend
    let llm: Box<dyn crate::llm::LlmClient> = Box::new(ClaudeCodeClient::new());
    let pm_agent = PmAgent::new(llm, pool.clone());

    let state = Arc::new(AppState { pool, event_tx, pm_agent });

    let app = create_router(state).layer(CorsLayer::permissive());

    tracing::info!("conducer orchestrator listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
