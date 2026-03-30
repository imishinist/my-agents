#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use std::path::PathBuf;

fn main() {
    tracing_subscriber::fmt::init();

    // Determine project directory (cwd)
    let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let conducer_dir = project_dir.join(".conducer");
    std::fs::create_dir_all(&conducer_dir).expect("Failed to create .conducer directory");
    let db_path = conducer_dir.join("state.db");
    let addr: SocketAddr = ([127, 0, 0, 1], 7700).into();

    tracing::info!("conducer starting in {}", project_dir.display());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            conducer_gui::commands::get_status,
        ])
        .setup(move |_app| {
            // Start Orchestrator in background
            let db = db_path.clone();
            tauri::async_runtime::spawn(async move {
                tracing::info!("Starting orchestrator on {}", addr);
                if let Err(e) = conducer_orchestrator::server::run_server(&db, addr).await {
                    tracing::error!("Orchestrator error: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
