import type { Screen } from "../types";

interface Props {
  currentScreen: Screen;
  onNavigate: (s: Screen) => void;
  approvalCount: number;
  serverRunning: boolean;
}

export default function Sidebar({ currentScreen, onNavigate, approvalCount, serverRunning }: Props) {
  const nav = [
    { id: "chat" as Screen, label: "Chat", icon: "💬" },
    { id: "sessions" as Screen, label: "Sessions", icon: "📋" },
    { id: "providers" as Screen, label: "Providers", icon: "⚡" },
    { id: "skills" as Screen, label: "Skills", icon: "🧠" },
    { id: "mcp" as Screen, label: "MCP", icon: "🔌" },
    { id: "trace" as Screen, label: "Trace", icon: "📊" },
    { id: "bandit" as Screen, label: "Bandit", icon: "🎰" },
    { id: "approvals" as Screen, label: `Approvals${approvalCount > 0 ? ` (${approvalCount})` : ""}`, icon: "🔒" },
    { id: "settings" as Screen, label: "Settings", icon: "⚙️" },
  ];

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="brand-wordmark">
          <span className="letter-n">N</span>
          <span className="letter-e">E</span>
          <span className="letter-x-core">
            <span className="x-tri-left" />
            <span className="x-tri-right" />
            <span className="x-dot" />
            X
          </span>
          <span className="letter-u">U</span>
          <span className="letter-s">S</span>
        </div>
        <span className="brand-slogan">THE CORE OF YOUR AGI</span>
      </div>

      <nav className="sidebar-nav">
        {nav.map(({ id, label, icon }) => (
          <button
            key={id}
            className={`nav-btn ${currentScreen === id ? "active" : ""}`}
            onClick={() => onNavigate(id)}
          >
            <span className="nav-icon">{icon}</span>
            <span className="nav-label">{label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="server-status">
          <span className={`status-dot ${serverRunning ? "active" : "inactive"}`} />
          <span className="status-text">{serverRunning ? "API Server: 18789" : "Server offline"}</span>
        </div>
        <div className="scanline" />
      </div>
    </aside>
  );
}