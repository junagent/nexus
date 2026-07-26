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

    // Listen for Tauri streaming events: nexus://stream/chunk, tool_call, tool_result, done
    {
        let streaming_text = streaming_text.clone();
        let tool_events = tool_events.clone();
        let messages = messages.clone();
        let streaming = streaming.clone();
        use_effect_with((), move |_| {
            // Tauri event listener via wasm-bindgen
            let window = web_sys::window().unwrap();
            let listener = Closure::wrap(Box::new(move |event: js_sys::JsString| {
                let s: String = event.into();
                // Parse event name + payload from the custom event
                if s.starts_with("nexus://stream/chunk:") {
                    let chunk = &s["nexus://stream/chunk:".len()..];
                    streaming_text.set(format!("{}{}", *streaming_text, chunk));
                } else if s.starts_with("nexus://stream/done:") {
                    // Finalize streaming text into a message
                    let text = (*streaming_text).clone();
                    if !text.is_empty() {
                        let mut msgs = (*messages).clone();
                        msgs.push(Message { role: "assistant".into(), content: text });
                        messages.set(msgs);
                    }
                    streaming_text.set(String::new());
                    streaming.set(false);
                }
            }) as Box<dyn FnMut(js_sys::JsString)>);
            window.add_event_listener_with_callback_and_bool_and_bool_and_bool(
                "nexus-stream", listener.as_ref().as_ref(), false
            ).unwrap();
            // Keep listener alive
            listener.forget();
            || ()
        });
    }

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
            let target = e.target_dyn_into::<web_sys::HtmlInputElement>();
            if let Some(el) = target {
                input.set(el.value());
            }
        })
    };

    let onsend = {
        let input = input.clone();
        let messages = messages.clone();
        let needs_setup = needs_setup.clone();
        let streaming = streaming.clone();
        Callback::from(move |_| {
            let msg = (*input).clone();
            if msg.trim().is_empty() { return; }
            input.set(String::new());
            streaming.set(true);
            let mut msgs = (*messages).clone();
            msgs.push(Message { role: "user".into(), content: msg.clone() });
            messages.set(msgs);
            // Call chat_stream via Tauri invoke — backend emits nexus://stream/* events
            wasm_bindgen_futures::spawn_local(async move {
                let _ = tauri_invoke::<String>("chat_stream", jsval(&serde_json::json!({
                    "request": { "message": msg, "sessionId": None, "model": None }
                }))).await;
            });
        })
    };

    html! {
        <div class="screen chat-screen">
            { if *needs_setup {
                html! {
                    <div class="setup-banner">
                        { "⚙️ No API key configured yet. Go to " }<strong>{ "Providers" }</strong>{ " to paste an API key." }
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
                            <div class="message-avatar">◆</div>
                            <div class="message-bubble">{ (*streaming_text).clone() }<span class="cursor-blink">|</span></div>
                        </div>
                    }
                } else if *streaming {
                    html! {
                        <div class={classes!("message", "message-assistant")}>
                            <div class="message-avatar">◆</div>
                            <div class="message-bubble">
                                <span class="thinking-dots">
                                    <span>.</span><span>.</span><span>.</span>
                                </span>
                            </div>
                        </div>
                    }
                }}
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

    html! {
        <div class="screen">
            <h2 class="screen-title">{ "MCP" }</h2>
            { for (*servers).iter().map(|s| {
                html! {
                    <div class="mcp-server-card" key={s.name.clone()}>
                        <strong>{ &s.name }</strong>
                        <span class={if s.connected { "status-tag connected" } else { "status-tag" }}>{ if s.connected { "Connected" } else { "Disconnected" } }</span>
                        <div class="mcp-server-tools">
                            { for s.tools.iter().map(|t| html! { <span class="tool-tag">{ t }</span> }) }
                        </div>
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