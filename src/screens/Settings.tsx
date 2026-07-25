import { agentServerStatus } from "../api";
import { useState, useEffect } from "react";

export default function SettingsScreen() {
  const [server, setServer] = useState<{ running: boolean; port: number; url: string } | null>(null);
  useEffect(() => { agentServerStatus().then(setServer); }, []);

  return (
    <div className="screen">
      <h2 className="screen-title">Settings</h2>

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