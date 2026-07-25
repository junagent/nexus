use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub platform: String,
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub uptime_secs: u64,
    pub agent_active: bool,
    pub active_provider: String,
    pub active_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineStatus {
    pub running: bool,
    pub provider_connected: bool,
    pub gateway_count: u32,
    pub session_count: u32,
    pub skill_count: u32,
    pub memory_usage_mb: f64,
}

#[tauri::command]
pub async fn get_system_info(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<SystemInfo, String> {
    let engine = engine.lock().await;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);

    Ok(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        cpu_cores: cores,
        memory_mb: 0,
        uptime_secs: engine.uptime.elapsed().as_secs(),
        agent_active: engine.active_provider.is_some(),
        active_provider: engine
            .active_provider
            .clone()
            .unwrap_or_else(|| "none".into()),
        active_model: engine.active_model.clone().unwrap_or_else(|| "none".into()),
    })
}

#[tauri::command]
pub async fn get_status(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<EngineStatus, String> {
    let engine = engine.lock().await;
    Ok(EngineStatus {
        running: engine.running,
        provider_connected: engine.active_provider.is_some(),
        gateway_count: engine.gateways.len() as u32,
        session_count: engine.session_count,
        skill_count: engine.skill_store.list().len() as u32,
        memory_usage_mb: 0.0,
    })
}
