use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    Router, Json, extract::State, routing::{get, post},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::agent::NexusEngine;

pub type SharedEngine = Arc<Mutex<NexusEngine>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatBody {
    pub message: String,
    pub session_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub running: bool,
    pub version: String,
    pub provider: String,
    pub model: String,
    pub sessions: u32,
    pub trace_events: usize,
    pub skills: usize,
    pub mcp_servers: usize,
}

/// Start the agent HTTP server on the given port.
/// This runs in the background, allowing the Tauri app to connect via HTTP.
pub async fn start_server(engine: SharedEngine, port: u16) -> SocketAddr {
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/status", get(get_status))
        .route("/api/chat", post(chat_handler))
        .route("/api/trace", get(trace_handler))
        .route("/api/providers", get(providers_handler))
        .route("/api/skills", get(skills_handler))
        .route("/api/mcp/servers", get(mcp_servers_handler))
        .layer(
            tower_http::cors::CorsLayer::permissive()
        )
        .with_state(engine);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Nexus agent server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    // Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    local_addr
}

// --- Handlers ---

async fn health() -> &'static str {
    "ok"
}

async fn get_status(State(engine): State<SharedEngine>) -> Json<StatusResponse> {
    let engine = engine.lock().await;
    Json(StatusResponse {
        running: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        provider: engine.active_provider.clone().unwrap_or_default(),
        model: engine.active_model.clone().unwrap_or_default(),
        sessions: engine.session_count,
        trace_events: engine.trace_store.len(),
        skills: engine.skill_store.list().len(),
        mcp_servers: engine.mcp_client.list_servers().len(),
    })
}

async fn chat_handler(
    State(engine): State<SharedEngine>,
    Json(body): Json<ChatBody>,
) -> Json<ChatResponse> {
    let mut engine = engine.lock().await;

    // Set provider/model if provided
    if let (Some(ref provider), Some(ref model)) = (&body.provider, &body.model) {
        engine.active_provider = Some(provider.clone());
        engine.active_model = Some(model.clone());
    }

    let result = engine.process_message(&body.message, body.session_id.as_deref()).await;

    match result {
        Ok(resp) => Json(ChatResponse {
            response: resp.response,
            session_id: resp.session_id,
        }),
        Err(e) => Json(ChatResponse {
            response: format!("Error: {}", e),
            session_id: body.session_id.unwrap_or_default(),
        }),
    }
}

async fn trace_handler(State(engine): State<SharedEngine>) -> Json<Vec<serde_json::Value>> {
    let engine = engine.lock().await;
    let traces: Vec<serde_json::Value> = engine.trace_store.query(None, 50).iter()
        .map(|e| serde_json::json!({
            "id": e.id, "type": e.event_type, "summary": e.summary,
            "duration_ms": e.duration_ms, "tags": e.tags,
        }))
        .collect();
    Json(traces)
}

async fn providers_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "providers": ["anthropic", "openai", "deepseek", "openrouter", "google"],
        "models": {
            "anthropic": ["claude-sonnet-4", "claude-3.5-haiku"],
            "openai": ["gpt-4o", "gpt-4o-mini", "o3-mini"],
            "deepseek": ["deepseek-chat", "deepseek-reasoner"],
            "openrouter": ["anthropic/claude-sonnet-4", "openai/gpt-4o", "google/gemini-2.0-flash"],
            "google": ["gemini-2.0-flash", "gemini-2.5-pro"]
        }
    }))
}

async fn skills_handler(State(engine): State<SharedEngine>) -> Json<Vec<serde_json::Value>> {
    let engine = engine.lock().await;
    let skills: Vec<serde_json::Value> = engine.skill_store.list().iter()
        .map(|s| serde_json::json!({
            "name": s.name, "version": s.version, "description": s.description,
            "enabled": s.enabled, "tags": s.tags,
        }))
        .collect();
    Json(skills)
}

async fn mcp_servers_handler(State(engine): State<SharedEngine>) -> Json<Vec<serde_json::Value>> {
    let engine = engine.lock().await;
    let servers: Vec<serde_json::Value> = engine.mcp_client.list_servers().iter()
        .map(|s| serde_json::json!({
            "name": s.name, "status": s.status,
            "tools": s.tools.iter().map(|t| serde_json::json!({"name": t.name, "description": t.description})).collect::<Vec<_>>(),
        }))
        .collect();
    Json(servers)
}