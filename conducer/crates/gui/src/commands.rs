use serde::Serialize;

/// Get the orchestrator status (health check from GUI)
#[tauri::command]
pub fn get_status() -> StatusResponse {
    StatusResponse {
        running: true,
        port: 7700,
    }
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub port: u16,
}
