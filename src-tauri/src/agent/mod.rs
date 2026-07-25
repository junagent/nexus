use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::providers;
use crate::tools::{ToolRegistry, ReadFileTool, WriteFileTool, ListDirTool, ShellTool, WebFetchTool};
use crate::commands::{
    agent::{ChatResponse, ProviderInfo, ToolCallInfo},
    config::{EnvVar, NexusConfig},
    sessions::SessionInfo,
    gateway::GatewayInfo,
};

// ---- Memory System ----

/// Simple SQLite-backed memory store.
pub struct MemoryStore {
    db: std::sync::Mutex<rusqlite::Connection>,
}

impl MemoryStore {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                model TEXT,
                created_at TEXT,
                updated_at TEXT
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                role TEXT,
                content TEXT,
                timestamp TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );"
        ).map_err(|e| e.to_string())?;
        Ok(Self { db: std::sync::Mutex::new(conn) })
    }

    pub fn save_message(&self, session_id: &str, role: &str, content: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, role, content, chrono::Utc::now().to_rfc3339()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_history(&self, session_id: &str, limit: usize) -> Vec<providers::ChatMessage> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id DESC LIMIT ?2"
        ).unwrap();
        let mut msgs: Vec<providers::ChatMessage> = stmt
            .query_map(rusqlite::params![session_id, limit as i64], |row| {
                Ok(providers::ChatMessage {
                    role: row.get(0)?,
                    content: row.get(1)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        msgs.reverse();
        msgs
    }
}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStore").finish()
    }
}

// ---- Engine ----

#[derive(Debug)]
pub struct NexusEngine {
    pub running: bool,
    pub uptime: Instant,
    pub active_provider: Option<String>,
    pub active_model: Option<String>,
    pub config: NexusConfig,
    pub env_vars: HashMap<String, String>,
    pub gateways: HashMap<String, GatewayInfo>,
    pub sessions: HashMap<String, SessionInfo>,
    pub session_count: u32,
    pub conversations: HashMap<String, Vec<providers::ChatMessage>>,
    pub tool_registry: ToolRegistry,
    pub memory: Option<MemoryStore>,
    pub mcp_client: crate::mcp::McpClient,
    pub bandit: crate::bandit::BanditSelector,
    pub skill_store: crate::skill_store::SkillStore,
    pub trace_store: crate::trace::TraceStore,
}

impl NexusEngine {
    pub fn new_with_tools() -> Self {
        let mut engine = Self::new();
        
        // Register built-in tools
        engine.tool_registry.register(ReadFileTool);
        engine.tool_registry.register(WriteFileTool);
        engine.tool_registry.register(ListDirTool);
        engine.tool_registry.register(ShellTool);
        engine.tool_registry.register(WebFetchTool);

        // Initialize memory store
        let db_path = dirs_next().join("nexus").join("memory.db");
        engine.memory = MemoryStore::new(&db_path.to_string_lossy()).ok();

        // Load MCP config and try to connect
        let mcp_path = dirs_next().join("nexus").join("mcp.json");
        if mcp_path.exists() {
            engine.mcp_client.load_config(&mcp_path);
            let servers: Vec<String> = engine.mcp_client.list_servers().iter()
                .map(|s| s.name.clone()).collect();
            tracing::info!("MCP servers configured: {:?}", servers);
        }

        // Initialize bandit selector with proper db path
        let bandit_db = dirs_next().join("nexus").join("bandit.db");
        engine.bandit = crate::bandit::BanditSelector::new(&bandit_db.to_string_lossy());

        // Initialize skill store
        let skills_dir = dirs_next().join("nexus").join("skills");
        engine.skill_store = crate::skill_store::SkillStore::load(&skills_dir);

        // Initialize trace store
        engine.trace_store = crate::trace::TraceStore::new(1000);

        // Register all provider arms in the bandit selector
        for (provider, models) in &[
            ("anthropic", &["claude-sonnet-4", "claude-3.5-haiku"] as &[&str]),
            ("openai", &["gpt-4o", "gpt-4o-mini", "o3-mini"]),
            ("deepseek", &["deepseek-chat", "deepseek-reasoner"]),
            ("openrouter", &["anthropic/claude-sonnet-4", "openai/gpt-4o", "google/gemini-2.0-flash"]),
            ("google", &["gemini-2.0-flash", "gemini-2.5-pro"]),
        ] {
            for model in *models {
                engine.bandit.register_arm(provider, model);
            }
        }

        tracing::info!(
            "Tools registered: {:?} | Memory: {} | MCP servers: {} | Bandit arms: {}",
            engine.tool_registry.list(),
            engine.memory.is_some(),
            engine.mcp_client.list_servers().len(),
            engine.bandit.summary().len()
        );

        engine
    }

    pub fn new() -> Self {
        let data_dir = dirs_next()
            .join("nexus")
            .to_string_lossy()
            .to_string();

        let env_path = dirs_next().join("nexus").join(".env");
        let env_vars = load_env_file(&env_path);

        for (k, v) in &env_vars {
            std::env::set_var(k, v);
        }

        Self {
            running: true,
            uptime: Instant::now(),
            active_provider: None,
            active_model: None,
            config: NexusConfig {
                theme: "dark".into(),
                font_size: 14,
                language: "en".into(),
                auto_start: false,
                data_dir,
            },
            env_vars,
            gateways: HashMap::new(),
            sessions: HashMap::new(),
            session_count: 0,
            conversations: HashMap::new(),
            tool_registry: ToolRegistry::new(),
            memory: None,
            mcp_client: crate::mcp::McpClient::new(),
                        bandit: crate::bandit::BanditSelector::new(""),
                        skill_store: crate::skill_store::SkillStore::load(&std::path::Path::new("")),
                        trace_store: crate::trace::TraceStore::new(1000usize),
                    }
    }

    pub fn save_config(&self) {
        let mcp_path = dirs_next().join("nexus").join("mcp.json");
        let _ = self.mcp_client.save_config(&mcp_path);
    }

    pub async fn process_message(
        &mut self,
        message: &str,
        session_id: Option<&str>,
    ) -> Result<ChatResponse, anyhow::Error> {
        let sid = session_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.create_session());

        // Add user message directly (not via self to avoid borrow conflict)
        {
            let msgs = self.conversations.get_mut(&sid);
            if let Some(msgs) = msgs {
                msgs.push(providers::ChatMessage { role: "user".into(), content: message.to_string() });
            }
            if let Some(ref memory) = self.memory {
                let _ = memory.save_message(&sid, "user", message);
            }
            if let Some(s) = self.sessions.get_mut(&sid) {
                s.message_count += 1;
                s.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }

        let start = std::time::Instant::now();
        let (provider, model) = match (&self.active_provider, &self.active_model) {
            (Some(p), Some(m)) => (p.clone(), m.clone()),
            _ => (String::new(), String::new()),
        };
        let tool_list = self.tool_registry.list();
        let memory_status = if self.memory.is_some() { "active" } else { "disabled" };
        let bandit_count = self.bandit.summary().len();

        // Trace: LLM request
        if !provider.is_empty() {
            self.trace_store.record_llm_request(&sid, &provider, &model, message);
        }

        let (result, tool_calls, selected_provider, selected_model) = {
            if provider.is_empty() {
                (format!(
                    "🤖 **Nexus v{}**\n\n⚙️ Configure a provider in Engine Config to get started.\n🔧 {} tools loaded: {}\n💾 Memory: {}\n🧠 Bandit: {} arms tracked",
                    env!("CARGO_PKG_VERSION"),
                    tool_list.len(),
                    tool_list.join(", "),
                    memory_status,
                    bandit_count,
                ), vec![], String::new(), String::new())
            } else {
                match providers::get_provider_config(&provider) {
                    Ok((base_url, api_key)) => {
                        let mut msgs = self.build_messages(&sid);

                        // Try with tool calling
                        let tools = self.tool_registry.to_openai_tools();
                        match providers::chat_with_tools(&base_url, &api_key, &model, &msgs, &tools).await {
                            Ok((text, calls)) => {
                                let mut executed = Vec::new();
                                for call in &calls {
                                    if let Some(tool) = self.tool_registry.get(&call.name) {
                                        let args: serde_json::Value = serde_json::from_str(&call.arguments).unwrap_or_default();
                                        match tool.execute(args).await {
                                            Ok(result) => {
                                                executed.push(ToolCallInfo { name: call.name.clone(), status: "success".into(), duration_ms: None });
                                                msgs.push(providers::ChatMessage { role: "tool".into(), content: result });
                                            }
                                            Err(e) => {
                                                executed.push(ToolCallInfo { name: call.name.clone(), status: format!("error: {}", e), duration_ms: None });
                                            }
                                        }
                                    }
                                }
                                if !executed.is_empty() {
                                    let final_text = providers::chat(&base_url, &api_key, &model, &msgs).await?;
                                    (final_text, executed, provider.clone(), model.clone())
                                } else {
                                    (text, vec![], provider.clone(), model.clone())
                                }
                            }
                            Err(_) => {
                                let text = providers::chat(&base_url, &api_key, &model, &msgs).await?;
                                (text, vec![], provider.clone(), model.clone())
                            }
                        }
                    }
                    Err(e) => (format!("⚠️ Provider error: {}. Set your API key in %APPDATA%/nexus/.env", e), vec![], provider.clone(), model.clone()),
                }
            }
        };

        // Add assistant message (inline to avoid borrow conflict)
        {
            let msgs = self.conversations.get_mut(&sid);
            if let Some(msgs) = msgs {
                msgs.push(providers::ChatMessage { role: "assistant".into(), content: result.clone() });
            }
            if let Some(ref memory) = self.memory {
                let _ = memory.save_message(&sid, "assistant", &result);
            }
            if let Some(s) = self.sessions.get_mut(&sid) {
                s.message_count += 1;
                s.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }

        // Record to bandit selector
        if !selected_provider.is_empty() && !selected_model.is_empty() {
            let latency = start.elapsed().as_millis() as f64;
            let cost = crate::bandit::estimate_cost(&selected_provider, &selected_model, 200, 500);
            let ok = !result.starts_with("⚠️") || tool_calls.iter().any(|t| t.status == "success");

            // Trace: LLM response
            self.trace_store.record_llm_response(&sid, &selected_provider, &selected_model, &result, latency);

            if ok {
                self.bandit.record_success(&selected_provider, &selected_model, latency, cost);
            } else {
                self.bandit.record_failure(&selected_provider, &selected_model, latency, cost);
            }
        }

        // Trace: tool calls
        for tc in &tool_calls {
            self.trace_store.record_tool_result(&sid, &tc.name, &tc.status, tc.duration_ms.unwrap_or(0) as f64, tc.status == "success");
        }

        Ok(ChatResponse {
            response: result,
            session_id: sid,
            tool_calls,
        })
    }

    pub async fn process_message_stream(
        &mut self,
        _app_handle: &tauri::AppHandle,
        message: &str,
        session_id: Option<&str>,
    ) -> Result<String, anyhow::Error> {
        let result = self.process_message(message, session_id).await?;
        Ok(result.response)
    }

    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        vec![
            ProviderInfo { id: "anthropic".into(), name: "Anthropic Claude".into(), models: vec!["claude-sonnet-4".into(), "claude-3.5-haiku".into()], active: self.active_provider.as_deref() == Some("anthropic") },
            ProviderInfo { id: "openai".into(), name: "OpenAI".into(), models: vec!["gpt-4o".into(), "gpt-4o-mini".into(), "o3-mini".into()], active: self.active_provider.as_deref() == Some("openai") },
            ProviderInfo { id: "deepseek".into(), name: "DeepSeek".into(), models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()], active: self.active_provider.as_deref() == Some("deepseek") },
            ProviderInfo { id: "openrouter".into(), name: "OpenRouter".into(), models: vec!["anthropic/claude-sonnet-4".into(), "openai/gpt-4o".into(), "google/gemini-2.0-flash".into()], active: self.active_provider.as_deref() == Some("openrouter") },
            ProviderInfo { id: "google".into(), name: "Google AI".into(), models: vec!["gemini-2.0-flash".into(), "gemini-2.5-pro".into()], active: self.active_provider.as_deref() == Some("google") },
        ]
    }

    pub async fn set_active_provider(&mut self, provider_id: &str, model: &str) {
        self.active_provider = Some(provider_id.to_string());
        self.active_model = Some(model.to_string());
    }

    pub async fn update_config(&mut self, config: NexusConfig) { self.config = config; }

    pub fn list_env_vars(&self) -> Vec<EnvVar> {
        self.env_vars.iter().map(|(k, v)| {
            let masked = v.len() > 8;
            EnvVar { key: k.clone(), value: if masked { format!("{}...{}", &v[..4], &v[v.len()-4..]) } else { v.clone() }, masked }
        }).collect()
    }

    pub async fn set_env_var(&mut self, key: &str, value: &str) { self.env_vars.insert(key.to_string(), value.to_string()); }

    pub async fn toggle_gateway(&mut self, id: &str, enable: bool) -> Result<(), anyhow::Error> {
        if let Some(g) = self.gateways.get_mut(id) { g.enabled = enable; }
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> { self.sessions.values().cloned().collect() }

    pub async fn delete_session(&mut self, id: &str) -> Result<(), anyhow::Error> {
        self.sessions.remove(id); self.conversations.remove(id);
        Ok(())
    }

    fn create_session(&mut self) -> String {
        self.session_count += 1;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.sessions.insert(id.clone(), SessionInfo {
            id: id.clone(), title: format!("Session {}", self.session_count),
            message_count: 0, created_at: now.clone(), updated_at: now,
            model: self.active_model.clone().unwrap_or_default(),
        });
        self.conversations.insert(id.clone(), vec![]);
        id
    }

    fn add_message(&mut self, session_id: &str, role: &str, content: &str) {
        if let Some(msgs) = self.conversations.get_mut(session_id) {
            msgs.push(providers::ChatMessage { role: role.to_string(), content: content.to_string() });
        }
        if let Some(ref memory) = self.memory {
            let _ = memory.save_message(session_id, role, content);
        }
        if let Some(s) = self.sessions.get_mut(session_id) {
            s.message_count += 1; s.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    fn build_messages(&self, session_id: &str) -> Vec<providers::ChatMessage> {
        let mut msgs = vec![providers::ChatMessage {
            role: "system".into(),
            content: format!(
                            "You are Nexus v{}, a desktop AI agent. Be concise and helpful. \
                             You have access to tools: {}. Use tools when appropriate. \
                             MCP servers available: {}. \
                             Active skills: {}. \
                             You can use MCP tools by requesting them via the tools interface.",
                            env!("CARGO_PKG_VERSION"),
                            self.tool_registry.list().join(", "),
                            self.mcp_client.list_servers().iter()
                                .map(|s| format!("{} ({} tools)", s.name, s.tools.len()))
                                .collect::<Vec<_>>()
                                .join(", "),
                            self.skill_store.list().iter()
                                .filter(|s| s.enabled)
                                .map(|s| s.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
        }];
        if let Some(history) = self.conversations.get(session_id) {
            let start = if history.len() > 20 { history.len() - 20 } else { 0 };
            msgs.extend(history[start..].to_vec());
        }
        msgs
    }
}

// ---- Helpers ----

fn load_env_file(path: &std::path::Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Ok(contents) = std::fs::read_to_string(path) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !k.is_empty() { vars.insert(k.to_string(), v.to_string()); }
            }
        }
    }
    vars
}

fn dirs_next() -> std::path::PathBuf {
    #[cfg(target_os = "linux")] {
        std::env::var("XDG_DATA_HOME").map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".local/share"))
    }
    #[cfg(target_os = "macos")] {
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join("Library/Application Support")
    }
    #[cfg(target_os = "windows")] {
        std::env::var("APPDATA").map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into())).join("AppData/Roaming"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))] {
        std::path::PathBuf::from(".")
    }
}
