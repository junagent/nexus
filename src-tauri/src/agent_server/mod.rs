use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::Infallible;
use axum::{
    Router, Json, extract::State, routing::{get, post},
    response::{IntoResponse, sse::{Sse, Event, KeepAlive}},
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use futures::stream::{self, Stream};

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
        .route("/api/chat/stream", get(chat_stream_handler))
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

/// SSE streaming endpoint: GET /api/chat/stream?message=...&provider=...&model=...
async fn chat_stream_handler(
    State(engine): State<SharedEngine>,
    axum::extract::Query(params): axum::extract::Query<ChatStreamParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);

    // Spawn task to stream response
    let engine_clone = engine.clone();
    tokio::spawn(async move {
        let mut engine = engine_clone.lock().await;
        if let (Some(p), Some(m)) = (&params.provider, &params.model) {
            engine.active_provider = Some(p.clone());
            engine.active_model = Some(m.clone());
        }

        // Send start event
        let _ = tx.send(Ok(Event::default().data(
            serde_json::json!({"event": "start"}).to_string()
        ))).await;

        match engine.process_message_stream(&params.message, None).await {
            Ok(mut rx_stream) => {
                while let Some(chunk) = rx_stream.recv().await {
                    let ev = match chunk {
                        crate::agent::StreamEvent::Chunk(text) => {
                            Event::default().data(
                                serde_json::json!({"event": "chunk", "content": text}).to_string()
                            )
                        }
                        crate::agent::StreamEvent::ToolCall { name, input } => {
                            Event::default().data(
                                serde_json::json!({"event": "tool_call", "name": name, "content": input}).to_string()
                            )
                        }
                        crate::agent::StreamEvent::ToolResult { name, output } => {
                            Event::default().data(
                                serde_json::json!({"event": "tool_result", "name": name, "content": output}).to_string()
                            )
                        }
                        crate::agent::StreamEvent::Done { response } => {
                            Event::default().data(
                                serde_json::json!({"event": "done", "content": response}).to_string()
                            )
                        }
                    };
                    let _ = tx.send(Ok(ev)).await;
                }
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default().data(
                    serde_json::json!({"event": "error", "content": e.to_string()}).to_string()
                ))).await;
            }
        }

        // Send final done event
        let _ = tx.send(Ok(Event::default().data(
            serde_json::json!({"event": "done"}).to_string()
        ))).await;
    });

    Sse::new(stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    })).keep_alive(KeepAlive::default())
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamParams {
    pub message: String,
    pub provider: Option<String>,
    pub model: Option<String>,
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
        "providers": ["github", "groq", "openrouter", "openai", "anthropic", "deepseek", "google"],
        "models": {
            "github": ["gpt-4o", "gpt-4o-mini"],
            "groq": ["llama-3.3-70b-versatile", "llama-3.1-8b-instant"],
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