use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

#[derive(Debug, Serialize)]
pub struct ApprovalRequestView {
    pub id: String,
    pub timestamp: String,
    pub tool_name: String,
    pub arguments: String,
    pub risk_level: String,
    pub reason: String,
    pub status: String,
}

#[tauri::command]
pub async fn approval_pending(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<ApprovalRequestView>, String> {
    let engine = engine.lock().await;
    Ok(engine.approval.pending().iter()
        .filter(|r| r.status == "pending")
        .map(|r| ApprovalRequestView {
            id: r.id.clone(),
            timestamp: r.timestamp.clone(),
            tool_name: r.tool_name.clone(),
            arguments: r.arguments.clone(),
            risk_level: format!("{:?}", r.risk_level),
            reason: r.reason.clone(),
            status: r.status.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn approval_respond(
    engine: State<'_, Mutex<NexusEngine>>,
    id: String,
    approved: bool,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    if engine.approval.respond(&id, approved) {
        Ok(())
    } else {
        Err(format!("Approval request '{}' not found", id))
    }
}

#[tauri::command]
pub async fn approval_check(
    engine: State<'_, Mutex<NexusEngine>>,
    tool_name: String,
    arguments: String,
) -> Result<serde_json::Value, String> {
    let mut engine = engine.lock().await;
    let (allowed, request) = engine.approval.check(&tool_name, &arguments);
    Ok(serde_json::json!({
        "allowed": allowed,
        "request": request.map(|r| serde_json::json!({
            "id": r.id,
            "risk_level": format!("{:?}", r.risk_level),
            "reason": r.reason,
        }))
    }))
}