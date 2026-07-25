import { useState, useRef, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Message {
  role: "user" | "assistant";
  content: string;
}

interface SystemInfo {
  version: string;
  platform: string;
  cpu_cores: number;
  agent_active: boolean;
  active_provider: string;
  active_model: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([
    {
      role: "assistant",
      content:
        "🔗 **Nexus v0.1.0**\n\n*The Core of Your AGI.*\n\nI'm your desktop agent — built on hermes-agent-rs with Rust + Tauri. Configure your LLM provider below and start chatting.",
    },
  ]);
  const [input, setInput] = useState("");
  const [sessionId] = useState<string | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [provider, setProvider] = useState("openrouter");
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [sending, setSending] = useState(false);
  const [showConfig, setShowConfig] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<SystemInfo>("get_system_info")
      .then(setSystemInfo)
      .catch(console.error);
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = useCallback(async () => {
    if (!input.trim() || sending) return;

    const userMsg = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: userMsg }]);
    setSending(true);

    try {
      const result = await invoke<string>("chat_stream", {
        request: {
          message: userMsg,
          sessionId,
          model,
        },
      });

      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: result },
      ]);
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: `❌ Error: ${err}`,
        },
      ]);
    }
    setSending(false);
  }, [input, sessionId, model, sending]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSetProvider = async () => {
    try {
      await invoke("set_provider", { providerId: provider, model });
      setSystemInfo((prev) =>
        prev
          ? { ...prev, active_provider: provider, active_model: model, agent_active: true }
          : prev
      );
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="app">
      {/* Animated background */}
      <div className="bg-grid" />
      <div className="bg-glow" />

      {/* Sidebar */}
      <div className="sidebar">
        <div className="sidebar-logo">
          <div className="logo-triangle">
            <svg viewBox="0 0 64 64" className="logo-svg">
              <polygon points="32,4 60,56 4,56" fill="none" stroke="#00d4ff" strokeWidth="2" opacity="0.8"/>
              <polygon points="32,18 46,46 18,46" fill="none" stroke="#ff00e4" strokeWidth="1.5" opacity="0.6"/>
              <polygon points="32,28 40,42 24,42" fill="none" stroke="#00ff88" strokeWidth="1" opacity="0.4"/>
              <circle cx="32" cy="32" r="4" fill="#00d4ff" opacity="0.9"/>
              <circle cx="32" cy="32" r="1.5" fill="#ffffff"/>
            </svg>
          </div>
          <div className="logo-text">
            <span className="logo-title">NEXUS</span>
            <span className="logo-subtitle">The Core of Your AGI</span>
          </div>
        </div>

        <div className="sidebar-status">
          <div className={`status-dot ${systemInfo?.agent_active ? "active" : "inactive"}`} />
          <span className="status-text">
            {systemInfo?.agent_active
              ? `Connected · ${systemInfo.active_model.split("/").pop()}`
              : "No provider configured"}
          </span>
        </div>

        <div className="sidebar-nav">
          <button className={`nav-btn ${!showConfig ? "active" : ""}`} onClick={() => setShowConfig(false)}>
            <svg viewBox="0 0 20 20" width="16" height="16"><polygon points="10,2 18,10 10,18 2,10" fill="currentColor"/></svg>
            Chat
          </button>
          <button className={`nav-btn ${showConfig ? "active" : ""}`} onClick={() => setShowConfig(true)}>
            <svg viewBox="0 0 20 20" width="16" height="16"><circle cx="10" cy="10" r="2" fill="none" stroke="currentColor" strokeWidth="2"/><circle cx="10" cy="10" r="7" fill="none" stroke="currentColor" strokeWidth="1.5"/><line x1="10" y1="1" x2="10" y2="4" stroke="currentColor" strokeWidth="1.5"/><line x1="10" y1="16" x2="10" y2="19" stroke="currentColor" strokeWidth="1.5"/><line x1="1" y1="10" x2="4" y2="10" stroke="currentColor" strokeWidth="1.5"/><line x1="16" y1="10" x2="19" y2="10" stroke="currentColor" strokeWidth="1.5"/></svg>
            Engine Config
          </button>
        </div>

        {/* Scanline effect */}
        <div className="sidebar-footer">
          <span className="version-text">v{systemInfo?.version || "0.1.0"} · rust+tauri</span>
          <div className="scanline" />
        </div>
      </div>

      {/* Main content */}
      <div className="main">
        {showConfig ? (
          <div className="config-panel">
            <h2 className="config-title">⚙️ Engine Configuration</h2>
            
            <div className="config-section">
              <label className="config-label">LLM Provider</label>
              <select className="config-select" value={provider} onChange={(e) => setProvider(e.target.value)}>
                <option value="anthropic">Anthropic Claude</option>
                <option value="openai">OpenAI</option>
                <option value="deepseek">DeepSeek</option>
                <option value="openrouter">OpenRouter</option>
                <option value="google">Google AI</option>
              </select>
            </div>

            <div className="config-section">
              <label className="config-label">Model</label>
              <input
                className="config-input"
                value={model}
                onChange={(e) => setModel(e.target.value)}
                placeholder="e.g., anthropic/claude-sonnet-4"
              />
            </div>

            <div className="config-section">
              <label className="config-label">Provider Config</label>
              <div className="config-info">
                Set your API keys via the Nexus env manager or ~/.nexus/.env
              </div>
            </div>

            <button className="config-apply" onClick={handleSetProvider}>
              Apply Provider
            </button>

            {systemInfo && (
              <div className="config-stats">
                <div className="stat-row"><span className="stat-key">Platform</span><span className="stat-val">{systemInfo.platform}</span></div>
                <div className="stat-row"><span className="stat-key">CPU Cores</span><span className="stat-val">{systemInfo.cpu_cores}</span></div>
                <div className="stat-row"><span className="stat-key">Active Provider</span><span className="stat-val">{systemInfo.active_provider}</span></div>
                <div className="stat-row"><span className="stat-key">Active Model</span><span className="stat-val">{systemInfo.active_model}</span></div>
                <div className="stat-row"><span className="stat-key">Agent Status</span><span className="stat-val">{systemInfo.agent_active ? "✅ Online" : "⛔ Offline"}</span></div>
              </div>
            )}
          </div>
        ) : (
          <div className="chat-area">
            <div className="messages">
              {messages.map((msg, i) => (
                <div key={i} className={`message message-${msg.role}`}>
                  <div className="message-avatar">
                    {msg.role === "user" ? "👤" : "◆"}
                  </div>
                  <div className="message-content">
                    <div className="message-text">{msg.content}</div>
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>

            <div className="input-bar">
              <textarea
                className="input-field"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Message Nexus..."
                rows={1}
                disabled={sending}
              />
              <button className="send-btn" onClick={handleSend} disabled={sending || !input.trim()}>
                {sending ? "..." : "⏎"}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
