import { useState } from "react";
import { setProvider } from "../api";

export default function ProvidersScreen() {
  const [provider, setProv] = useState("openrouter");
  const [model, setModel] = useState("anthropic/claude-sonnet-4");
  const [status, setStatus] = useState("");

  const apply = async () => {
    try {
      await setProvider(provider, model);
      setStatus(`✅ Active: ${provider}/${model}`);
    } catch (e) { setStatus(`❌ ${e}`); }
  };

  return (
    <div className="screen">
      <h2 className="screen-title">Providers</h2>
      <div className="config-section">
        <label className="config-label">LLM Provider</label>
        <select className="config-select" value={provider} onChange={e => setProv(e.target.value)}>
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic</option>
          <option value="deepseek">DeepSeek</option>
          <option value="openrouter">OpenRouter</option>
          <option value="google">Google AI</option>
        </select>
      </div>
      <div className="config-section">
        <label className="config-label">Model</label>
        <input className="config-input" value={model} onChange={e => setModel(e.target.value)} />
      </div>
      <button className="btn-primary" onClick={apply}>Apply</button>
      {status && <p className="status-msg">{status}</p>}
      <div className="config-info" style={{ marginTop: 16 }}>
        Set API keys in <code>%APPDATA%/nexus/.env</code>
      </div>
    </div>
  );
}