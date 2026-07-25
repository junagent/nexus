use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize)]
pub struct TraceEventView {
    pub id: u64,
    pub timestamp: String,
    pub event_type: String,
    pub session_id: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub summary: String,
    pub detail: String,
    pub duration_ms: f64,
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn trace_query(
    engine: State<'_, Mutex<NexusEngine>>,
    filter: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<TraceEventView>, String> {
    let engine = engine.lock().await;
    let limit = limit.unwrap_or(50);
    Ok(engine.trace_store.query(filter.as_deref(), limit).iter().map(|e| TraceEventView {
        id: e.id,
        timestamp: e.timestamp.clone(),
        event_type: e.event_type.clone(),
        session_id: e.session_id.clone(),
        provider: e.provider.clone(),
        model: e.model.clone(),
        summary: e.summary.clone(),
        detail: e.detail.clone(),
        duration_ms: e.duration_ms,
        tags: e.tags.clone(),
    }).collect())
}

#[tauri::command]
pub async fn trace_clear(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.trace_store.clear();
    Ok(())
}

#[tauri::command]
pub async fn trace_count(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<usize, String> {
    let engine = engine.lock().await;
    Ok(engine.trace_store.len())
}