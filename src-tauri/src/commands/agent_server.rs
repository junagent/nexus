use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub running: bool,
    pub port: u16,
    pub url: String,
}

#[tauri::command]
pub async fn agent_server_status(
    _engine: State<'_, Mutex<NexusEngine>>,
) -> Result<ServerInfo, String> {
    // Check if the server is running by hitting the health endpoint
    let port = 18789u16;
    let url = format!("http://127.0.0.1:{}", port);
    let running = reqwest::get(&format!("{}/health", url))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    Ok(ServerInfo { running, port, url })
}