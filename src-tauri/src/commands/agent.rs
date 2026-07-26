use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Emitter};
use tokio::sync::Mutex;
use crate::agent::NexusEngine;
use crate::providers;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub status: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub chunk: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEvent {
    pub session_id: String,
    pub tool_name: String,
    pub status: String,
    pub arguments: String,
    pub result: Option<String>,
}

#[tauri::command]
pub async fn chat(
    engine: State<'_, Mutex<NexusEngine>>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let mut engine = engine.lock().await;
    engine
        .process_message(&request.message, request.session_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_stream(
    app_handle: AppHandle,
    engine: State<'_, Mutex<NexusEngine>>,
    request: ChatRequest,
) -> Result<String, String> {
    let mut engine = engine.lock().await;
    let sid = request.session_id.clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Create session if needed (must come before pushing the user message)
    if !engine.sessions.contains_key(&sid) {
        let now = chrono::Utc::now().to_rfc3339();
        let session_model = engine.active_model.clone().unwrap_or_default();
        engine.sessions.insert(sid.clone(), crate::commands::sessions::SessionInfo {
            id: sid.clone(), title: "New Chat".into(),
            message_count: 0, created_at: now.clone(), updated_at: now,
            model: session_model,
        });
        engine.conversations.insert(sid.clone(), vec![]);
    }

    // Add user message
    {
        if let Some(msgs) = engine.conversations.get_mut(&sid) {
            msgs.push(providers::ChatMessage { role: "user".into(), content: request.message.clone() });
        }
        if let Some(ref memory) = engine.memory {
            let _ = memory.save_message(&sid, "user", &request.message);
        }
        if let Some(s) = engine.sessions.get_mut(&sid) {
            s.message_count += 1;
        }
    }

    // Resolve provider/model. Priority:
    // 1) explicitly-set active provider/model
    // 2) auto-detect from whichever API key is configured, using the
    //    model the frontend sent (or a sane default for that provider)
    let (provider, model) = match (&engine.active_provider, &engine.active_model) {
        (Some(p), Some(m)) => (p.clone(), m.clone()),
        _ => {
            match providers::auto_detect_provider() {
                Some((p, default_model)) => {
                    // Honor the frontend-selected model when it looks compatible,
                    // otherwise use the provider's default.
                    let req_model = request.model.clone().unwrap_or_default();
                    let m = if req_model.is_empty() { default_model } else { req_model };
                    // Remember the choice for subsequent turns.
                    engine.active_provider = Some(p.clone());
                    engine.active_model = Some(m.clone());
                    (p, m)
                }
                None => {
                    let msg = "⚠️ No API key configured. Open **Providers** and paste an API key (OpenRouter, OpenAI, Anthropic, DeepSeek, or Google), then click Save.";
                    let _ = app_handle.emit("nexus://stream/chunk", StreamChunk {
                        chunk: msg.to_string(), session_id: sid.clone(),
                    });
                    let _ = app_handle.emit("nexus://stream/done", serde_json::json!({
                        "session_id": sid, "tool_calls": Vec::<ToolCallInfo>::new(),
                    }));
                    return Ok(msg.to_string());
                }
            }
        }
    };

    let (base_url, api_key) = match providers::get_provider_config(&provider) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("⚠️ {}. Open **Providers** to set your API key.", e);
            let _ = app_handle.emit("nexus://stream/chunk", StreamChunk {
                chunk: msg.clone(), session_id: sid.clone(),
            });
            let _ = app_handle.emit("nexus://stream/done", serde_json::json!({
                "session_id": sid, "tool_calls": Vec::<ToolCallInfo>::new(),
            }));
            return Ok(msg);
        }
    };

    // Build messages
    let mut msgs = vec![providers::ChatMessage {
        role: "system".into(),
        content: format!("You are Nexus v{}, a desktop AI agent.", env!("CARGO_PKG_VERSION")),
    }];
    if let Some(history) = engine.conversations.get(&sid) {
        let start = if history.len() > 20 { history.len() - 20 } else { 0 };
        msgs.extend(history[start..].to_vec());
    }

    // Get tools
    let tools = engine.tool_registry.to_openai_tools();

    // Emit stream start
    let _ = app_handle.emit("nexus://stream/start", serde_json::json!({
        "session_id": sid
    }));

    // Try streaming with tools
    let (full_text, executed_tools) = match providers::chat_with_tools_stream(
        &base_url, &api_key, &model, &msgs, &tools,
        |chunk| {
            let _ = app_handle.emit("nexus://stream/chunk", StreamChunk {
                chunk: chunk.to_string(),
                session_id: sid.clone(),
            });
        },
    ).await {
        Ok((text, calls)) => {
            let mut executed = Vec::new();
            for call in &calls {
                // Emit tool call event
                let _ = app_handle.emit("nexus://stream/tool_call", ToolEvent {
                    session_id: sid.clone(),
                    tool_name: call.name.clone(),
                    status: "started".into(),
                    arguments: call.arguments.clone(),
                    result: None,
                });

                if let Some(tool) = engine.tool_registry.get(&call.name) {
                    let args: serde_json::Value = serde_json::from_str(&call.arguments).unwrap_or_default();
                    match tool.execute(args).await {
                        Ok(result) => {
                            let _ = app_handle.emit("nexus://stream/tool_result", ToolEvent {
                                session_id: sid.clone(),
                                tool_name: call.name.clone(),
                                status: "success".into(),
                                arguments: call.arguments.clone(),
                                result: Some(result.clone()),
                            });
                            msgs.push(providers::ChatMessage { role: "tool".into(), content: result });
                            executed.push(ToolCallInfo { name: call.name.clone(), status: "success".into(), duration_ms: None });
                        }
                        Err(e) => {
                            let _ = app_handle.emit("nexus://stream/tool_result", ToolEvent {
                                session_id: sid.clone(),
                                tool_name: call.name.clone(),
                                status: "error".into(),
                                arguments: call.arguments.clone(),
                                result: Some(e.to_string()),
                            });
                            executed.push(ToolCallInfo { name: call.name.clone(), status: format!("error: {}", e), duration_ms: None });
                        }
                    }
                }
            }

            if !executed.is_empty() {
                // Get final response after tools
                match providers::chat_stream(&base_url, &api_key, &model, &msgs, |chunk| {
                    let _ = app_handle.emit("nexus://stream/chunk", StreamChunk {
                        chunk: chunk.to_string(),
                        session_id: sid.clone(),
                    });
                }).await {
                    Ok(final_text) => (final_text, executed),
                    Err(e) => (format!("Tool execution error: {}", e), executed),
                }
            } else {
                (text, executed)
            }
        }
        Err(_) => {
            // Fallback to plain chat
            match providers::chat_stream(&base_url, &api_key, &model, &msgs, |chunk| {
                let _ = app_handle.emit("nexus://stream/chunk", StreamChunk {
                    chunk: chunk.to_string(),
                    session_id: sid.clone(),
                });
            }).await {
                Ok(text) => (text, vec![]),
                Err(e) => (format!("Error: {}", e), vec![]),
            }
        }
    };

    // Store assistant message
    {
        let msgs = engine.conversations.get_mut(&sid);
        if let Some(msgs) = msgs {
            msgs.push(providers::ChatMessage { role: "assistant".into(), content: full_text.clone() });
        }
        if let Some(ref memory) = engine.memory {
            let _ = memory.save_message(&sid, "assistant", &full_text);
        }
        if let Some(s) = engine.sessions.get_mut(&sid) {
            s.message_count += 1;
            s.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    // Record to bandit
    let latency = 0.0; // TODO: track actual latency
    let cost = crate::bandit::estimate_cost(&provider, &model, 200, 500);
    let ok = executed_tools.iter().any(|t| t.status == "success") || !full_text.starts_with("⚠️");
    if ok {
        engine.bandit.record_success(&provider, &model, latency, cost);
    } else {
        engine.bandit.record_failure(&provider, &model, latency, cost);
    }

    // Trace recording
    engine.trace_store.record_llm_request(&sid, &provider, &model, &request.message);
    engine.trace_store.record_llm_response(&sid, &provider, &model, &full_text, latency);
    for tc in &executed_tools {
        engine.trace_store.record_tool_result(&sid, &tc.name, &tc.status, tc.duration_ms.unwrap_or(0) as f64, tc.status == "success");
    }

    // Emit done
    let _ = app_handle.emit("nexus://stream/done", serde_json::json!({
        "session_id": sid,
        "tool_calls": executed_tools,
    }));

    Ok(full_text)
}

#[tauri::command]
pub async fn get_providers(
    engine: State<'_, Mutex<NexusEngine>>,
) -> Result<Vec<ProviderInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.list_providers())
}

#[tauri::command]
pub async fn set_provider(
    engine: State<'_, Mutex<NexusEngine>>,
    provider_id: String,
    model: String,
) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_active_provider(&provider_id, &model).await;
    Ok(())
}