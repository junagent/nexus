import { useState } from "react";

export default function SchedulesScreen() {
  const [crons] = useState<{ name: string; schedule: string; enabled: boolean }[]>([
    { name: "Morning Briefing", schedule: "0 9 * * *", enabled: true },
    { name: "Evening Review", schedule: "0 22 * * *", enabled: false },
  ]);

  return (
    <div className="screen">
      <h2 className="screen-title">Schedules</h2>
      <p className="config-info" style={{ marginBottom: 16 }}>Scheduled tasks run automatically. Configure via cron syntax or the Nexus API.</p>
      {crons.map(c => (
        <div key={c.name} className="skill-card">
          <div className="skill-info">
            <strong>{c.name}</strong>
            <span className="skill-version">{c.schedule}</span>
          </div>
          <div className="skill-actions">
            <label className="toggle">
              <input type="checkbox" checked={c.enabled} readOnly />
              <span className="toggle-slider" />
            </label>
          </div>
        </div>
      ))}
      <p className="empty-state">More scheduling features coming soon.</p>
    </div>
  );
}