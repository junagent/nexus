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
        "**Nexus v0.1.0**\n\nThe Core of Your AGI.\n\nI'm your desktop agent — built on Rust + Tauri, powered by the hermes-agent-rs engine. Configure your LLM provider below and start the conversation.",
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

        <div className="sidebar-status">
          <div className={`status-dot ${systemInfo?.agent_active ? "active" : "inactive"}`} />
          <span className="status-text">
            {systemInfo?.agent_active
              ? `Connected · ${systemInfo.active_model.split("/").pop()}`
              : "No provider configured"}
          </span>
        </div>

        <div className="sidebar-nav">
          <button
            className={`nav-btn ${!showConfig ? "active" : ""}`}
            onClick={() => setShowConfig(false)}
          >
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.5">
              <polygon points="8,1 15,8 8,15 1,8" />
            </svg>
            Chat
          </button>
          <button
            className={`nav-btn ${showConfig ? "active" : ""}`}
            onClick={() => setShowConfig(true)}
          >
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="8" cy="8" r="2" />
              <circle cx="8" cy="8" r="6" />
              <line x1="8" y1="0" x2="8" y2="3" />
              <line x1="8" y1="13" x2="8" y2="16" />
              <line x1="0" y1="8" x2="3" y2="8" />
              <line x1="13" y1="8" x2="16" y2="8" />
            </svg>
            Engine Config
          </button>
        </div>

        <div className="sidebar-footer">
          <span className="version-text">v{systemInfo?.version || "0.1.0"} · rust+tauri</span>
          <div className="scanline" />
        </div>
      </div>

      {/* Main content */}
      <div className="main">
        {showConfig ? (
          <div className="config-panel">
            <h2 className="config-title">Engine Configuration</h2>

            <div className="config-section">
              <label className="config-label">LLM Provider</label>
              <select
                className="config-select"
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
              >
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
                <div className="stat-row">
                  <span className="stat-key">Platform</span>
                  <span className="stat-val">{systemInfo.platform}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-key">CPU Cores</span>
                  <span className="stat-val">{systemInfo.cpu_cores}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-key">Active Provider</span>
                  <span className="stat-val">{systemInfo.active_provider}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-key">Active Model</span>
                  <span className="stat-val">{systemInfo.active_model}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-key">Agent Status</span>
                  <span className="stat-val">{systemInfo.agent_active ? "Online" : "Offline"}</span>
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="chat-area">
            <div className="messages">
              {messages.map((msg, i) => (
                <div key={i} className={`message message-${msg.role}`}>
                  <div className="message-avatar">
                    {msg.role === "user" ? (
                      <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
                        <circle cx="8" cy="5" r="3" />
                        <path d="M2,14 C2,10 6,9 8,9 C10,9 14,10 14,14" />
                      </svg>
                    ) : (
                      <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
                        <polygon points="8,2 14,8 8,14 2,8" />
                      </svg>
                    )}
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
              <button
                className="send-btn"
                onClick={handleSend}
                disabled={sending || !input.trim()}
              >
                <svg viewBox="0 0 20 20" width="18" height="18" fill="currentColor">
                  <polygon points="2,2 18,10 2,18 5,10" />
                </svg>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
