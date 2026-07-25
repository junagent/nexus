import { useState, useEffect, useCallback } from "react";
import type { Screen, SystemInfo } from "./types";
import { getSystemInfo } from "./api";
import Sidebar from "./components/Sidebar";
import ChatScreen from "./screens/Chat";
import SessionsScreen from "./screens/Sessions";
import ProvidersScreen from "./screens/Providers";
import SkillsScreen from "./screens/Skills";
import McpScreen from "./screens/Mcp";
import TraceScreen from "./screens/Trace";
import BanditScreen from "./screens/Bandit";
import ApprovalsScreen from "./screens/Approvals";
import SettingsScreen from "./screens/Settings";
import "./App.css";

function App() {
  const [screen, setScreen] = useState<Screen>("chat");
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);
  const [serverRunning, setServerRunning] = useState(false);

  useEffect(() => {
    getSystemInfo().then(setSysInfo).catch(console.error);
    const i = setInterval(async () => {
      try {
        const m = await import("./api");
        const s = await m.agentServerStatus();
        setServerRunning(s.running);
      } catch {}
    }, 5000);
    return () => clearInterval(i);
  }, []);

  const render = useCallback(() => {
    switch (screen) {
      case "chat": return <ChatScreen />;
      case "sessions": return <SessionsScreen />;
      case "providers": return <ProvidersScreen />;
      case "skills": return <SkillsScreen />;
      case "mcp": return <McpScreen />;
      case "trace": return <TraceScreen />;
      case "bandit": return <BanditScreen />;
      case "approvals": return <ApprovalsScreen />;
      case "settings": return <SettingsScreen />;
    }
  }, [screen]);

  return (
    <div className="app">
      <div className="bg-grid" />
      <div className="bg-glow" />
      <Sidebar currentScreen={screen} onNavigate={setScreen} approvalCount={0} serverRunning={serverRunning} />
      <main className="main-content">{render()}</main>
      <footer className="status-bar">
        <span className="status-item">{sysInfo?.version ? `v${sysInfo.version}` : ""}</span>
        <span className="status-item">{sysInfo?.active_provider || "no provider"}</span>
        <span className="status-item">{serverRunning ? "API: 18789" : "API: offline"}</span>
      </footer>
    </div>
  );
}
export default App;