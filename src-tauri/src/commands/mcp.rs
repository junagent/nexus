use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;
use crate::mcp::{McpServerConfig, McpServerInfo, McpTool};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCall {
    pub server: String,
    pub tool: String,
    pub arguments: serde_json::Value,
}

#[tauri::command]
pub async fn list_mcp_servers(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<McpServerInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.mcp_client.list_servers())
}

#[tauri::command]
pub async fn add_mcp_server(
    engine: State<'_, Mutex<NexusEngine>>,
    config: McpServerConfig,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.mcp_client.add_server(config);
    engine.save_config();
    Ok(())
}

#[tauri::command]
pub async fn remove_mcp_server(
    engine: State<'_, Mutex<NexusEngine>>,
    name: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.mcp_client.remove_server(&name);
    engine.save_config();
    Ok(())
}

#[tauri::command]
pub async fn connect_mcp_server(
    engine: State<'_, Mutex<NexusEngine>>,
    name: String,
) -> Result<Vec<McpTool>, String> {
    let mut engine = engine.lock().await;
    engine.mcp_client.connect_server(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn call_mcp_tool(
    engine: State<'_, Mutex<NexusEngine>>,
    call: McpToolCall,
) -> Result<String, String> {
    let mut engine = engine.lock().await;
    engine.mcp_client.call_tool(&call.server, &call.tool, call.arguments).await.map_err(|e| e.to_string())
}
