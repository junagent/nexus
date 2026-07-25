use std::collections::HashMap;
use std::process::Stdio;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub status: String,
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self { servers: HashMap::new() }
    }
}

impl McpConfig {
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        Ok(std::fs::write(path, serde_json::to_string_pretty(self)?)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

/// MCP connection (not Debug because Child is not Debug)
struct McpConnection {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    tools: Vec<McpTool>,
}

/// Manual Debug for McpConnection (skip tokio internals)
impl std::fmt::Debug for McpConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpConnection")
            .field("tools", &self.tools)
            .finish()
    }
}

pub struct McpClient {
    config: McpConfig,
    connections: HashMap<String, McpConnection>,
    next_id: u64,
}

/// Manual Debug for McpClient
impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient")
            .field("config", &self.config)
            .field("connections", &self.connections)
            .finish()
    }
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
            connections: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn load_config(&mut self, path: &std::path::Path) {
        self.config = McpConfig::load(path);
    }

    pub fn save_config(&self, path: &std::path::Path) -> anyhow::Result<()> {
        self.config.save(path)
    }

    pub fn list_servers(&self) -> Vec<McpServerInfo> {
        self.config.servers.keys().map(|name| {
            let connected = self.connections.contains_key(name);
            let tools = self.connections.get(name)
                .map(|c| c.tools.clone())
                .unwrap_or_default();
            McpServerInfo {
                name: name.clone(),
                status: if connected { "connected".into() } else { "disconnected".into() },
                tools,
            }
        }).collect()
    }

    pub fn add_server(&mut self, config: McpServerConfig) {
        self.config.servers.insert(config.name.clone(), config);
    }

    pub fn remove_server(&mut self, name: &str) {
        self.config.servers.remove(name);
        if let Some(mut conn) = self.connections.remove(name) {
            let _ = conn.child.kill();
        }
    }

    /// Connect to an MCP server, initialize it, and list tools.
    pub async fn connect_server(&mut self, name: &str) -> anyhow::Result<Vec<McpTool>> {
        let cfg = self.config.servers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

        let mut cmd = tokio::process::Command::new(&cfg.command);
        cmd.args(&cfg.args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        for (k, v) in &cfg.env { cmd.env(k, v); }

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;

        let mut conn = McpConnection {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            tools: vec![],
        };

        // Initialize — pass next_id as a separate &mut u64, not as &mut self
        let _ = send_request(&mut self.next_id, &mut conn, "initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "clientInfo": { "name": "nexus", "version": "0.1.0" }
        })).await?;

        let _ = send_notification(&mut conn, "initialized", serde_json::json!({})).await?;

        // List tools
        let tools_result = send_request(&mut self.next_id, &mut conn, "tools/list", serde_json::json!({})).await?;
        let tools: Vec<McpTool> = tools_result.get("tools")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        conn.tools = tools.clone();
        self.connections.insert(name.to_string(), conn);
        Ok(tools)
    }

    /// Call a tool on a connected MCP server.
    pub async fn call_tool(
        &mut self, server: &str, tool: &str, arguments: serde_json::Value,
    ) -> anyhow::Result<String> {
        let conn = self.connections.get_mut(server)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not connected", server))?;

        let result = send_request(&mut self.next_id, conn, "tools/call", serde_json::json!({
            "name": tool, "arguments": arguments
        })).await?;

        // Extract text content
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let texts: Vec<String> = content.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()).map(String::from))
                .collect();
            if !texts.is_empty() { return Ok(texts.join("\n")); }
        }
        Ok(result.to_string())
    }

    /// Get all tools from all connected servers.
    pub fn list_all_tools(&self) -> Vec<(String, McpTool)> {
        let mut all = vec![];
        for (name, conn) in &self.connections {
            for tool in &conn.tools {
                all.push((name.clone(), tool.clone()));
            }
        }
        all
    }
}

// --- Free functions (no &mut self, avoids double-borrow) ---

async fn send_request(
    next_id: &mut u64, conn: &mut McpConnection, method: &str, params: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let id = *next_id;
    *next_id += 1;

    let req = McpRequest { jsonrpc: "2.0".into(), id, method: method.into(), params };
    let line = serde_json::to_string(&req)?;
    conn.stdin.write_all(format!("{}\n", line).as_bytes()).await?;
    conn.stdin.flush().await?;

    loop {
        let mut buf = String::new();
        match conn.stdout.read_line(&mut buf).await {
            Ok(0) => return Err(anyhow::anyhow!("Connection closed")),
            Ok(_) => {
                let buf = buf.trim();
                if buf.is_empty() { continue; }
                if let Ok(resp) = serde_json::from_str::<McpResponse>(buf) {
                    if resp.id == Some(id) {
                        if let Some(e) = resp.error {
                            return Err(anyhow::anyhow!("MCP error: {:?}", e));
                        }
                        return Ok(resp.result.unwrap_or(serde_json::Value::Null));
                    }
                }
            }
            Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
        }
    }
}

async fn send_notification(
    conn: &mut McpConnection, method: &str, params: serde_json::Value,
) -> anyhow::Result<()> {
    let notif = serde_json::json!({ "jsonrpc": "2.0", "method": method, "params": params });
    let line = serde_json::to_string(&notif)?;
    conn.stdin.write_all(format!("{}\n", line).as_bytes()).await?;
    conn.stdin.flush().await?;
    Ok(())
}