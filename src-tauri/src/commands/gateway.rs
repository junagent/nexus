use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub enabled: bool,
    pub connected: bool,
}

#[tauri::command]
pub async fn list_gateways(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<GatewayInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.gateways.values().cloned().collect())
}

#[tauri::command]
pub async fn toggle_gateway(
    engine: State<'_, Mutex<NexusEngine>>,
    id: String,
    enable: bool,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.toggle_gateway(&id, enable).await.map_err(|e| e.to_string())
}
