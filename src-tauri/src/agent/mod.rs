use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::providers;
use crate::commands::{
    agent::{ChatResponse, ProviderInfo, ToolCallInfo},
    config::{EnvVar, NexusConfig},
    sessions::SessionInfo,
    skills::SkillInfo,
    skills::SkillInstallResult,
    gateway::GatewayInfo,
};

/// The core Nexus agent engine.
#[derive(Debug)]
pub struct NexusEngine {
    pub running: bool,
    pub uptime: Instant,
    pub active_provider: Option<String>,
    pub active_model: Option<String>,
    pub config: NexusConfig,
    pub env_vars: HashMap<String, String>,
    pub skills: HashMap<String, SkillInfo>,
    pub gateways: HashMap<String, GatewayInfo>,
    pub sessions: HashMap<String, SessionInfo>,
    pub session_count: u32,
    pub conversations: HashMap<String, Vec<providers::ChatMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl NexusEngine {
    pub fn new() -> Self {
        let data_dir = dirs_next()
            .join("nexus")
            .to_string_lossy()
            .to_string();

        // Load env from ~/.nexus/.env
        let env_path = dirs_next().join("nexus").join(".env");
        let env_vars = load_env_file(&env_path);

        // Apply to process environment
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
            skills: HashMap::new(),
            gateways: HashMap::new(),
            sessions: HashMap::new(),
            session_count: 0,
            conversations: HashMap::new(),
        }
    }

    /// Process a chat message against real LLM.
    pub async fn process_message(
        &mut self,
        message: &str,
        session_id: Option<&str>,
    ) -> Result<ChatResponse, anyhow::Error> {
        let sid = session_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.create_session());

        self.add_message(&sid, "user", message);

        let result = if let (Some(ref provider), Some(ref model)) =
            (&self.active_provider, &self.active_model)
        {
            match providers::get_provider_config(provider) {
                Ok((base_url, api_key)) => {
                    let msgs = self.build_messages(&sid);
                    providers::chat(&base_url, &api_key, model, &msgs).await?
                }
                Err(e) => format!("⚠️ Provider error: {}. Set your API key in %APPDATA%/nexus/.env", e),
            }
        } else {
            format!(
                "🤖 **Nexus v{}**\n\nConfigure an LLM provider in the Engine Config panel (left sidebar).\n\nThen I can call real APIs — OpenAI, Anthropic, DeepSeek, OpenRouter, Google AI — all supported.",
                env!("CARGO_PKG_VERSION")
            )
        };

        self.add_message(&sid, "assistant", &result);

        Ok(ChatResponse {
            response: result,
            session_id: sid,
            tool_calls: vec![],
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
            ProviderInfo {
                id: "anthropic".into(),
                name: "Anthropic Claude".into(),
                models: vec!["claude-sonnet-4".into(), "claude-3.5-haiku".into()],
                active: self.active_provider.as_deref() == Some("anthropic"),
            },
            ProviderInfo {
                id: "openai".into(),
                name: "OpenAI".into(),
                models: vec!["gpt-4o".into(), "gpt-4o-mini".into(), "o3-mini".into()],
                active: self.active_provider.as_deref() == Some("openai"),
            },
            ProviderInfo {
                id: "deepseek".into(),
                name: "DeepSeek".into(),
                models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
                active: self.active_provider.as_deref() == Some("deepseek"),
            },
            ProviderInfo {
                id: "openrouter".into(),
                name: "OpenRouter".into(),
                models: vec![
                    "anthropic/claude-sonnet-4".into(),
                    "openai/gpt-4o".into(),
                    "google/gemini-2.0-flash".into(),
                ],
                active: self.active_provider.as_deref() == Some("openrouter"),
            },
            ProviderInfo {
                id: "google".into(),
                name: "Google AI".into(),
                models: vec!["gemini-2.0-flash".into(), "gemini-2.5-pro".into()],
                active: self.active_provider.as_deref() == Some("google"),
            },
        ]
    }

    pub async fn set_active_provider(&mut self, provider_id: &str, model: &str) {
        self.active_provider = Some(provider_id.to_string());
        self.active_model = Some(model.to_string());
    }

    pub async fn update_config(&mut self, config: NexusConfig) {
        self.config = config;
    }

    pub fn list_env_vars(&self) -> Vec<EnvVar> {
        self.env_vars
            .iter()
            .map(|(k, v)| {
                let masked = v.len() > 8;
                EnvVar {
                    key: k.clone(),
                    value: if masked {
                        format!("{}...{}", &v[..4], &v[v.len() - 4..])
                    } else {
                        v.clone()
                    },
                    masked,
                }
            })
            .collect()
    }

    pub async fn set_env_var(&mut self, key: &str, value: &str) {
        self.env_vars.insert(key.to_string(), value.to_string());
    }

    pub async fn install_skill(&mut self, source: &str) -> Result<SkillInstallResult, anyhow::Error> {
        let name = source.split('/').last().unwrap_or(source).trim_end_matches(".git").to_string();
        self.skills.insert(
            name.clone(),
            SkillInfo {
                name: name.clone(),
                version: "0.1.0".into(),
                description: format!("Skill from {}", source),
                enabled: true,
            },
        );
        Ok(SkillInstallResult {
            success: true,
            name,
            message: format!("Installed from {}", source),
        })
    }

    pub async fn remove_skill(&mut self, name: &str) -> Result<(), anyhow::Error> {
        self.skills.remove(name);
        Ok(())
    }

    pub async fn toggle_gateway(&mut self, id: &str, enable: bool) -> Result<(), anyhow::Error> {
        if let Some(gateway) = self.gateways.get_mut(id) {
            gateway.enabled = enable;
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.values().cloned().collect()
    }

    pub async fn delete_session(&mut self, id: &str) -> Result<(), anyhow::Error> {
        self.sessions.remove(id);
        self.conversations.remove(id);
        Ok(())
    }

    fn create_session(&mut self) -> String {
        self.session_count += 1;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.sessions.insert(
            id.clone(),
            SessionInfo {
                id: id.clone(),
                title: format!("Session {}", self.session_count),
                message_count: 0,
                created_at: now.clone(),
                updated_at: now,
                model: self.active_model.clone().unwrap_or_default(),
            },
        );
        self.conversations.insert(id.clone(), vec![]);
        id
    }

    fn add_message(&mut self, session_id: &str, role: &str, content: &str) {
        if let Some(messages) = self.conversations.get_mut(session_id) {
            messages.push(providers::ChatMessage {
                role: role.to_string(),
                content: content.to_string(),
            });
        }
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.message_count += 1;
            session.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    fn build_messages(&self, session_id: &str) -> Vec<providers::ChatMessage> {
        let mut msgs = vec![providers::ChatMessage {
            role: "system".into(),
            content: format!(
                "You are Nexus v{}, an intelligent desktop AI agent. \
                 Be concise, helpful, and direct. Use markdown for formatting. \
                 You run as a native Rust + Tauri application.",
                env!("CARGO_PKG_VERSION")
            ),
        }];

        if let Some(history) = self.conversations.get(session_id) {
            // Take last 20 messages to stay within context limits
            let start = if history.len() > 20 { history.len() - 20 } else { 0 };
            msgs.extend(history[start..].to_vec());
        }

        msgs
    }
}

/// Load .env file in KEY=VALUE format.
fn load_env_file(path: &std::path::Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Ok(contents) = std::fs::read_to_string(path) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !k.is_empty() {
                    vars.insert(k.to_string(), v.to_string());
                }
            }
        }
    }
    vars
}

/// Get the user's data directory.
fn dirs_next() -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                std::path::PathBuf::from(home).join(".local/share")
            })
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        std::path::PathBuf::from(home).join("Library/Application Support")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
                std::path::PathBuf::from(home).join("AppData/Roaming")
            })
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        std::path::PathBuf::from(".")
    }
}
