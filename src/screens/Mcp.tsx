import { useState, useEffect } from "react";
import type { McpServerInfo } from "../types";
import { listMcpServers, addMcpServer, removeMcpServer, connectMcpServer } from "../api";

export default function McpScreen() {
  const [servers, setServers] = useState<McpServerInfo[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [name, setName] = useState("");
  const [cmd, setCmd] = useState("");
  const [args, setArgs] = useState("");

  const load = async () => setServers(await listMcpServers());
  useEffect(() => { load(); }, []);

  const add = async () => {
    await addMcpServer({ name, command: cmd, args: args.split(" ").filter(Boolean), env: {} });
    setShowAdd(false);
    setName(""); setCmd(""); setArgs("");
    await load();
  };

  const remove = async (n: string) => { await removeMcpServer(n); await load(); };
  const connect = async (n: string) => { await connectMcpServer(n); await load(); };

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">MCP Servers</h2>
        <button className="btn-primary" onClick={() => setShowAdd(!showAdd)}>+ Add Server</button>
      </div>

      {showAdd && (
        <div className="card">
          <input className="config-input" value={name} onChange={e => setName(e.target.value)} placeholder="Server name" />
          <input className="config-input" value={cmd} onChange={e => setCmd(e.target.value)} placeholder="Command (e.g. npx)" />
          <input className="config-input" value={args} onChange={e => setArgs(e.target.value)} placeholder="Args (e.g. -y @modelcontextprotocol/server-filesystem)" />
          <button className="btn-primary" onClick={add}>Add</button>
        </div>
      )}

      {servers.map(s => (
        <div key={s.name} className="mcp-server-card">
          <div className="mcp-server-header">
            <strong>{s.name}</strong>
            <span className={`status-tag ${s.status}`}>{s.status}</span>
          </div>
          <div className="mcp-server-tools">
            {s.tools.map(t => <span key={t.name} className="tool-tag">{t.name}</span>)}
            {s.tools.length === 0 && <span className="text-muted">No tools</span>}
          </div>
          <div className="mcp-server-actions">
            {s.status !== "connected" && <button className="btn-sm" onClick={() => connect(s.name)}>Connect</button>}
            <button className="btn-sm danger" onClick={() => remove(s.name)}>Remove</button>
          </div>
        </div>
      ))}
      {servers.length === 0 && <p className="empty-state">No MCP servers configured.</p>}
    </div>
  );
}