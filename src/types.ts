export interface SystemInfo {
  version: string;
  platform: string;
  cpu_cores: number;
  agent_active: boolean;
  active_provider: string;
  active_model: string;
}

export interface Message {
  role: "user" | "assistant" | "tool" | "system";
  content: string;
  id?: string;
  timestamp?: string;
  tool_calls?: ToolCallInfo[];
}

export interface ToolCallInfo {
  name: string;
  status: string;
  duration_ms: number | null;
}

export interface SessionInfo {
  id: string;
  title: string;
  message_count: number;
  created_at: string;
  updated_at: string;
  model: string;
}

export interface SkillInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  tags: string[];
}

export interface McpServerInfo {
  name: string;
  status: string;
  tools: { name: string; description: string }[];
}

export interface TraceEvent {
  id: number;
  timestamp: string;
  event_type: string;
  session_id: string;
  provider: string | null;
  model: string | null;
  summary: string;
  detail: string;
  duration_ms: number;
  tags: string[];
}

export interface BanditArm {
  provider: string;
  model: string;
  trials: number;
  success_rate: number;
  avg_latency_ms: number;
  avg_cost: number;
  ucb1_score: number;
}

export interface ApprovalRequest {
  id: string;
  timestamp: string;
  tool_name: string;
  arguments: string;
  risk_level: string;
  reason: string;
  status: string;
}

export type Screen =
  | "chat"
  | "sessions"
  | "providers"
  | "skills"
  | "mcp"
  | "trace"
  | "bandit"
  | "settings"
  | "approvals";