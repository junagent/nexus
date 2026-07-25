import { useState, useEffect } from "react";
import { agentServerStatus } from "../api";

export default function SettingsScreen() {
  const [server, setServer] = useState<{ running: boolean; port: number; url: string } | null>(null);
  const [systemPrompt, setSystemPrompt] = useState("You are Nexus v0.1.0, a desktop AI agent. Be concise and helpful.");
  useEffect(() => { agentServerStatus().then(setServer); }, []);

  return (
    <div className="screen">
      <h2 className="screen-title">Settings</h2>

      <div className="config-section">
        <h3>Agent Personality</h3>
        <label className="config-label">System Prompt</label>
        <textarea
          className="config-input"
          value={systemPrompt}
          onChange={e => setSystemPrompt(e.target.value)}
          rows={4}
          style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 12 }}
        />
        <p className="config-info" style={{ marginTop: 8 }}>This system prompt is injected into every conversation. Customize Nexus's personality.</p>
      </div>

      <div className="config-section">
        <h3>Agent Server</h3>
        {server && (
          <div className="config-stats">
            <div className="stat-row"><span className="stat-key">Status</span><span className="stat-val">{server.running ? "Running" : "Offline"}</span></div>
            <div className="stat-row"><span className="stat-key">Port</span><span className="stat-val">{server.port}</span></div>
            <div className="stat-row"><span className="stat-key">URL</span><span className="stat-val">{server.url}</span></div>
          </div>
        )}
      </div>

      <div className="config-section">
        <h3>API Keys</h3>
        <p className="config-info">Set API keys in <code>%APPDATA%/nexus/.env</code></p>
      </div>

      <div className="config-section">
        <h3>Data Directory</h3>
        <p className="config-info">Config: <code>%APPDATA%/nexus/</code></p>
      </div>
    </div>
  );
}