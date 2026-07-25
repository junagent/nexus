import { useState } from "react";

export default function GatewayScreen() {
  const [gateways] = useState([
    { name: "Telegram", platform: "telegram", status: "disconnected", icon: "✈" },
    { name: "Discord", platform: "discord", status: "disconnected", icon: "💬" },
    { name: "Slack", platform: "slack", status: "disconnected", icon: "🔷" },
    { name: "WhatsApp", platform: "whatsapp", status: "disconnected", icon: "📱" },
  ]);

  return (
    <div className="screen">
      <h2 className="screen-title">Gateway</h2>
      <p className="config-info" style={{ marginBottom: 16 }}>Connect Nexus to messaging platforms. Configure via <code>%APPDATA%/nexus/mcp.json</code> or the MCP panel.</p>
      {gateways.map(g => (
        <div key={g.platform} className="mcp-server-card">
          <div className="mcp-server-header">
            <strong>{g.icon} {g.name}</strong>
            <span className={`status-tag ${g.status}`}>{g.status}</span>
          </div>
          <p className="config-info" style={{ marginTop: 8, fontSize: 12 }}>Configure via MCP or gateway config file</p>
        </div>
      ))}
    </div>
  );
}