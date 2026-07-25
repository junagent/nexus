import { useState, useEffect } from "react";
import type { BanditArm } from "../types";
import { banditStats } from "../api";

export default function BanditScreen() {
  const [arms, setArms] = useState<BanditArm[]>([]);
  useEffect(() => { banditStats().then(setArms); }, []);

  return (
    <div className="screen">
      <h2 className="screen-title">Bandit Stats</h2>
      {arms.length === 0 && <p className="empty-state">No data yet. Send messages to build stats.</p>}
      <div className="bandit-table">
        <div className="bandit-header">
          <span>Provider</span><span>Model</span><span>Trials</span><span>Success Rate</span><span>Avg Latency</span><span>Avg Cost</span><span>UCB1</span>
        </div>
        {arms.map(a => (
          <div key={`${a.provider}/${a.model}`} className="bandit-row">
            <span>{a.provider}</span>
            <span className="mono">{a.model}</span>
            <span className="mono">{a.trials}</span>
            <span className="mono">{(a.success_rate * 100).toFixed(0)}%</span>
            <span className="mono">{a.avg_latency_ms.toFixed(0)}ms</span>
            <span className="mono">${a.avg_cost.toFixed(5)}</span>
            <span className="mono">{a.ucb1_score.toFixed(3)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}