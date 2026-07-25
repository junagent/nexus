import { useState, useEffect } from "react";
import type { ApprovalRequest } from "../types";
import { approvalPending, approvalRespond } from "../api";

export default function ApprovalsScreen() {
  const [requests, setRequests] = useState<ApprovalRequest[]>([]);
  const load = async () => setRequests(await approvalPending());
  useEffect(() => { load(); const i = setInterval(load, 3000); return () => clearInterval(i); }, []);

  const respond = async (id: string, approved: boolean) => {
    await approvalRespond(id, approved);
    await load();
  };

  return (
    <div className="screen">
      <h2 className="screen-title">Approval Requests</h2>
      {requests.length === 0 && <p className="empty-state">No pending approval requests.</p>}
      {requests.map(r => (
        <div key={r.id} className="approval-card">
          <div className="approval-header">
            <span className={`risk-tag ${r.risk_level.toLowerCase()}`}>{r.risk_level}</span>
            <strong>{r.tool_name}</strong>
          </div>
          <p className="approval-reason">{r.reason}</p>
          <pre className="approval-args">{r.arguments}</pre>
          <div className="approval-actions">
            <button className="btn-sm danger" onClick={() => respond(r.id, false)}>Reject</button>
            <button className="btn-sm" onClick={() => respond(r.id, true)}>Approve</button>
          </div>
        </div>
      ))}
    </div>
  );
}