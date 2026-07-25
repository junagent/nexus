use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub message_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub model: String,
}

#[tauri::command]
pub async fn list_sessions(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<SessionInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.list_sessions())
}

#[tauri::command]
pub async fn delete_session(
    engine: State<'_, Mutex<NexusEngine>>,
    id: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.delete_session(&id).await.map_err(|e| e.to_string())
}
