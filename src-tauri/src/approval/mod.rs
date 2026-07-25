use serde::{Deserialize, Serialize};

/// Risk level of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,     // Auto-approved
    Warning,  // Logged, no confirmation needed
    Dangerous, // Requires human approval
    Critical, // Always blocked unless explicitly whitelisted
}

/// An approval request pending user action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub timestamp: String,
    pub tool_name: String,
    pub arguments: String,
    pub risk_level: RiskLevel,
    pub reason: String,
    pub status: String, // "pending", "approved", "rejected"
    pub responded_at: Option<String>,
}

/// The approval handler prevents dangerous operations without user consent.
pub struct ApprovalHandler {
    pending_requests: Vec<ApprovalRequest>,
    /// Auto-approved tool patterns (substring match on tool name)
    safe_tools: Vec<String>,
    /// Dangerous tool patterns
    dangerous_tools: Vec<String>,
    /// Critical tool patterns (blocked unless explicitly allowed)
    critical_tools: Vec<String>,
}

impl std::fmt::Debug for ApprovalHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalHandler")
            .field("pending", &self.pending_requests.len())
            .field("safe_tools", &self.safe_tools)
            .field("dangerous_tools", &self.dangerous_tools)
            .finish()
    }
}

impl ApprovalHandler {
    pub fn new() -> Self {
        Self {
            pending_requests: Vec::new(),
            safe_tools: vec![
                "read".into(), "list".into(), "search".into(), "get".into(),
                "fetch".into(), "query".into(), "lookup".into(),
            ],
            dangerous_tools: vec![
                "write".into(), "delete".into(), "remove".into(), "edit".into(),
                "create".into(), "mkdir".into(), "shell".into(), "exec".into(),
                "command".into(), "run".into(),
            ],
            critical_tools: vec![
                "format".into(), "wipe".into(), "shutdown".into(), "reboot".into(),
                "sudo".into(), "rm -rf".into(),
            ],
        }
    }

    /// Determine the risk level for a tool call.
    pub fn assess(&self, tool_name: &str, _arguments: &str) -> RiskLevel {
        let lower = tool_name.to_lowercase();

        // Check critical first
        for pattern in &self.critical_tools {
            if lower.contains(pattern) {
                return RiskLevel::Critical;
            }
        }

        // Check dangerous
        for pattern in &self.dangerous_tools {
            if lower.contains(pattern) {
                return RiskLevel::Dangerous;
            }
        }

        // Check safe
        for pattern in &self.safe_tools {
            if lower.contains(pattern) {
                return RiskLevel::Safe;
            }
        }

        // Default: warning for unknown tools
        RiskLevel::Warning
    }

    /// Create a pending approval request.
    pub fn request_approval(&mut self, tool_name: &str, arguments: &str, risk_level: RiskLevel) -> ApprovalRequest {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let reason = match risk_level {
            RiskLevel::Safe => "Auto-approved".into(),
            RiskLevel::Warning => format!("Tool '{}' may modify system state", tool_name),
            RiskLevel::Dangerous => format!("Tool '{}' is potentially destructive. Approve?", tool_name),
            RiskLevel::Critical => format!("Tool '{}' is critical. Manual approval required.", tool_name),
        };

        let request = ApprovalRequest {
            id: id.clone(),
            timestamp: now.clone(),
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            risk_level,
            reason,
            status: "pending".into(),
            responded_at: None,
        };

        self.pending_requests.push(request.clone());
        request
    }

    /// Approve or reject a pending request.
    pub fn respond(&mut self, id: &str, approved: bool) -> bool {
        if let Some(req) = self.pending_requests.iter_mut().find(|r| r.id == id) {
            req.status = if approved { "approved".into() } else { "rejected".into() };
            req.responded_at = Some(chrono::Utc::now().to_rfc3339());
            true
        } else {
            false
        }
    }

    /// Check if a tool call is allowed (auto-approved for safe/warning, pending for dangerous/critical).
    pub fn check(&mut self, tool_name: &str, arguments: &str) -> (bool, Option<ApprovalRequest>) {
        let risk = self.assess(tool_name, arguments);

        match risk {
            RiskLevel::Safe | RiskLevel::Warning => (true, None),
            RiskLevel::Dangerous | RiskLevel::Critical => {
                let request = self.request_approval(tool_name, arguments, risk);
                (false, Some(request))
            }
        }
    }

    /// Get all pending requests.
    pub fn pending(&self) -> &[ApprovalRequest] {
        &self.pending_requests
    }

    /// Clear completed requests.
    pub fn clear_completed(&mut self) {
        self.pending_requests.retain(|r| r.status == "pending");
    }
}