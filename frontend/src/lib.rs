// Nexus Frontend — Rust (Yew) WASM
// Replaces the entire React/TypeScript frontend.
// Compiled to wasm32-unknown-unknown and loaded by Tauri's webview.

use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub active_provider: String,
    pub active_model: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub message_count: u32,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub enabled: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub connected: bool,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BanditArm {
    pub provider: String,
    pub model: String,
    pub trials: u32,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub avg_cost: f64,
    pub ucb1_score: f64,
}

// ── Tauri IPC helper ─────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

async fn tauri_invoke<T: for<'de> Deserialize<'de>>(cmd: &str, args: JsValue) -> Result<T, String> {
    let result = invoke(cmd, args).await;
    if result.is_null() {
        // void return — inject a default
        return serde_json::from_str("{}").map_err(|e| e.to_string());
    }
    let js_str = js_sys::JSON::stringify(&result).map_err(|e| format!("{:?}", e))?;
    let s: String = js_str.into();
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

fn jsval(obj: &serde_json::Value) -> JsValue {
    let s = serde_json::to_string(obj).unwrap_or_default();
    js_sys::JSON::parse(&s).unwrap_or(JsValue::UNDEFINED)
}

// ── Routes ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/chat")]
    Chat,
    #[at("/sessions")]
    Sessions,
    #[at("/providers")]
    Providers,
    #[at("/skills")]
    Skills,
    #[at("/mcp")]
    Mcp,
    #[at("/gateway")]
    Gateway,
    #[at("/schedules")]
    Schedules,
    #[at("/memory")]
    Memory,
    #[at("/trace")]
    Trace,
    #[at("/bandit")]
    Bandit,
    #[at("/approvals")]
    Approvals,
    #[at("/settings")]
    Settings,
    #[at("/")]
    #[not_found]
    Root,
}

// ── Sidebar ───────────────────────────────────────────────────────────

#[function_component(Sidebar)]
fn sidebar() -> Html {
    let route = use_route::<Route>();
    let current = route.unwrap_or(Route::Root);
    let items = [
        (Route::Chat, "💬", "Chat"),
        (Route::Sessions, "📋", "Sessions"),
        (Route::Providers, "⚡", "Providers"),
        (Route::Skills, "🧠", "Skills"),
        (Route::Mcp, "🔌", "MCP"),
        (Route::Gateway, "🌐", "Gateway"),
        (Route::Schedules, "⏰", "Schedules"),
        (Route::Memory, "💾", "Memory"),
        (Route::Trace, "📊", "Trace"),
        (Route::Bandit, "🎰", "Bandit"),
        (Route::Approvals, "🔒", "Approvals"),
        (Route::Settings, "⚙️", "Settings"),
    ];

    html! {
        <div class="sidebar">
            <div class="sidebar-logo">{ "NEXUS" }</div>
            <div class="sidebar-nav">
                { for items.iter().map(|(route, icon, label)| {
                    let active = *route == current;
                    html! {
                        <Link<Route> to={route.clone()} classes={classes!("sidebar-item", active.then_some("active"))}>
                            {format!("{} {}", icon, label)}
                        </Link<Route>>
                    }
                })}
            </div>
            <div class="sidebar-footer">
                <span class="status-dot on"></span>
                <span>{ "API 18789" }</span>
            </div>
        </div>
    }
}

// ── Screen: Chat ──────────────────────────────────────────────────────

#[function_component(ChatScreen)]
fn chat_screen() -> Html {
    let input = use_state(|| String::new());
    let messages = use_state(|| Vec::<Message>::new());
    let needs_setup = use_state(|| false);
    let streaming = use_state(|| false);
    let streaming_text = use_state(|| String::new());
    let tool_events = use_state(|| Vec::<(String, String)>::new());

    // Check API key presence
    {
        let needs_setup = needs_setup.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let cmd = "get_env";
                let args = jsval(&serde_json::json!({}));
                let envs: Vec<EnvVar> = tauri_invoke(cmd, args).await.unwrap_or_default();
                let has_key = envs.iter().any(|e| e.key.ends_with("_API_KEY") && !e.value.is_empty());
                needs_setup.set(!has_key);
            });
            || ()
        });
    }

    let oninput = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input.set(el.value());
            }
        })
    };

    let onsend = {
        let input = input.clone();
        let messages = messages.clone();
        let streaming = streaming.clone();
        let streaming_text = streaming_text.clone();
        let tool_events = tool_events.clone();
        Callback::from(move |_| {
            let msg = (*input).clone();
            if msg.trim().is_empty() { return; }
            input.set(String::new());
            streaming.set(true);
            streaming_text.set(String::new());
            tool_events.set(Vec::new());

            let mut msgs = (*messages).clone();
            msgs.push(Message { role: "user".into(), content: msg.clone() });
            messages.set(msgs);

            // Open SSE stream from agent server (port 18789)
            let streaming_text = streaming_text.clone();
            let tool_events = tool_events.clone();
            let messages = messages.clone();
            let streaming = streaming.clone();
            let msg_clone = msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let encoded = web_sys::js_sys::encode_uri_component(&msg_clone).as_string().unwrap_or_default();
                let url = format!("http://localhost:18789/api/chat/stream?message={}&provider=github&model=gpt-4o-mini", encoded);
                if let Ok(es_raw) = web_sys::EventSource::new(&url) {
                    let es = std::rc::Rc::new(es_raw);
                    let st = streaming_text.clone();
                    let te = tool_events.clone();
                    let msgs_state = messages.clone();
                    let streaming_state = streaming.clone();
                    let es_for_handler = es.clone();
                    let on_msg = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                        if let Ok(data) = e.data().dyn_into::<js_sys::JsString>() {
                            let s: String = data.into();
                            if s.starts_with("data: ") {
                                let payload = &s[6..];
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                                    if let Some(ev) = v.get("event").and_then(|x| x.as_str()) {
                                        match ev {
                                            "chunk" => {
                                                if let Some(c) = v.get("content").and_then(|x| x.as_str()) {
                                                    st.set(format!("{}{}", *st, c));
                                                }
                                            }
                                            "tool_call" | "tool_result" => {
                                                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                                let content = v.get("content").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                                let mut evs = (*te).clone();
                                                evs.push((format!("{}: {}", ev, name), content));
                                                te.set(evs);
                                            }
                                            "done" => {
                                                let text = (*st).clone();
                                                if !text.is_empty() {
                                                    let mut ms = (*msgs_state).clone();
                                                    ms.push(Message { role: "assistant".into(), content: text });
                                                    msgs_state.set(ms);
                                                }
                                                st.set(String::new());
                                                streaming_state.set(false);
                                                es_for_handler.close();
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }) as Box<dyn FnMut(web_sys::MessageEvent)>);
                    es.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
                    on_msg.forget();
                }
            });
        })
    };

    html! {
        <div class="screen chat-screen">
            { if *needs_setup {
                html! {
                    <div class="setup-banner">
                        { "⚙️ No API key configured yet. Go to " }<strong>{ "Providers" }</strong>{ " to paste an API key." }
                        { " We ship GitHub Models + Groq keys pre-filled so it works out of the box." }
                    </div>
                }
            } else {
                html! {}
            }}
            <div class="messages">
                { for (*messages).iter().map(|m| {
                    html! {
                        <div class={classes!("message", format!("message-{}", m.role))}>
                            <div class="message-avatar">{ if m.role == "user" { "👤" } else { "◆" } }</div>
                            <div class="message-bubble">{ &m.content }</div>
                        </div>
                    }
                })}
                { if *streaming && !(*streaming_text).is_empty() {
                    html! {
                        <div class={classes!("message", "message-assistant")}>
                            <div class="message-avatar">{ "◆" }</div>
                            <div class="message-bubble">{ (*streaming_text).clone() }<span class="cursor-blink">{ "|" }</span></div>
                        </div>
                    }
                } else if *streaming {
                    html! {
                        <div class={classes!("message", "message-assistant")}>
                            <div class="message-avatar">{ "◆" }</div>
                            <div class="message-bubble">
                                <span class="thinking-dots">
                                    <span>{ "." }</span><span>{ "." }</span><span>{ "." }</span>
                                </span>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
                { for (*tool_events).iter().map(|(label, content)| {
                    html! {
                        <div class="tool-event">
                            <span class="tool-event-label">{ label.clone() }</span>
                            <span class="tool-event-content">{ content.clone() }</span>
                        </div>
                    }
                })}
            </div>
            <div class="input-bar">
                <input type="text" class="input-field" value={(*input).clone()} oninput={oninput} placeholder="Type a message..." />
                <button class="btn-primary" onclick={onsend}>{ "Send" }</button>
            </div>
        </div>
    }
}

// ── Screen: Providers ─────────────────────────────────────────────────

#[function_component(ProvidersScreen)]
fn providers_screen() -> Html {
    let provider = use_state(|| "github".to_string());
    let model = use_state(|| "gpt-4o-mini".to_string());
    let api_key = use_state(|| String::new());
    let status = use_state(|| String::new());

    let PROVIDERS = [
        ("github", "GitHub Models (free)", "GITHUB_TOKEN", &["gpt-4o-mini", "gpt-4o"] as &[&str]),
        ("groq", "Groq (free)", "GROQ_API_KEY", &["llama-3.3-70b-versatile", "llama-3.1-8b-instant"]),
        ("openrouter", "OpenRouter", "OPENROUTER_API_KEY", &["anthropic/claude-sonnet-4", "openai/gpt-4o"]),
        ("anthropic", "Anthropic Claude", "ANTHROPIC_API_KEY", &["claude-sonnet-4", "claude-3.5-haiku"]),
        ("openai", "OpenAI", "OPENAI_API_KEY", &["gpt-4o", "gpt-4o-mini"]),
        ("deepseek", "DeepSeek", "DEEPSEEK_API_KEY", &["deepseek-chat", "deepseek-reasoner"]),
        ("google", "Google AI", "GOOGLE_API_KEY", &["gemini-2.0-flash", "gemini-2.5-pro"]),
    ];

    let save_key = {
        let provider = provider.clone();
        let model = model.clone();
        let api_key = api_key.clone();
        let status = status.clone();
        Callback::from(move |_| {
            if (*api_key).trim().is_empty() { status.set("❌ Enter an API key first".into()); return; }
            let pk = (*provider).clone();
            let md = (*model).clone();
            let key = (*api_key).clone();
            let status = status.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Find the env key for this provider
                let env_key = match pk.as_str() {
                    "github" => "GITHUB_TOKEN",
                    "groq" => "GROQ_API_KEY",
                    "openrouter" => "OPENROUTER_API_KEY",
                    "anthropic" => "ANTHROPIC_API_KEY",
                    "openai" => "OPENAI_API_KEY",
                    "deepseek" => "DEEPSEEK_API_KEY",
                    "google" => "GOOGLE_API_KEY",
                    _ => "OPENROUTER_API_KEY",
                };
                // Set env
                let _ = tauri_invoke::<()>("set_env", jsval(&serde_json::json!({"key": env_key, "value": key}))).await;
                // Set active provider
                let _ = tauri_invoke::<()>("set_provider", jsval(&serde_json::json!({"providerId": pk, "model": md}))).await;
                status.set(format!("✅ Active: {} / {}", pk, md));
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Providers" }</h2>
            <div class="config-section">
                <label class="config-label">{ "LLM Provider" }</label>
                <select class="config-select" onchange={{
                    let provider = provider.clone();
                    let model = model.clone();
                    Callback::from(move |e: Event| {
                        let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                        let val = target.value();
                        provider.set(val.clone());
                        // Reset model to first
                        for (id, _, _, models) in &PROVIDERS {
                            if *id == val {
                                model.set(models[0].to_string());
                                break;
                            }
                        }
                    })
                }}>
                    { for PROVIDERS.iter().map(|(id, name, _, _)| {
                        let selected = *id == *provider;
                        html! {
                            <option value={*id} selected={selected}>{ *name }</option>
                        }
                    })}
                </select>
            </div>
            <div class="config-section">
                <label class="config-label">{ "Model" }</label>
                <input class="config-input" type="text" value={(*model).clone()} oninput={{
                    let model = model.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                            model.set(el.value());
                        }
                    })
                }} />
            </div>
            <div class="config-section">
                <label class="config-label">{ "API Key" }</label>
                <input class="config-input" type="password" value={(*api_key).clone()} oninput={{
                    let api_key = api_key.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                            api_key.set(el.value());
                        }
                    })
                }} placeholder="Paste your API key here" />
            </div>
            <button class="btn-primary" onclick={save_key}>{ "Save Key & Activate" }</button>
            if !(*status).is_empty() {
                <p class="status-msg">{ (*status).clone() }</p>
            }
        </div>
    }
}

// ── Screen: Sessions ──────────────────────────────────────────────────

#[function_component(SessionsScreen)]
fn sessions_screen() -> Html {
    let sessions = use_state(|| Vec::<SessionInfo>::new());
    use_effect_with((), {
        let sessions = sessions.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let s: Vec<SessionInfo> = tauri_invoke("list_sessions", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                sessions.set(s);
            });
            || ()
        }
    });

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Sessions" }</h2>
            { for (*sessions).iter().map(|s| {
                html! {
                    <div class="session-card" key={s.id.clone()}>
                        <strong>{ &s.title }</strong>
                        <span class="text-muted">{ format!("{} messages · {}", s.message_count, &s.model) }</span>
                    </div>
                }
            })}
            if sessions.is_empty() {
                <p class="empty-state">{ "No sessions yet." }</p>
            }
        </div>
    }
}

// ── Screen: Skills ────────────────────────────────────────────────────

#[function_component(SkillsScreen)]
fn skills_screen() -> Html {
    let skills = use_state(|| Vec::<SkillInfo>::new());

    use_effect_with((), {
        let skills = skills.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let s: Vec<SkillInfo> = tauri_invoke("list_skills", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                skills.set(s);
            });
            || ()
        }
    });

    let install_name = use_state(|| String::new());

    let install = {
        let skills = skills.clone();
        let install_name = install_name.clone();
        Callback::from(move |_: ()| {
            let name = (*install_name).clone();
            if name.trim().is_empty() { return; }
            let skills = skills.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _: () = tauri_invoke("install_skill", jsval(&serde_json::json!({"name": name}))).await.unwrap_or_default();
                let s: Vec<SkillInfo> = tauri_invoke("list_skills", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                skills.set(s);
            });
        })
    };

    let toggle = {
        let skills = skills.clone();
        Callback::from(move |name: String| {
            let skills = skills.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Find current enabled state
                let current: Vec<SkillInfo> = tauri_invoke("list_skills", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                let enabled = current.iter().find(|s| s.name == name).map(|s| !s.enabled).unwrap_or(true);
                let _: () = tauri_invoke("toggle_skill", jsval(&serde_json::json!({"name": name, "enabled": enabled}))).await.unwrap_or_default();
                let s: Vec<SkillInfo> = tauri_invoke("list_skills", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                skills.set(s);
            });
        })
    };

    let remove = {
        let skills = skills.clone();
        Callback::from(move |name: String| {
            let skills = skills.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _: () = tauri_invoke("remove_skill", jsval(&serde_json::json!({"name": name}))).await.unwrap_or_default();
                let s: Vec<SkillInfo> = tauri_invoke("list_skills", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                skills.set(s);
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Skills" }</h2>
            <div class="config-section">
                <input class="config-input" placeholder="Skill name to install" value={(*install_name).clone()} oninput={{
                    let install_name = install_name.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() { install_name.set(el.value()); }
                    })
                }} />
                <button class="btn-primary" onclick={install}>{ "Install" }</button>
            </div>
            { for (*skills).iter().map(|s| {
                let toggle = toggle.clone();
                let remove = remove.clone();
                let name = s.name.clone();
                let name_remove = s.name.clone();
                html! {
                    <div class="skill-card" key={s.name.clone()}>
                        <div class="skill-header">
                            <strong>{ &s.name }</strong>
                            <span class={if s.enabled { "status-tag connected" } else { "status-tag" }}>{ if s.enabled { "Enabled" } else { "Disabled" } }</span>
                            <span class="text-muted">{ format!("v{}", &s.version) }</span>
                        </div>
                        <p class="text-muted">{ &s.description }</p>
                        { if !s.tags.is_empty() {
                            html! { <div class="trace-tags">{ for s.tags.iter().map(|t| html! { <span class="tag">{ t.clone() }</span> }) }</div> }
                        } else { html! {} } }
                        <div class="skill-actions">
                            <button class="btn-sm" onclick={Callback::from(move |_| toggle.emit(name.clone()))}>{ if s.enabled { "Disable" } else { "Enable" } }</button>
                            <button class="btn-sm" onclick={Callback::from(move |_| remove.emit(name_remove.clone()))}>{ "Remove" }</button>
                        </div>
                    </div>
                }
            })}
            if skills.is_empty() {
                <p class="empty-state">{ "No skills installed." }</p>
            }
        </div>
    }
}

// ── Screen: MCP ───────────────────────────────────────────────────────

#[function_component(McpScreen)]
fn mcp_screen() -> Html {
    let servers = use_state(|| Vec::<McpServerInfo>::new());
    let new_name = use_state(|| String::new());
    let new_command = use_state(|| String::new());
    let new_args = use_state(|| String::new());

    let refresh = {
        let servers = servers.clone();
        Callback::from(move |_: ()| {
            let servers = servers.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let s: Vec<McpServerInfo> = tauri_invoke("list_mcp_servers", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                servers.set(s);
            });
        })
    };

    use_effect_with((), {
        let servers = servers.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let s: Vec<McpServerInfo> = tauri_invoke("list_mcp_servers", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                servers.set(s);
            });
            || ()
        }
    });

    let add_server = {
        let new_name = new_name.clone();
        let new_command = new_command.clone();
        let new_args = new_args.clone();
        let servers = servers.clone();
        Callback::from(move |_| {
            let name = (*new_name).clone();
            let cmd = (*new_command).clone();
            let args_str = (*new_args).clone();
            let args: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
            let servers = servers.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = tauri_invoke::<()>("add_mcp_server", jsval(&serde_json::json!({
                    "config": { "name": name, "command": cmd, "args": args, "env": {} }
                }))).await;
                // Refresh
                let s: Vec<McpServerInfo> = tauri_invoke("list_mcp_servers", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                servers.set(s);
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "MCP" }</h2>

            <div class="config-section">
                <label class="config-label">{ "Add MCP Server" }</label>
                <input class="config-input" placeholder="Name" value={(*new_name).clone()} oninput={{
                    let new_name = new_name.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() { new_name.set(el.value()); }
                    })
                }} />
                <input class="config-input" placeholder="Command (e.g. npx)" value={(*new_command).clone()} oninput={{
                    let new_command = new_command.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() { new_command.set(el.value()); }
                    })
                }} />
                <input class="config-input" placeholder="Args (space-separated)" value={(*new_args).clone()} oninput={{
                    let new_args = new_args.clone();
                    Callback::from(move |e: InputEvent| {
                        if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() { new_args.set(el.value()); }
                    })
                }} />
                <button class="btn-primary" onclick={add_server}>{ "Add" }</button>
            </div>

            { for (*servers).iter().map(|s| {
                let servers = servers.clone();
                let s_name = s.name.clone();
                html! {
                    <div class="mcp-server-card" key={s.name.clone()}>
                        <strong>{ &s.name }</strong>
                        <span class={if s.connected { "status-tag connected" } else { "status-tag" }}>{ if s.connected { "Connected" } else { "Disconnected" } }</span>
                        <div class="mcp-server-tools">
                            { for s.tools.iter().map(|t| html! { <span class="tool-tag">{ t }</span> }) }
                        </div>
                        <button class="btn-sm" onclick={Callback::from(move |_| {
                            let servers = servers.clone();
                            let s_name_clone = s_name.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = tauri_invoke::<()>("remove_mcp_server", jsval(&serde_json::json!({"name": s_name_clone.clone()}))).await;
                                let s2: Vec<McpServerInfo> = tauri_invoke("list_mcp_servers", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                                servers.set(s2);
                            });
                        })}>{ "Remove" }</button>
                    </div>
                }
            })}
            if servers.is_empty() {
                <p class="empty-state">{ "No MCP servers configured." }</p>
            }
        </div>
    }
}

// ── Screen: Trace ─────────────────────────────────────────────────────

#[function_component(TraceScreen)]
fn trace_screen() -> Html {
    let events = use_state(|| Vec::<TraceEvent>::new());

    use_effect_with((), {
        let events = events.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let e: Vec<TraceEvent> = tauri_invoke("trace_query", jsval(&serde_json::json!({"limit": 50}))).await.unwrap_or_default();
                events.set(e);
            });
            || ()
        }
    });

    let clear = {
        let events = events.clone();
        Callback::from(move |_: ()| {
            let events = events.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _: () = tauri_invoke("trace_clear", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                events.set(Vec::new());
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Trace" }</h2>
            <p class="text-muted">{ "Full request/response/tool-call lifecycle tracing." }</p>
            <button class="btn-sm" onclick={clear}>{ "Clear" }</button>
            { for (*events).iter().map(|e| {
                html! {
                    <div class="trace-event" key={e.id}>
                        <div class="trace-header">
                            <span class="trace-type">{ &e.event_type }</span>
                            { if let Some(p) = &e.provider { html! { <span class="trace-provider">{ p.clone() }</span> } } else { html! {} } }
                            { if let Some(m) = &e.model { html! { <span class="trace-model">{ m.clone() }</span> } } else { html! {} } }
                            <span class="trace-duration">{ format!("{:.0}ms", e.duration_ms) }</span>
                        </div>
                        <div class="trace-summary">{ &e.summary }</div>
                        <div class="text-muted">{ &e.detail }</div>
                        { if !e.tags.is_empty() {
                            html! { <div class="trace-tags">{ for e.tags.iter().map(|t| html! { <span class="tag">{ t.clone() }</span> }) }</div> }
                        } else { html! {} } }
                    </div>
                }
            })}
            if events.is_empty() {
                <p class="empty-state">{ "No trace events. Send a message to populate." }</p>
            }
        </div>
    }
}

// ── Screen: Bandit ────────────────────────────────────────────────────

#[function_component(BanditScreen)]
fn bandit_screen() -> Html {
    let arms = use_state(|| Vec::<BanditArm>::new());
    let selected = use_state(|| String::new());

    {
        let arms = arms.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let a: Vec<BanditArm> = tauri_invoke("bandit_stats", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                arms.set(a);
            });
            || ()
        });
    }

    let auto_select = {
        let arms = arms.clone();
        let selected = selected.clone();
        Callback::from(move |_: ()| {
            let arms = arms.clone();
            let selected = selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let r: serde_json::Value = tauri_invoke("bandit_select", jsval(&serde_json::json!({"preferred": null}))).await.unwrap_or(serde_json::json!({}));
                if let (Some(p), Some(m)) = (r.get("provider").and_then(|x| x.as_str()), r.get("model").and_then(|x| x.as_str())) {
                    selected.set(format!("{} / {}", p, m));
                    // Refresh stats
                    let a: Vec<BanditArm> = tauri_invoke("bandit_stats", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                    arms.set(a);
                }
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Bandit" }</h2>
            <p class="text-muted">{ "UCB1 multi-armed bandit auto-routes to the best (provider, model) arm by balancing exploration vs. exploitation." }</p>
            <button class="btn-primary" onclick={auto_select}>{ "Auto-select Best Arm" }</button>
            { if !(*selected).is_empty() {
                html! { <div class="setup-banner">{ "Selected: " }<strong>{ (*selected).clone() }</strong></div> }
            } else { html! {} }}
            <table class="bandit-table">
                <thead><tr><th>{ "Provider" }</th><th>{ "Model" }</th><th>{ "Trials" }</th><th>{ "Success" }</th><th>{ "Avg Latency" }</th><th>{ "Avg Cost" }</th><th>{ "UCB1" }</th></tr></thead>
                <tbody>
                    { for (*arms).iter().map(|a| {
                        html! {
                            <tr key={format!("{}-{}", a.provider, a.model)}>
                                <td>{ &a.provider }</td>
                                <td>{ &a.model }</td>
                                <td>{ a.trials }</td>
                                <td>{ format!("{:.1}%", a.success_rate * 100.0) }</td>
                                <td>{ format!("{:.0}ms", a.avg_latency_ms) }</td>
                                <td>{ format!("${:.4}", a.avg_cost) }</td>
                                <td>{ format!("{:.3}", a.ucb1_score) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
            if arms.is_empty() {
                <p class="empty-state">{ "No bandit data yet. Send a message to populate arms." }</p>
            }
        </div>
    }
}

// ── Screen: Gateway ───────────────────────────────────────────────────

#[function_component(GatewayScreen)]
fn gateway_screen() -> Html {
    let gateways = use_state(|| Vec::<GatewayInfo>::new());

    {
        let gateways = gateways.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let g: Vec<GatewayInfo> = tauri_invoke("list_gateways", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                gateways.set(g);
            });
            || ()
        });
    }

    let toggle = {
        let gateways = gateways.clone();
        Callback::from(move |(id, enable): (String, bool)| {
            let gateways = gateways.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _: () = tauri_invoke("toggle_gateway", jsval(&serde_json::json!({"id": id, "enable": enable}))).await.unwrap_or_default();
                let g: Vec<GatewayInfo> = tauri_invoke("list_gateways", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                gateways.set(g);
            });
        })
    };

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Gateway" }</h2>
            <p class="text-muted">{ "Connect Nexus to external messaging platforms (Telegram, Discord, etc.)." }</p>
            { for (*gateways).iter().map(|g| {
                let toggle = toggle.clone();
                let id = g.id.clone();
                let will_enable = !g.enabled;
                html! {
                    <div class="mcp-server-card" key={g.id.clone()}>
                        <div class="skill-header">
                            <strong>{ &g.name }</strong>
                            <span class={if g.connected { "status-tag connected" } else { "status-tag" }}>{ if g.connected { "Connected" } else { "Offline" } }</span>
                            <span class="text-muted">{ &g.platform }</span>
                        </div>
                        <div class="skill-actions">
                            <button class="btn-sm" onclick={Callback::from(move |_| toggle.emit((id.clone(), will_enable)))}>{ if g.enabled { "Disable" } else { "Enable" } }</button>
                        </div>
                    </div>
                }
            })}
            if gateways.is_empty() {
                <p class="empty-state">{ "No gateways configured." }</p>
            }
        </div>
    }
}

// ── Screen: Schedules ─────────────────────────────────────────────────

#[function_component(SchedulesScreen)]
fn schedules_screen() -> Html {
    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Schedules" }</h2>
            <p class="empty-state">{ "Schedules — coming soon." }</p>
        </div>
    }
}

// ── Screen: Memory ────────────────────────────────────────────────────

#[function_component(MemoryScreen)]
fn memory_screen() -> Html {
    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Memory" }</h2>
            <p class="empty-state">{ "Memory — coming soon." }</p>
        </div>
    }
}

// ── Screen: Approvals ─────────────────────────────────────────────────

#[function_component(ApprovalsScreen)]
fn approvals_screen() -> Html {
    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Approvals" }</h2>
            <p class="empty-state">{ "No pending approvals." }</p>
        </div>
    }
}

// ── Screen: Settings ──────────────────────────────────────────────────

#[function_component(SettingsScreen)]
fn settings_screen() -> Html {
    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Settings" }</h2>
            <div class="config-info">
                { "Nexus v0.1.0 — The Core of Your AGI." }
            </div>
        </div>
    }
}

// ── App ───────────────────────────────────────────────────────────────

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app">
            <div class="bg-grid"></div>
            <div class="bg-glow"></div>
            <BrowserRouter>
                <Sidebar />
                <main class="main-content">
                    <Switch<Route> render={switch} />
                </main>
                <footer class="status-bar">
                    <span class="status-item">{ "v0.1.0" }</span>
                    <span class="status-item">{ "Rust" }</span>
                    <span class="status-item">{ "API: 18789" }</span>
                </footer>
            </BrowserRouter>
        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Chat => html! { <ChatScreen /> },
        Route::Sessions => html! { <SessionsScreen /> },
        Route::Providers => html! { <ProvidersScreen /> },
        Route::Skills => html! { <SkillsScreen /> },
        Route::Mcp => html! { <McpScreen /> },
        Route::Gateway => html! { <GatewayScreen /> },
        Route::Schedules => html! { <SchedulesScreen /> },
        Route::Memory => html! { <MemoryScreen /> },
        Route::Trace => html! { <TraceScreen /> },
        Route::Bandit => html! { <BanditScreen /> },
        Route::Approvals => html! { <ApprovalsScreen /> },
        Route::Settings => html! { <SettingsScreen /> },
        Route::Root => html! { <ChatScreen /> },
    }
}

// ── Entry point ───────────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}