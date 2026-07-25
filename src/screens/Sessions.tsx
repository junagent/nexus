import { useState, useEffect } from "react";
import type { SessionInfo } from "../types";
import { listSessions, deleteSession } from "../api";

export default function SessionsScreen() {
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  useEffect(() => { listSessions().then(setSessions); }, []);
  const del = async (id: string) => { await deleteSession(id); setSessions(s => s.filter(x => x.id !== id)); };

  return (
    <div className="screen">
      <h2 className="screen-title">Sessions</h2>
      {sessions.length === 0 && <p className="empty-state">No sessions yet.</p>}
      <div className="session-list">
        {sessions.map(s => (
          <div key={s.id} className="session-card">
            <div className="session-info">
              <strong>{s.title}</strong>
              <span className="session-meta">{s.message_count} msgs · {s.model} · {new Date(s.updated_at).toLocaleDateString()}</span>
            </div>
            <button className="btn-icon" onClick={() => del(s.id)} title="Delete">✕</button>
          </div>
        ))}
      </div>
    </div>
  );
}