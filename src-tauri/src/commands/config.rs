use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct NexusConfig {
    pub theme: String,
    pub font_size: u32,
    pub language: String,
    pub auto_start: bool,
    pub data_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub masked: bool,
}

#[tauri::command]
pub async fn get_config(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<NexusConfig, String> {
    let engine = engine.lock().await;
    Ok(NexusConfig {
        theme: engine.config.theme.clone(),
        font_size: engine.config.font_size,
        language: engine.config.language.clone(),
        auto_start: engine.config.auto_start,
        data_dir: engine.config.data_dir.clone(),
    })
}

#[tauri::command]
pub async fn update_config(
    engine: State<'_, Mutex<NexusEngine>>,
    config: NexusConfig,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn get_env(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<EnvVar>, String> {
    let engine = engine.lock().await;
    Ok(engine.list_env_vars())
}

#[tauri::command]
pub async fn set_env(
    engine: State<'_, Mutex<NexusEngine>>,
    key: String,
    value: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_env_var(&key, &value).await;
    Ok(())
}
