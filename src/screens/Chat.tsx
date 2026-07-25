import { useState, useRef, useEffect, useCallback } from "react";
import type { Message } from "../types";
import { chatSend } from "../api";

export default function ChatScreen() {
  const [messages, setMessages] = useState<Message[]>([
    { role: "assistant", content: "**Nexus v0.1.0**\n\nThe Core of Your AGI.\n\nRust + Tauri desktop agent. Select a provider in Engine Config to start." },
  ]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [sessionId, setSessionId] = useState<string | null>(null);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => { endRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages]);

  const send = useCallback(async () => {
    if (!input.trim() || sending) return;
    const text = input.trim();
    setInput("");
    setMessages(m => [...m, { role: "user", content: text }]);
    setSending(true);
    try {
      const response = await chatSend(text, sessionId, model);
      setMessages(m => [...m, { role: "assistant", content: response }]);
    } catch (e) {
      setMessages(m => [...m, { role: "assistant", content: `❌ Error: ${e}` }]);
    }
    setSending(false);
  }, [input, sessionId, model, sending]);

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); send(); }
  };

  return (
    <div className="chat-screen">
      <div className="chat-toolbar">
        <input
          className="model-input"
          value={model}
          onChange={e => setModel(e.target.value)}
          placeholder="model name"
        />
        <button className="new-chat-btn" onClick={() => { setMessages([]); setSessionId(null); }}>
          ✦ New Chat
        </button>
      </div>

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
        {sending && <div className="message message-assistant"><div className="message-avatar">◆</div><div className="message-content"><span className="typing">Thinking...</span></div></div>}
        <div ref={endRef} />
      </div>

      <div className="input-bar">
        <textarea
          className="input-field"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKey}
          placeholder="Message Nexus..."
          rows={1}
          disabled={sending}
        />
        <button className="send-btn" onClick={send} disabled={sending || !input.trim()}>
          ▶
        </button>
      </div>
    </div>
  );
}