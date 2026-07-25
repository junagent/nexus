<p align="center">
  <a href="https://github.com/junagent/nexus">
    <img src="public/nexus-logo.svg" width="500" alt="Nexus Logo">
  </a>
</p>

<p align="center">
  <strong>The Core of Your AGI.</strong><br>
  A high-performance desktop agent — Rust · Tauri · React
</p>

<p align="center">
  <a href="#-architecture">Architecture</a> •
  <a href="#-brand-identity">Brand Identity</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-building">Building</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.97+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-2.0+-purple?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/React-19-blue?logo=react" alt="React">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT">
  <a href="https://github.com/junagent/nexus/actions/workflows/build.yml">
    <img src="https://github.com/junagent/nexus/actions/workflows/build.yml/badge.svg" alt="Build Status">
  </a>
</p>

---

> **Nexus** — a desktop agent born from the hermes-agent-rs engine. Rust-native performance. Tauri shell. Self-evolving AGI core.

## ✦ Brand Identity

**N E X U S** — The name is the logo. The **X** is formed by two intersecting triangles, the central point where all intelligence converges:

```
N  E  ⧉  U  S
   The Core of Your AGI
```

The X-core uses:
- **Cyan triangle** `#00d4ff` — active intelligence, data flow
- **Magenta triangle** `#ff00e4` — creative reasoning, exploration
- **White core dot** — the AGI nexus point, pulsing with energy

The wordmark is built with `JetBrains Mono` — geometric, sharp, monospaced. Every letter is a structural element.

## ✦ Architecture

```
┌──────────────────────────────────────────┐
│             Nexus Desktop App            │
├──────────────┬───────────────────────────┤
│  Frontend    │  Backend (Rust + Tauri)   │
│  React 19    │                           │
│  Vite 6      │  Agent Engine             │
│  TS 5.6      │  Provider Router          │
│              │  Skill Manager            │
│  Dark UI     │  Session Store            │
│  Chat +      │  Config System            │
│  Config      │  Gateway Manager          │
├──────────────┴───────────────────────────┤
│  hermes-agent-rs (Lumio Research)        │
│  AgentLoop · LlmProvider · ToolHandler   │
│  PlatformAdapter · Memory · Cron · MCP   │
├──────────────────────────────────────────┤
│  Windows · macOS · Linux                 │
└──────────────────────────────────────────┘
```

10 LLM providers · 30+ tool backends · 17 platform gateways · 8 memory backends

## ✦ Quick Start

```bash
# Prerequisites
# ─ Windows: Visual Studio Build Tools (msvc linker)
# ─ macOS:   brew install webkit2gtk
# ─ Linux:   sudo apt install libwebkit2gtk-4.1-dev

git clone https://github.com/junagent/nexus.git
cd nexus
npm install
npm run tauri dev
```

## ✦ Building

```bash
# Development (hot-reload frontend + backend)
npm run tauri dev

# Production installer
npm run tauri build
```

Output installers:
- **Windows:** `src-tauri/target/release/bundle/msi/Nexus_*.msi`
- **macOS:** `src-tauri/target/release/bundle/dmg/Nexus_*.dmg`
- **Linux:** `src-tauri/target/release/bundle/deb/nexus_*.deb`

> **GitHub Actions** builds all three platforms automatically on every push. Download the latest release from the [Releases page](https://github.com/junagent/nexus/releases).

## ✦ Configuration

Config directory: `~/.nexus/` (Linux/macOS) or `%APPDATA%/nexus/` (Windows)

Set API keys in `~/.nexus/.env`:
```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-...
```

## ✦ Credits

- **[Nous Research](https://nousresearch.com)** — Original Hermes Agent
- **[Lumio Research](https://github.com/Lumio-Research/hermes-agent-rs)** — Rust port
- **[Tauri](https://tauri.app)** — Desktop framework

## ✦ License

MIT — see [LICENSE](LICENSE)

---

<p align="center">
  <sub>Built with Rust · Tauri · ❤️</sub>
</p>
