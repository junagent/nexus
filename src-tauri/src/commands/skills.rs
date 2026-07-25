use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub tags: Vec<String>,
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
    Ok(engine.skill_store.list().iter().map(|s| SkillInfo {
        name: s.name.clone(),
        version: s.version.clone(),
        description: s.description.clone(),
        author: s.author.clone(),
        enabled: s.enabled,
        tags: s.tags.clone(),
    }).collect())
}

#[tauri::command]
pub async fn install_skill(
    engine: State<'_, Mutex<NexusEngine>>,
    source: String,
) -> Result<SkillInstallResult, String> {
    let mut engine = engine.lock().await;
    engine.skill_store.install(&source).await.map(|msg| SkillInstallResult {
        success: true,
        name: source.split('/').last().unwrap_or(&source).trim_end_matches(".git").to_string(),
        message: msg,
    })
}

#[tauri::command]
pub async fn remove_skill(
    engine: State<'_, Mutex<NexusEngine>>,
    name: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    if engine.skill_store.remove(&name) {
        Ok(())
    } else {
        Err(format!("Skill '{}' not found", name))
    }
}

#[tauri::command]
pub async fn toggle_skill(
    engine: State<'_, Mutex<NexusEngine>>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    if engine.skill_store.set_enabled(&name, enabled) {
        Ok(())
    } else {
        Err(format!("Skill '{}' not found", name))
    }
}

#[tauri::command]
pub async fn reload_skills(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<SkillInfo>, String> {
    let mut engine = engine.lock().await;
    engine.skill_store.reload();
    Ok(engine.skill_store.list().iter().map(|s| SkillInfo {
        name: s.name.clone(),
        version: s.version.clone(),
        description: s.description.clone(),
        author: s.author.clone(),
        enabled: s.enabled,
        tags: s.tags.clone(),
    }).collect())
}