import { useState, useRef, useEffect, useCallback } from "react";
import type { Message } from "../types";
import { chatSend } from "../api";

// Simulated streaming for demo — real streaming will use Tauri events
function useStreamingResponse(
  setMessages: (fn: (prev: Message[]) => Message[]) => void,
  sending: boolean,
  setSending: (v: boolean) => void,
  input: string,
  setInput: (v: string) => void,
  sessionId: string | null,
  model: string,
) {
  const send = useCallback(async () => {
    if (!input.trim() || sending) return;
    const text = input.trim();
    setInput("");
    setMessages(m => [...m, { role: "user", content: text }]);

    setSending(true);
    // Add a placeholder assistant message
    const msgId = Date.now().toString();
    setMessages(m => [...m, { role: "assistant", content: "", id: msgId }]);

    try {
      const response = await chatSend(text, sessionId, model);
      // Replace the placeholder with the full response
      setMessages(m => m.map(msg =>
        msg.id === msgId ? { ...msg, content: response, id: undefined } : msg
      ));
    } catch (e) {
      setMessages(m => m.map(msg =>
        msg.id === msgId
          ? { role: "assistant", content: `❌ Error: ${e}`, id: undefined }
          : msg
      ));
    }
    setSending(false);
  }, [input, sending, sessionId, model]);

  return send;
}

export default function ChatScreen() {
  const [messages, setMessages] = useState<Message[]>([
    { role: "assistant", content: "**Nexus v0.1.0**\n\nThe Core of Your AGI." },
  ]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [showSidebar, setShowSidebar] = useState(true);
  const [sessions, setSessions] = useState<{ id: string; title: string }[]>([]);
  const endRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => { endRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages]);

  // Load sessions
  useEffect(() => {
    import("../api").then(m => m.listSessions().then(s => setSessions(s.map(x => ({ id: x.id, title: x.title })))));
  }, []);

  const send = useStreamingResponse(setMessages, sending, setSending, input, setInput, sessionId, model);

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); send(); }
    // Slash command detection
    if (e.key === " " && input.startsWith("/")) {
      const parts = input.split(" ");
      const cmd = parts[0].toLowerCase();
      if (cmd === "/model" && parts[1]) {
        setModel(parts[1]);
        setInput("");
      }
      if (cmd === "/new") {
        setMessages([]);
        setSessionId(null);
        setInput("");
      }
      if (cmd === "/help") {
        setMessages(m => [...m, { role: "assistant", content: "**Slash Commands:**\n- `/model <name>` — switch model\n- `/new` — new conversation\n- `/help` — this help" }]);
        setInput("");
      }
    }
  };

  return (
    <div className="chat-screen">
      {/* Top bar */}
      <div className="chat-topbar">
        <button className="topbar-btn" onClick={() => setShowSidebar(!showSidebar)}>☰</button>
        <div className="topbar-tabs">
          {sessions.slice(0, 5).map(s => (
            <span key={s.id} className="session-tab">{s.title}</span>
          ))}
          <button className="topbar-btn" onClick={() => { setMessages([]); setSessionId(null); }}>+ New</button>
        </div>
        <div className="topbar-right">
          <span className="model-badge">{model}</span>
        </div>
      </div>

      <div className="chat-body">
        {/* Sessions sidebar */}
        {showSidebar && (
          <aside className="chat-sidebar">
            <h3 className="sidebar-title">Sessions</h3>
            {sessions.map(s => (
              <div key={s.id} className="sidebar-session" onClick={() => setSessionId(s.id)}>
                <span className="session-title">{s.title}</span>
              </div>
            ))}
            {sessions.length === 0 && <p className="text-muted" style={{ padding: 12, fontSize: 12 }}>No sessions yet</p>}
          </aside>
        )}

        {/* Messages */}
        <div className="messages">
          {messages.map((msg, i) => (
            <div key={i} className={`message message-${msg.role}`}>
              <div className="message-avatar">
                {msg.role === "user" ? "👤" : "◆"}
              </div>
              <div className="message-bubble">
                <div className="message-text">{msg.content || (sending && i === messages.length - 1 ? <span className="cursor-blink">▊</span> : "")}</div>
              </div>
            </div>
          ))}
          {sending && messages[messages.length - 1]?.content === "" && (
            <div className="message message-assistant">
              <div className="message-avatar">◆</div>
              <div className="message-bubble">
                <div className="thinking-dots">
                  <span className="dot" /><span className="dot" /><span className="dot" />
                </div>
              </div>
            </div>
          )}
          <div ref={endRef} />
        </div>
      </div>

      {/* Input */}
      <div className="input-bar">
        <textarea
          ref={inputRef}
          className="input-field"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKey}
          placeholder="Message Nexus... (/model, /new, /help)"
          rows={1}
          disabled={sending}
        />
        <button className="send-btn" onClick={send} disabled={sending || !input.trim()}>
          <svg viewBox="0 0 20 20" width="18" height="18" fill="currentColor"><polygon points="2,2 18,10 2,18 5,10" /></svg>
        </button>
      </div>
    </div>
  );
}