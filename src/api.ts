import { invoke } from "@tauri-apps/api/core";
import type {
  SystemInfo, SessionInfo, SkillInfo, McpServerInfo,
  TraceEvent, BanditArm, ApprovalRequest,
} from "./types";

// Chat
export async function chatSend(message: string, sessionId: string | null, model: string): Promise<string> {
  return invoke("chat_stream", { request: { message, sessionId, model } });
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke("get_system_info");
}

// Sessions
export async function listSessions(): Promise<SessionInfo[]> {
  return invoke("list_sessions");
}

export async function deleteSession(id: string): Promise<void> {
  return invoke("delete_session", { id });
}

// Providers
export async function setProvider(providerId: string, model: string): Promise<void> {
  return invoke("set_provider", { providerId, model });
}

// Skills
export async function listSkills(): Promise<SkillInfo[]> {
  return invoke("list_skills");
}

export async function installSkill(source: string): Promise<string> {
  const result = await invoke<{ success: boolean; message: string }>("install_skill", { source });
  return result.message;
}

export async function removeSkill(name: string): Promise<void> {
  return invoke("remove_skill", { name });
}

export async function toggleSkill(name: string, enabled: boolean): Promise<void> {
  return invoke("toggle_skill", { name, enabled });
}

// MCP
export async function listMcpServers(): Promise<McpServerInfo[]> {
  return invoke("list_mcp_servers");
}

export async function addMcpServer(config: { name: string; command: string; args: string[]; env: Record<string, string> }): Promise<void> {
  return invoke("add_mcp_server", { config });
}

export async function removeMcpServer(name: string): Promise<void> {
  return invoke("remove_mcp_server", { name });
}

export async function connectMcpServer(name: string): Promise<void> {
  return invoke("connect_mcp_server", { name });
}

// Trace
export async function traceQuery(filter?: string, limit?: number): Promise<TraceEvent[]> {
  return invoke("trace_query", { filter, limit });
}

export async function traceClear(): Promise<void> {
  return invoke("trace_clear");
}

// Bandit
export async function banditStats(): Promise<BanditArm[]> {
  return invoke("bandit_stats");
}

// Approvals
export async function approvalPending(): Promise<ApprovalRequest[]> {
  return invoke("approval_pending");
}

export async function approvalRespond(id: string, approved: boolean): Promise<void> {
  return invoke("approval_respond", { id, approved });
}

// Agent Server
export async function agentServerStatus(): Promise<{ running: boolean; port: number; url: string }> {
  return invoke("agent_server_status");
}