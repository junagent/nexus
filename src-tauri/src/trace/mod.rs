use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// A single trace event recording agent activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: u64,
    pub timestamp: String,
    pub event_type: String, // "llm_request", "llm_response", "tool_call", "tool_result", "decision", "error"
    pub session_id: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub summary: String,
    pub detail: String,
    pub duration_ms: f64,
    pub tags: Vec<String>,
}

/// The trace store maintains a ring buffer of recent events.
pub struct TraceStore {
    events: VecDeque<TraceEvent>,
    max_events: usize,
    next_id: u64,
}

impl std::fmt::Debug for TraceStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraceStore")
            .field("events", &self.events.len())
            .field("max_events", &self.max_events)
            .field("next_id", &self.next_id)
            .finish()
    }
}

impl TraceStore {
    /// Create a new trace store with a maximum event count.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            next_id: 1,
        }
    }

    /// Record an event.
    fn record(&mut self, event_type: &str, session_id: &str, provider: Option<&str>, model: Option<&str>, summary: &str, detail: &str, duration_ms: f64, tags: Vec<String>) {
        let id = self.next_id;
        self.next_id += 1;

        let event = TraceEvent {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            session_id: session_id.to_string(),
            provider: provider.map(|s| s.to_string()),
            model: model.map(|s| s.to_string()),
            summary: summary.to_string(),
            detail: detail.to_string(),
            duration_ms,
            tags,
        };

        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Record an LLM request.
    pub fn record_llm_request(&mut self, session_id: &str, provider: &str, model: &str, messages: &str) {
        self.record("llm_request", session_id, Some(provider), Some(model), 
            &format!("LLM request to {}/{}", provider, model),
            &format!("Messages:\n{}", truncate(messages, 500)),
            0.0, vec!["llm".into()]);
    }

    /// Record an LLM response.
    pub fn record_llm_response(&mut self, session_id: &str, provider: &str, model: &str, response: &str, duration_ms: f64) {
        self.record("llm_response", session_id, Some(provider), Some(model),
            &format!("LLM response from {}/{} ({}ms)", provider, model, duration_ms as u64),
            &format!("Response:\n{}", truncate(response, 500)),
            duration_ms, vec!["llm".into()]);
    }

    /// Record a tool call.
    pub fn record_tool_call(&mut self, session_id: &str, tool_name: &str, arguments: &str) {
        self.record("tool_call", session_id, None, None,
            &format!("Tool call: {}", tool_name),
            &format!("Tool: {}\nArguments: {}", tool_name, truncate(arguments, 500)),
            0.0, vec!["tool".into(), tool_name.to_string()]);
    }

    /// Record a tool result.
    pub fn record_tool_result(&mut self, session_id: &str, tool_name: &str, result: &str, duration_ms: f64, success: bool) {
        let status = if success { "success" } else { "error" };
        self.record("tool_result", session_id, None, None,
            &format!("Tool result: {} ({})", tool_name, status),
            &format!("Tool: {}\nResult:\n{}", tool_name, truncate(result, 500)),
            duration_ms, vec!["tool".into(), tool_name.to_string(), status.into()]);
    }

    /// Record a decision.
    pub fn record_decision(&mut self, session_id: &str, summary: &str, detail: &str) {
        self.record("decision", session_id, None, None, summary, detail, 0.0, vec!["decision".into()]);
    }

    /// Record an error.
    pub fn record_error(&mut self, session_id: &str, error: &str) {
        self.record("error", session_id, None, None, "Error", error, 0.0, vec!["error".into()]);
    }

    /// Get all events, optionally filtered.
    pub fn query(&self, filter: Option<&str>, limit: usize) -> Vec<TraceEvent> {
        let mut events: Vec<TraceEvent> = self.events.iter().rev().cloned().collect();
        
        if let Some(f) = filter {
            let f = f.to_lowercase();
            events.retain(|e| {
                e.event_type.to_lowercase().contains(&f) ||
                e.summary.to_lowercase().contains(&f) ||
                e.tags.iter().any(|t| t.to_lowercase().contains(&f))
            });
        }

        events.truncate(limit);
        events
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get event count.
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...\n[truncated {} chars]", &s[..max], s.len() - max)
    }
}