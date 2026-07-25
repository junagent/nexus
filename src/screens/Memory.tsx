import { useState, useEffect } from "react";
import { listSessions } from "../api";

export default function MemoryScreen() {
  const [sessionCount, setSessionCount] = useState(0);
  const [totalMsgs, setTotalMsgs] = useState(0);
  useEffect(() => {
    listSessions().then(sessions => {
      setSessionCount(sessions.length);
      setTotalMsgs(sessions.reduce((sum, s) => sum + s.message_count, 0));
    });
  }, []);

  return (
    <div className="screen">
      <h2 className="screen-title">Memory</h2>
      <div className="config-stats" style={{ marginBottom: 20 }}>
        <div className="stat-row"><span className="stat-key">Sessions</span><span className="stat-val">{sessionCount}</span></div>
        <div className="stat-row"><span className="stat-key">Total Messages</span><span className="stat-val">{totalMsgs}</span></div>
        <div className="stat-row"><span className="stat-key">Memory Backend</span><span className="stat-val">SQLite</span></div>
      </div>
      <p className="config-info">Memory is stored in <code>%APPDATA%/nexus/memory.db</code>. Future: vector store + RAG.</p>
    </div>
  );
}