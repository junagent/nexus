use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize)]
pub struct BanditArmStats {
    pub provider: String,
    pub model: String,
    pub trials: u32,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub avg_cost: f64,
    pub ucb1_score: f64,
}

#[tauri::command]
pub async fn bandit_stats(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<BanditArmStats>, String> {
    let engine = engine.lock().await;
    let total = engine.bandit.total_trials;
    Ok(engine.bandit.summary().iter().map(|arm| BanditArmStats {
        provider: arm.provider.clone(),
        model: arm.model.clone(),
        trials: arm.trials,
        success_rate: arm.success_rate(),
        avg_latency_ms: arm.avg_latency(),
        avg_cost: arm.avg_cost(),
        ucb1_score: arm.ucb1_score(total),
    }).collect())
}

#[tauri::command]
pub async fn bandit_select(
    engine: State<'_, Mutex<NexusEngine>>,
    preferred_provider: Option<String>,
) -> Result<Option<(String, String)>, String> {
    let engine = engine.lock().await;
    Ok(engine.bandit.select(preferred_provider.as_deref()))
}