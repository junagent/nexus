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
    pub enabled: bool,
    pub description: String,
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
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BanditArm {
    pub provider: String,
    pub model: String,
    pub pulls: u32,
    pub successes: u32,
    pub failures: u32,
    pub rate: f64,
    pub cost: f64,
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

    // Helper: append assistant message from streaming buffer
    let finalize = {
        let streaming_text = streaming_text.clone();
        let messages = messages.clone();
        let streaming = streaming.clone();
        Callback::from(move |_| {
            let text = (*streaming_text).clone();
            if !text.is_empty() {
                let mut msgs = (*messages).clone();
                msgs.push(Message { role: "assistant".into(), content: text });
                messages.set(msgs);
            }
            streaming_text.set(String::new());
            streaming.set(false);
        })
    };

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

            // Open SSE stream from agent server
            let streaming_text = streaming_text.clone();
            let tool_events = tool_events.clone();
            // Open SSE stream from agent server (port 18789)
            wasm_bindgen_futures::spawn_local(async move {
                let encoded = web_sys::js_sys::encode_uri_component(&msg).as_string().unwrap_or_default();
                let url = format!("http://localhost:18789/api/chat/stream?message={}&provider=github&model=gpt-4o-mini", encoded);
                if let Ok(es) = web_sys::EventSource::new(&url) {
                    let st = streaming_text.clone();
                    let te = tool_events.clone();
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
                                                finalize.emit(());
                                                es.close();
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

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Skills" }</h2>
            { for (*skills).iter().map(|s| {
                html! {
                    <div class="skill-card" key={s.name.clone()}>
                        <strong>{ &s.name }</strong>
                        <span class={if s.enabled { "status-tag connected" } else { "status-tag" }}>{ if s.enabled { "Enabled" } else { "Disabled" } }</span>
                        <p class="text-muted">{ &s.description }</p>
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
        Callback::from(move |_| {
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
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = tauri_invoke::<()>("remove_mcp_server", jsval(&serde_json::json!({"name": s_name.clone()}))).await;
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

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Trace" }</h2>
            { for (*events).iter().map(|e| {
                html! {
                    <div class="trace-event" key={e.id.clone()}>
                        <span class="trace-type">{ &e.event_type }</span>
                        <span class="text-muted">{ &e.detail }</span>
                    </div>
                }
            })}
            if events.is_empty() {
                <p class="empty-state">{ "No trace events." }</p>
            }
        </div>
    }
}

// ── Screen: Bandit ────────────────────────────────────────────────────

#[function_component(BanditScreen)]
fn bandit_screen() -> Html {
    let arms = use_state(|| Vec::<BanditArm>::new());

    use_effect_with((), {
        let arms = arms.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let a: Vec<BanditArm> = tauri_invoke("bandit_stats", jsval(&serde_json::json!({}))).await.unwrap_or_default();
                arms.set(a);
            });
            || ()
        }
    });

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Bandit" }</h2>
            <table class="bandit-table">
                <thead><tr><th>{ "Provider" }</th><th>{ "Model" }</th><th>{ "Pulls" }</th><th>{ "Rate" }</th></tr></thead>
                <tbody>
                    { for (*arms).iter().map(|a| {
                        html! {
                            <tr key={format!("{}-{}", a.provider, a.model)}>
                                <td>{ &a.provider }</td>
                                <td>{ &a.model }</td>
                                <td>{ a.pulls }</td>
                                <td>{ format!("{:.1}%", a.rate * 100.0) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
            if arms.is_empty() {
                <p class="empty-state">{ "No bandit data yet." }</p>
            }
        </div>
    }
}

// ── Screen: Gateway ───────────────────────────────────────────────────

#[function_component(GatewayScreen)]
fn gateway_screen() -> Html {
    html! {
        <div class="screen">
            <h2 class="screen-title">{ "Gateway" }</h2>
            <p class="empty-state">{ "Gateways — coming soon." }</p>
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