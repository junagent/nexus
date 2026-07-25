use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInstallResult {
    pub success: bool,
    pub name: String,
    pub message: String,
}

#[tauri::command]
pub async fn list_skills(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<SkillInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.skills.values().cloned().collect())
}

#[tauri::command]
pub async fn install_skill(
    engine: State<'_, Mutex<NexusEngine>>,
    source: String,
) -> Result<SkillInstallResult, String> {
    let mut engine = engine.lock().await;
    engine.install_skill(&source).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_skill(
    engine: State<'_, Mutex<NexusEngine>>,
    name: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.remove_skill(&name).await.map_err(|e| e.to_string())
}
