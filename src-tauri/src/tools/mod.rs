//! Tool Registry — typed tools with auto schema generation.
//!
//! Inspired by hermes-rs and hermes-agent-rs.
//! Each tool implements the `Tool` trait and gets registered.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A tool that the agent can invoke.
#[async_trait::async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// Unique tool name (used in LLM function calls).
    fn name(&self) -> &str;
    /// Human-readable description for the LLM.
    fn description(&self) -> &str;
    /// JSON schema of the tool's parameters.
    fn schema(&self) -> Value;
    /// Execute the tool with the given arguments.
    async fn execute(&self, args: Value) -> Result<String, String>;
}

/// Central tool registry.
#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Build OpenAI-compatible tools array for API calls.
    pub fn to_openai_tools(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": {
                        "type": "object",
                        "properties": tool.schema()["properties"].clone(),
                        "required": tool.schema().get("required").cloned().unwrap_or_default(),
                    }
                }
            })
        }).collect()
    }

    /// List all tool names.
    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

// ---- File System Tools ----

#[derive(Debug)]
pub struct ReadFileTool;

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read the contents of a file at the given path." }
    fn schema(&self) -> Value {
        serde_json::json!({
            "properties": {
                "path": { "type": "string", "description": "Absolute path to the file" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String, String> {
        let path = args["path"].as_str().ok_or("Missing 'path' argument")?;
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
    }
}

#[derive(Debug)]
pub struct WriteFileTool;

#[async_trait::async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str {
        "Write content to a file. Creates parent directories if needed. Overwrites existing files."
    }
    fn schema(&self) -> Value {
        serde_json::json!({
            "properties": {
                "path": { "type": "string", "description": "Absolute path to write to" },
                "content": { "type": "string", "description": "Content to write" }
            },
            "required": ["path", "content"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String, String> {
        let path = args["path"].as_str().ok_or("Missing 'path'")?;
        let content = args["content"].as_str().ok_or("Missing 'content'")?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path, e))?;
        Ok(format!("Written {} bytes to {}", content.len(), path))
    }
}

#[derive(Debug)]
pub struct ListDirTool;

#[async_trait::async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &str { "list_dir" }
    fn description(&self) -> &str { "List files and directories at the given path." }
    fn schema(&self) -> Value {
        serde_json::json!({
            "properties": {
                "path": { "type": "string", "description": "Directory path to list" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String, String> {
        let path = args["path"].as_str().ok_or("Missing 'path'")?;
        let entries: Vec<String> = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let meta = e.metadata().ok();
                let prefix = match meta {
                    Some(m) if m.is_dir() => "📁 ",
                    _ => "📄 ",
                };
                format!("{}{}", prefix, name)
            })
            .collect();
        Ok(if entries.is_empty() { "(empty)".into() } else { entries.join("\n") })
    }
}

// ---- Shell Execution Tool ----

#[derive(Debug)]
pub struct ShellTool;

#[async_trait::async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str { "shell" }
    fn description(&self) -> &str {
        "Execute a shell command and return its output. Use with caution."
    }
    fn schema(&self) -> Value {
        serde_json::json!({
            "properties": {
                "command": { "type": "string", "description": "The shell command to execute" }
            },
            "required": ["command"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String, String> {
        let cmd = args["command"].as_str().ok_or("Missing 'command'")?;
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/c", cmd]).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).output()
        };
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                if o.status.success() {
                    Ok(stdout.to_string())
                } else {
                    Ok(format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr))
                }
            }
            Err(e) => Err(format!("Command failed: {}", e)),
        }
    }
}

// ---- Web Tool ----

#[derive(Debug)]
pub struct WebFetchTool;

#[async_trait::async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }
    fn description(&self) -> &str { "Fetch content from a URL and return as text." }
    fn schema(&self) -> Value {
        serde_json::json!({
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" }
            },
            "required": ["url"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String, String> {
        let url = args["url"].as_str().ok_or("Missing 'url'")?;
        let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let text = resp.text().await.map_err(|e| e.to_string())?;
        // Truncate to 5000 chars
        if text.len() > 5000 {
            Ok(format!("{}...(truncated, {} total chars)", &text[..5000], text.len()))
        } else {
            Ok(text)
        }
    }
}
