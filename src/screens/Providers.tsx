import { useState, useEffect } from "react";
import { setProvider, getEnv, setEnv } from "../api";

const PROVIDERS: Record<string, { name: string; envKey: string; models: string[] }> = {
  openrouter: { name: "OpenRouter (recommended)", envKey: "OPENROUTER_API_KEY", models: ["anthropic/claude-sonnet-4", "openai/gpt-4o", "google/gemini-2.0-flash", "deepseek/deepseek-chat"] },
  anthropic: { name: "Anthropic Claude", envKey: "ANTHROPIC_API_KEY", models: ["claude-sonnet-4", "claude-3.5-haiku"] },
  openai: { name: "OpenAI", envKey: "OPENAI_API_KEY", models: ["gpt-4o", "gpt-4o-mini", "o3-mini"] },
  deepseek: { name: "DeepSeek", envKey: "DEEPSEEK_API_KEY", models: ["deepseek-chat", "deepseek-reasoner"] },
  google: { name: "Google AI", envKey: "GOOGLE_API_KEY", models: ["gemini-2.0-flash", "gemini-2.5-pro"] },
};

export default function ProvidersScreen() {
  const [provider, setProv] = useState("openrouter");
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [apiKey, setApiKey] = useState("");
  const [status, setStatus] = useState("");
  const [configured, setConfigured] = useState<Record<string, string>>({});

  const refresh = async () => {
    try {
      const envs = await getEnv();
      const map: Record<string, string> = {};
      for (const e of envs) map[e.key] = e.value;
      setConfigured(map);
    } catch { /* ignore */ }
  };

  useEffect(() => { refresh(); }, []);

  const onProviderChange = (p: string) => {
    setProv(p);
    setModel(PROVIDERS[p].models[0]);
    setApiKey("");
  };

  const saveKey = async () => {
    if (!apiKey.trim()) { setStatus("❌ Enter an API key first"); return; }
    try {
      await setEnv(PROVIDERS[provider].envKey, apiKey.trim());
      await setProvider(provider, model);
      setStatus(`✅ Saved & activated: ${provider} / ${model}`);
      setApiKey("");
      await refresh();
    } catch (e) { setStatus(`❌ ${e}`); }
  };

  const activateOnly = async () => {
    try {
      await setProvider(provider, model);
      setStatus(`✅ Active: ${provider} / ${model}`);
    } catch (e) { setStatus(`❌ ${e}`); }
  };

  const cfg = PROVIDERS[provider];
  const isConfigured = !!configured[cfg.envKey];

  return (
    <div className="screen">
      <h2 className="screen-title">Providers</h2>

      <div className="config-section">
        <label className="config-label">LLM Provider</label>
        <select className="config-select" value={provider} onChange={e => onProviderChange(e.target.value)}>
          {Object.entries(PROVIDERS).map(([id, p]) => (
            <option key={id} value={id}>{p.name}{configured[p.envKey] ? " ✓" : ""}</option>
          ))}
        </select>
      </div>

      <div className="config-section">
        <label className="config-label">Model</label>
        <select className="config-select" value={model} onChange={e => setModel(e.target.value)}>
          {cfg.models.map(m => <option key={m} value={m}>{m}</option>)}
        </select>
        <input className="config-input" style={{ marginTop: 8 }} value={model} onChange={e => setModel(e.target.value)} placeholder="or type a custom model id" />
      </div>

      <div className="config-section">
        <label className="config-label">
          API Key <span className="config-env-key">({cfg.envKey})</span>
          {isConfigured && <span className="status-tag connected" style={{ marginLeft: 8 }}>configured: {configured[cfg.envKey]}</span>}
        </label>
        <input
          className="config-input"
          type="password"
          value={apiKey}
          onChange={e => setApiKey(e.target.value)}
          placeholder={isConfigured ? "•••••••• (enter a new key to replace)" : "Paste your API key here"}
        />
      </div>

      <div style={{ display: "flex", gap: 8 }}>
        <button className="btn-primary" onClick={saveKey}>Save Key & Activate</button>
        <button className="btn-secondary" onClick={activateOnly} disabled={!isConfigured}>Activate (key already set)</button>
      </div>
      {status && <p className="status-msg">{status}</p>}

      <div className="config-info" style={{ marginTop: 16 }}>
        Keys are stored locally in <code>%APPDATA%/nexus/.env</code> and never leave your machine.
        {provider === "openrouter" && <> Get a free key at <code>openrouter.ai/keys</code>.</>}
      </div>
    </div>
  );
}