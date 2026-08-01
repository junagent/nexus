use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: DeltaMessage,
}

#[derive(Debug, Deserialize)]
struct DeltaMessage {
    content: String,
    #[serde(default)]
    role: String,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

/// Call an OpenAI-compatible API (non-streaming).
pub async fn chat(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: false,
        max_tokens: Some(4096),
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, text));
    }

    let data: ChatResponse = resp.json().await?;
    let content = data
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string());

    Ok(content)
}

/// Call an OpenAI-compatible API (streaming). Returns chunks via callback.
pub async fn chat_stream(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    on_chunk: impl Fn(&str),
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: true,
        max_tokens: Some(4096),
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, text));
    }

    let mut full_response = String::new();
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            if let Some(json_str) = line.strip_prefix("data: ") {
                if let Ok(data) = serde_json::from_str::<StreamChunk>(json_str) {
                    if let Some(content) = data.choices.first().and_then(|c| c.delta.content.as_deref()) {
                        full_response.push_str(content);
                        on_chunk(content);
                    }
                }
            }
        }
    }

    Ok(full_response)
}

/// Auto-detect a usable provider from whichever API key is set in the
/// environment. Returns (provider_id, default_model). Priority order favors
/// the most capable / most commonly-used providers first.
pub fn auto_detect_provider() -> Option<(String, String)> {
    let candidates = [
        ("GITHUB_TOKEN", "github", "gpt-4o-mini"),
        ("GROQ_API_KEY", "groq", "llama-3.3-70b-versatile"),
        ("OPENROUTER_API_KEY", "openrouter", "anthropic/claude-sonnet-4"),
        ("ANTHROPIC_API_KEY", "anthropic", "claude-sonnet-4"),
        ("OPENAI_API_KEY", "openai", "gpt-4o"),
        ("DEEPSEEK_API_KEY", "deepseek", "deepseek-chat"),
        ("GOOGLE_API_KEY", "google", "gemini-2.0-flash"),
    ];
    for (env_key, provider, default_model) in candidates {
        if std::env::var(env_key).map(|v| !v.trim().is_empty()).unwrap_or(false) {
            return Some((provider.to_string(), default_model.to_string()));
        }
    }
    None
}

/// Get the appropriate base URL and API key for a provider.
pub fn get_provider_config(provider: &str) -> Result<(String, String)> {
    match provider {
        "openai" => {
            let key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("OPENAI_API_KEY not set")); }
            Ok(("https://api.openai.com".into(), key))
        }
        "anthropic" => {
            let key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("ANTHROPIC_API_KEY not set")); }
            Ok(("https://api.anthropic.com".into(), key))
        }
        "deepseek" => {
            let key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("DEEPSEEK_API_KEY not set")); }
            Ok(("https://api.deepseek.com".into(), key))
        }
        "openrouter" => {
            let key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("OPENROUTER_API_KEY not set")); }
            Ok(("https://openrouter.ai/api".into(), key))
        }
        "google" => {
            let key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("GOOGLE_API_KEY not set")); }
            Ok(("https://generativelanguage.googleapis.com".into(), key))
        }
        "github" => {
            // GitHub Models — free inference endpoint. Uses GITHUB_TOKEN.
            let key = std::env::var("GITHUB_TOKEN").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("GITHUB_TOKEN not set")); }
            Ok(("https://models.github.ai/inference".into(), key))
        }
        "groq" => {
            let key = std::env::var("GROQ_API_KEY").unwrap_or_default();
            if key.is_empty() { return Err(anyhow!("GROQ_API_KEY not set")); }
            Ok(("https://api.groq.com/openai".into(), key))
        }
        _ => Err(anyhow!("Unknown provider: {}", provider)),
    }
}

/// Chat with tool support. Returns (text_response, tool_calls).
pub async fn chat_with_tools(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
) -> Result<(String, Vec<FunctionCall>), anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "tools": tools,
        "tool_choice": "auto",
        "max_tokens": 4096,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, text));
    }

    let data: serde_json::Value = resp.json().await?;
    let choice = data["choices"][0].clone();
    let msg = choice["message"].clone();

    let text = msg["content"].as_str().unwrap_or("").to_string();

    let mut calls = Vec::new();
    if let Some(tool_calls) = msg["tool_calls"].as_array() {
        for tc in tool_calls {
            if let (Some(name), Some(args)) = (
                tc["function"]["name"].as_str(),
                tc["function"]["arguments"].as_str(),
            ) {
                calls.push(FunctionCall {
                    name: name.to_string(),
                    arguments: args.to_string(),
                });
            }
        }
    }

    Ok((text, calls))
}

/// Chat with tool support + streaming. Returns (text_response, tool_calls).
pub async fn chat_with_tools_stream(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    on_chunk: impl Fn(&str),
) -> Result<(String, Vec<FunctionCall>), anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "tools": tools,
        "tool_choice": "auto",
        "max_tokens": 4096,
        "stream": true,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, text));
    }

    let mut full_response = String::new();
    let mut calls = Vec::<FunctionCall>::new();
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line == "data: [DONE]" { continue; }
            if let Some(json_str) = line.strip_prefix("data: ") {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(delta) = data["choices"][0]["delta"].as_object() {
                        // Text content
                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            full_response.push_str(content);
                            on_chunk(content);
                        }
                        // Tool calls
                        if let Some(tool_calls) = delta.get("tool_calls").and_then(|c| c.as_array()) {
                            for tc in tool_calls {
                                if let (Some(name), Some(args)) = (
                                    tc["function"]["name"].as_str(),
                                    tc["function"]["arguments"].as_str(),
                                ) {
                                    calls.push(FunctionCall {
                                        name: name.to_string(),
                                        arguments: args.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((full_response, calls))
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}
