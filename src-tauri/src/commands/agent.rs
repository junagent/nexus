use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub status: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
    pub active: bool,
}

#[tauri::command]
pub async fn chat(
    engine: State<'_, Mutex<NexusEngine>>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let mut engine = engine.lock().await;
    engine
        .process_message(&request.message, request.session_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_stream(
    app_handle: tauri::AppHandle,
    engine: State<'_, Mutex<NexusEngine>>,
    request: ChatRequest,
) -> Result<String, String> {
    let mut engine = engine.lock().await;
    engine
        .process_message_stream(&app_handle, &request.message, request.session_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_providers(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<ProviderInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.list_providers())
}

#[tauri::command]
pub async fn set_provider(
    engine: State<'_, Mutex<NexusEngine>>,
    provider_id: String,
    model: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_active_provider(&provider_id, &model).await;
    Ok(())
}
