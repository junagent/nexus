<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="public/nexus-logo.svg">
    <img src="public/nexus-logo.svg" width="180" alt="Nexus Logo" style="filter: drop-shadow(0 0 20px rgba(0,212,255,0.3));">
  </picture>
</p>

<h1 align="center">Nexus</h1>

<p align="center">
  <strong>The Core of Your AGI.</strong><br>
  A high-performance desktop agent — Rust + Tauri + React<br>
  Built on the <code>hermes-agent-rs</code> engine.
</p>

<p align="center">
  <a href="#-architecture">Architecture</a> •
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-building">Building</a> •
  <a href="#-visual-design">Visual Design</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.97+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-2.0+-purple?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/React-19-blue?logo=react" alt="React">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT">
</p>

---

> **Nexus** reimagines the Hermes Agent as a native desktop experience. Written in Rust with Tauri for the GUI layer, it combines the *self-evolving AI agent* philosophy of Nous Research's Hermes Agent with the raw performance of a compiled binary.

## ✦ Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Nexus Desktop App                      │
├──────────────────────┬───────────────────────────────────┤
│   Frontend (React)   │   Backend (Rust + Tauri)          │
│                      │                                   │
│  ┌────────────────┐  │  ┌─────────────────────────────┐  │
│  │  Chat UI       │  │  │  Tauri Commands              │  │
│  │  Config Panel  │◄─IPC─►│  Agent Engine               │  │
│  │  Skill Browser │  │  │  Provider Router             │  │
│  │  Gateway View  │  │  │  Skill Manager               │  │
│  └────────────────┘  │  │  Session Store               │  │
│                      │  │  Config System                │  │
│  Vite + TypeScript   │  │  └─────────────────────────┘  │
│                      │  │                                │
├──────────────────────┴───────────────────────────────────┤
│  hermes-agent-rs                                        │
│  ┌────────────────────────────────────────────────────┐  │
│  │ AgentLoop · LlmProvider (10) · ToolHandler (30+)   │  │
│  │ PlatformAdapter (17) · MemoryProvider (8)          │  │
│  │ SkillOrchestrator · Cron · Gateway · MCP/ACP      │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  OS Layer (Windows · macOS · Linux)                      │
└──────────────────────────────────────────────────────────┘
```

**Key traits from hermes-agent-rs:**
- **`LlmProvider`** — Anthropic, OpenAI, OpenRouter, DeepSeek, Google, and 5 more
- **`ToolHandler`** — 30+ tool backends (files, browser, code, web, vision, etc.)
- **`PlatformAdapter`** — 17 messaging platforms (Telegram, Discord, WhatsApp, etc.)
- **`TerminalBackend`** — Local, Docker, SSH, Daytona, Modal, Singularity
- **`MemoryProvider`** — 8 memory backends (SQLite, Redis, vector stores, etc.)

## ✦ Visual Identity

```
             ▲
            /█\
           / █ \
          /  █  \
         /   █   \
        /    █    \
       /─────█─────\
      / █    █    █ \
     /  █    █    █  \
    /   █    █    █   \
   /    █    █    █    \
  /─────█────█────█─────\
 █ █   █    ███    █   █ █
      █      █      █
     █       █       █
    █        █        █
   █         █         █
  ███████████████████████
```

The logo uses **intersecting equilateral triangles** as core motif — representing:
- **Three layers** of AGI: Perception → Reasoning → Action
- **Triangular mesh** as a network of intelligent nodes
- **Central glow** — the Nexus core where all connections converge

Color palette:
- `#00d4ff` — Cyan (primary accent, active intelligence)
- `#ff00e4` — Magenta (creative reasoning)
- `#00ff88` — Green (growth, learning)
- `#7b00ff` — Purple (depth, complexity)

## ✦ Features

- **⚡ Native performance** — Rust binary, no Electron overhead, no Python GIL
- **🧠 Multi-provider** — 10 LLM providers with hot-swappable models
- **🔌 17 platform gateways** — Telegram, Discord, Slack, WhatsApp, Signal, Matrix + 11 more
- **🛠️ 30+ tool backends** — File ops, browser, code execution, vision, web search, Home Assistant
- **💾 Multi-memory** — SQLite, Redis, vector stores, filesystem, and more
- **🎨 Cyberpunk UI** — Dark tech aesthetic with animated triangle-node network
- **🔄 Session management** — Multiple conversations, history, search
- **📦 Skill ecosystem** — Install, manage, and create skills from the community hub

## ✦ Quick Start

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust 1.75+
npm install -g pnpm                                                # Node 20+

# Clone
git clone https://github.com/junagent/nexus.git
cd nexus

# Install frontend deps
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## ✦ Configuration

Nexus stores its configuration at:
- **Linux/macOS:** `~/.nexus/`
- **Windows:** `%APPDATA%/nexus/`

Set your API keys in `~/.nexus/.env`:
```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-...
```

Or use the Nexus Engine Config panel in the desktop app.

## ✦ Building

```bash
# Development
cd nexus
npm run tauri dev    # Hot-reload frontend + Rust backend

# Release build
npm run tauri build  # Produces platform-specific installer

# Cross-platform (from CI)
cargo build --release
```

## ✦ Credits

- **[Nous Research](https://nousresearch.com)** — Original Hermes Agent vision
- **[Lumio Research](https://github.com/Lumio-Research/hermes-agent-rs)** — Rust port of Hermes Agent
- **[Tauri](https://tauri.app)** — Desktop framework

## ✦ License

MIT — see [LICENSE](LICENSE).

---

<p align="center">
  <sub>Built with Rust, Tauri, and ❤️</sub>
</p>
