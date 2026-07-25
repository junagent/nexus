import { useState, useEffect } from "react";
import type { TraceEvent } from "../types";
import { traceQuery, traceClear } from "../api";

export default function TraceScreen() {
  const [events, setEvents] = useState<TraceEvent[]>([]);
  const [filter, setFilter] = useState("");
  const [detail, setDetail] = useState<TraceEvent | null>(null);

  const load = async () => setEvents(await traceQuery(filter || undefined, 100));
  useEffect(() => { load(); }, [filter]);

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Trace Events</h2>
        <button className="btn-sm" onClick={() => { traceClear(); setEvents([]); }}>Clear</button>
      </div>
      <input className="config-input" value={filter} onChange={e => setFilter(e.target.value)} placeholder="Filter by type, tag, or text..." style={{ marginBottom: 12 }} />

      <div className="trace-list">
        {events.map(e => (
          <div key={e.id} className={`trace-row trace-${e.event_type}`} onClick={() => setDetail(detail?.id === e.id ? null : e)}>
            <span className="trace-type">{e.event_type}</span>
            <span className="trace-summary">{e.summary}</span>
            <span className="trace-duration">{e.duration_ms > 0 ? `${e.duration_ms.toFixed(0)}ms` : ""}</span>
            {detail?.id === e.id && <pre className="trace-detail">{e.detail}</pre>}
          </div>
        ))}
        {events.length === 0 && <p className="empty-state">No trace events. Send a message to generate events.</p>}
      </div>
    </div>
  );
}