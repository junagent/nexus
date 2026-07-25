import { useState, useRef, useEffect, useCallback } from "react";
import type { Message } from "../types";
import { chatSend, listSessions } from "../api";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export default function ChatScreen() {
  const [messages, setMessages] = useState<Message[]>([
    { role: "assistant", content: "**Nexus v0.1.0**\n\nThe Core of Your AGI." },
  ]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [streamingText, setStreamingText] = useState("");
  const [toolEvents, setToolEvents] = useState<{ tool: string; status: string }[]>([]);
  const [sessions, setSessions] = useState<{ id: string; title: string }[]>([]);
  const [showSidebar, setShowSidebar] = useState(true);
  const [estTokens, setEstTokens] = useState(0);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => { endRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages, streamingText, toolEvents]);

  // Load sessions
  useEffect(() => { listSessions().then(s => setSessions(s.map(x => ({ id: x.id, title: x.title })))); }, []);

  // Listen for Tauri streaming events
  useEffect(() => {
    const unlisten: (() => void)[] = [];
    const setup = async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten.push(await listen<{ chunk: string; session_id: string }>("nexus://stream/chunk", (e) => {
        setStreamingText(prev => prev + e.payload.chunk);
      }));
      unlisten.push(await listen<{ session_id: string; tool_name: string; status: string; arguments: string; result?: string }>("nexus://stream/tool_call", (e) => {
        setToolEvents(prev => [...prev, { tool: e.payload.tool_name, status: "running..." }]);
      }));
      unlisten.push(await listen<{ session_id: string; tool_name: string; status: string; arguments: string; result?: string }>("nexus://stream/tool_result", (e) => {
        setToolEvents(prev => {
          const idx = prev.map(t => t.tool).lastIndexOf(e.payload.tool_name);
          if (idx >= 0) {
            const copy = [...prev];
            copy[idx] = { tool: e.payload.tool_name, status: e.payload.status === "success" ? "✅ done" : "❌ failed" };
            return copy;
          }
          return [...prev, { tool: e.payload.tool_name, status: e.payload.status === "success" ? "✅ done" : "❌ failed" }];
        });
      }));
      unlisten.push(await listen("nexus://stream/done", () => {
        setMessages(m => {
          if (streamingText) {
            return [...m.slice(0, -1), { role: "assistant" as const, content: streamingText }];
          }
          return m;
        });
        setStreamingText("");
        setToolEvents([]);
        setSending(false);
      }));
    };
    setup();
    return () => unlisten.forEach(fn => fn());
  }, [streamingText]);

  const send = useCallback(async () => {
    if (!input.trim() || sending) return;
    const text = input.trim();
    setInput("");
    setMessages(m => [...m, { role: "user", content: text }, { role: "assistant", content: "" }]);
    setSending(true);
    try {
      await chatSend(text, sessionId, model);
    } catch (e) {
      setMessages(m => {
        const copy = [...m];
        copy[copy.length - 1] = { role: "assistant", content: `❌ Error: ${e}` };
        return copy;
      });
      setSending(false);
    }
  }, [input, sessionId, model, sending]);

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); send(); }
    if (e.key === " " && input.startsWith("/")) {
      const cmd = input.split(" ")[0].toLowerCase();
      if (cmd === "/model" && input.split(" ")[1]) { setModel(input.split(" ")[1]); setInput(""); }
      if (cmd === "/new") { setMessages([]); setSessionId(null); setInput(""); }
      if (cmd === "/help") {
        setMessages(m => [...m, { role: "assistant", content: "**Slash Commands:**\n- `/model <name>` — switch model\n- `/new` — new conversation\n- `/help` — this help" }]);
        setInput("");
      }
    }
  };

  return (
    <div className="chat-screen">
      <div className="chat-topbar">
        <button className="topbar-btn" onClick={() => setShowSidebar(!showSidebar)}>☰</button>
        <div className="topbar-tabs">
          {sessions.slice(0, 5).map(s => (
            <span key={s.id} className="session-tab" onClick={() => setSessionId(s.id)}>{s.title}</span>
          ))}
          <button className="topbar-btn" onClick={() => { setMessages([]); setSessionId(null); }}>+ New</button>
        </div>
        <div className="topbar-right">
          <span className="model-badge">{model}</span>
        </div>
      </div>

      <div className="chat-body">
        {showSidebar && (
          <aside className="chat-sidebar">
            <h3 className="sidebar-title">Sessions</h3>
            {sessions.map(s => (
              <div key={s.id} className="sidebar-session" onClick={() => setSessionId(s.id)}>
                <span className="session-title">{s.title}</span>
              </div>
            ))}
            {sessions.length === 0 && <p className="text-muted" style={{ padding: 12, fontSize: 12 }}>No sessions</p>}
          </aside>
        )}

        <div className="messages">
          {messages.map((msg, i) => (
            <div key={i} className={`message message-${msg.role}`}>
              <div className="message-avatar">{msg.role === "user" ? "👤" : "◆"}</div>
              <div className="message-bubble">
                {msg.role === "assistant" ? (
                  <div className="message-text">
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>{msg.content}</ReactMarkdown>
                  </div>
                ) : (
                  <div className="message-text">{msg.content}</div>
                )}
              </div>
            </div>
          ))}

          {/* Streaming text */}
          {streamingText && (
            <div className="message message-assistant">
              <div className="message-avatar">◆</div>
              <div className="message-bubble">
                <div className="message-text">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{streamingText}</ReactMarkdown>
                  <span className="cursor-blink">▊</span>
                </div>
              </div>
            </div>
          )}

          {/* Tool events */}
          {toolEvents.length > 0 && (
            <div className="message message-assistant">
              <div className="message-avatar">🔧</div>
              <div className="message-bubble tool-events">
                {toolEvents.map((t, i) => (
                  <div key={i} className="tool-event-row">
                    <span className="tool-event-name">{t.tool}</span>
                    <span className="tool-event-status">{t.status}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {sending && !streamingText && (
            <div className="message message-assistant">
              <div className="message-avatar">◆</div>
              <div className="message-bubble"><div className="thinking-dots"><span className="dot" /><span className="dot" /><span className="dot" /></div></div>
            </div>
          )}
          <div ref={endRef} />
        </div>
      </div>

      <div className="input-bar">
        <div className="input-toolbar">
          <button className="input-toolbar-btn" title="Attach file">📎</button>
          <div className="context-gauge">
            <div className="context-gauge-fill" style={{ width: `${Math.min(estTokens / 100, 100)}%` }} />
            <span className="context-gauge-text">{estTokens > 0 ? `${estTokens.toFixed(0)}K` : ""}</span>
          </div>
        </div>
        <textarea
          className="input-field"
          value={input}
          onChange={e => { setInput(e.target.value); setEstTokens(e.target.value.length * 0.75 / 1000); }}
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