import { useState, useEffect } from "react";
import type { SkillInfo } from "../types";
import { listSkills, toggleSkill, removeSkill, installSkill } from "../api";

export default function SkillsScreen() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [source, setSource] = useState("");

  const load = async () => setSkills(await listSkills());
  useEffect(() => { load(); }, []);

  const toggle = async (name: string, enabled: boolean) => {
    await toggleSkill(name, enabled);
    await load();
  };

  const remove = async (name: string) => {
    await removeSkill(name);
    await load();
  };

  const install = async () => {
    if (!source.trim()) return;
    await installSkill(source.trim());
    setSource("");
    await load();
  };

  return (
    <div className="screen">
      <h2 className="screen-title">Skills</h2>

      <div className="install-row">
        <input
          className="config-input"
          value={source}
          onChange={e => setSource(e.target.value)}
          placeholder="GitHub URL or local path..."
          style={{ flex: 1 }}
        />
        <button className="btn-primary" onClick={install}>Install</button>
      </div>

      {skills.length === 0 && <p className="empty-state">No skills installed. Skills live in <code>%APPDATA%/nexus/skills/</code></p>}

      <div className="skill-list">
        {skills.map(s => (
          <div key={s.name} className="skill-card">
            <div className="skill-info">
              <strong>{s.name}</strong> <span className="skill-version">v{s.version}</span>
              <p className="skill-desc">{s.description}</p>
              <span className="skill-author">{s.author}</span>
              {s.tags.length > 0 && <div className="skill-tags">{s.tags.map(t => <span key={t} className="tag">{t}</span>)}</div>}
            </div>
            <div className="skill-actions">
              <label className="toggle">
                <input type="checkbox" checked={s.enabled} onChange={() => toggle(s.name, !s.enabled)} />
                <span className="toggle-slider" />
              </label>
              <button className="btn-icon" onClick={() => remove(s.name)} title="Remove">✕</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}