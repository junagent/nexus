use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize)]
pub struct MemoryEntry {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct MemorySession {
    pub session_id: String,
    pub message_count: u32,
    pub first_ts: String,
    pub last_ts: String,
}

#[tauri::command]
pub async fn memory_list(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<MemorySession>, String> {
    let engine = engine.lock().await;
    match &engine.memory {
        Some(m) => m.list_sessions(),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn memory_get(
    engine: State<'_, Mutex<NexusEngine>>,
    session_id: String,
    limit: Option<usize>,
) -> Result<Vec<MemoryEntry>, String> {
    let engine = engine.lock().await;
    match &engine.memory {
        Some(m) => m.get_messages(&session_id, limit.unwrap_or(100)),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn memory_clear(
    engine: State<'_, Mutex<NexusEngine>>,
    session_id: String,
) -> Result<(), String> {
    let engine = engine.lock().await;
    match &engine.memory {
        Some(m) => m.delete_session(&session_id),
        None => Err("Memory not initialized".into()),
    }
}
