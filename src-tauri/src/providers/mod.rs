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
        _ => Err(anyhow!("Unknown provider: {}", provider)),
    }
}
