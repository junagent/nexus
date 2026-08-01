<p align="center">
  <a href="https://github.com/junagent/nexus">
    <img src="public/nexus-logo.svg" width="500" alt="Nexus Logo">
  </a>
</p>

<p align="center">
  <strong>The Core of Your AGI.</strong><br>
  A high-performance desktop agent — Rust · Tauri · Yew
</p>

<p align="center">
  <a href="#-architecture">Architecture</a> &bull;
  <a href="#-brand-identity">Brand Identity</a> &bull;
  <a href="#-quick-start">Quick Start</a> &bull;
  <a href="#-building">Building</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.97+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-2.0+-purple?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/Yew-0.23-lightgrey?logo=rust" alt="Yew">
  <img src="https://img.shields.io/badge/WASM-WebAssembly-green?logo=webassembly" alt="WASM">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT">
  <a href="https://github.com/junagent/nexus/actions/workflows/build.yml">
    <img src="https://github.com/junagent/nexus/actions/workflows/build.yml/badge.svg" alt="Build Status">
  </a>
</p>

---

> **Nexus** — 一个原生 Rust 桌面 Agent，基于 Tauri + Yew WASM 构建。
> 全 Rust 技术栈：前端 Yew 编译为 WebAssembly，后端 Tauri 提供系统能力，
> 内置 Agent Engine、MCP 支持、10+ LLM Provider 和可扩展工具系统。

## ✦ Brand Identity

**N E X U S** — 名字即 Logo。两个相交三角形组成 **X**，代表所有智能汇聚的中心点：

```
N  E  ⧉  U  S
   The Core of Your AGI
```

X-core 配色：
- **Cyan triangle** `#00d4ff` — 活跃智能、数据流
- **Magenta triangle** `#ff00e4` — 创造性推理、探索
- **White core dot** — AGI 汇聚点，持续脉冲

## ✦ Architecture

```
┌───────────────────────────────────────────────┐
│              Nexus Desktop App                │
├────────────────────┬──────────────────────────┤
│  Frontend          │  Backend (Tauri Rust)    │
│  Yew 0.23 (Rust)   │                          │
│  ↓ wasm-pack       │  NexusEngine             │
│  WASM (.wasm)      │  Agent Server :18789     │
│  Tauri Webview     │  Provider Router         │
│                    │  Skill Store             │
│                    │  MCP Client              │
│                    │  Tool Registry           │
│                    │  Approval Gateway        │
│                    │  Session / Trace         │
├────────────────────┴──────────────────────────┤
│  hermes-agent-rs (Lumio Research)             │
│  AgentLoop · LlmProvider · ToolHandler        │
│  PlatformAdapter · Memory · Cron              │
├───────────────────────────────────────────────┤
│  Windows · macOS · Linux                      │
└───────────────────────────────────────────────┘

Key modules: agent · agent_server · approval · bandit
             commands · mcp · providers · skill_store
             tools · trace
```

10 LLM providers &middot; 30+ tool backends &middot; MCP support &middot; Agent server API

## ✦ Quick Start

### Prerequisites

- **Rust** 1.97+ (`rustup install stable`)
- **wasm-pack** (`cargo install wasm-pack`)
- **Tauri CLI** (`cargo install tauri-cli --version "^2"`)
- **Visual Studio Build Tools** (Windows, msvc linker)
- **libwebkit2gtk-4.1-dev** (Linux)
- **Xcode command-line tools** (macOS)

### Install & Run

```bash
git clone https://github.com/junagent/nexus.git
cd nexus
cargo tauri dev
```

## ✦ Building

```bash
# Development (hot-reload)
cargo tauri dev

# Production installer
cargo tauri build
```

### Build Output

```
Windows:  target/x86_64-pc-windows-msvc/release/bundle/msi/Nexus_*.msi
macOS:    target/aarch64-apple-darwin/release/bundle/dmg/Nexus_*.dmg
Linux:    target/x86_64-unknown-linux-gnu/release/bundle/deb/nexus_*.deb
```

### CI/CD

GitHub Actions 自动构建所有三个平台。安装程序可通过 [Releases 页面](https://github.com/junagent/nexus/releases) 获取。

> 构建需要以下 [Actions Secrets](https://github.com/junagent/nexus/settings/secrets/actions)：
> - `TAURI_PRIVATE_KEY` — Tauri 签名私钥
> - `TAURI_KEY_PASSWORD` — 私钥密码

## ✦ Configuration

Config 目录：`~/.nexus/`（Linux/macOS）或 `%APPDATA%/nexus/`（Windows）

在 `~/.nexus/.env` 中设置 API keys：
```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-...
```

## ✦ Agent Server

启动后 Nexus 自动在 **端口 18789** 启动 Agent Server，提供 HTTP API：
```bash
curl -X POST http://localhost:18789/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"hello"}'
```

## ✦ Project Structure

```
nexus/
├── Cargo.toml              # Workspace (frontend + src-tauri)
├── frontend/               # Yew WASM 前端 (Rust → .wasm)
│   ├── Cargo.toml
│   └── src/lib.rs
├── src-tauri/              # Tauri 后端
│   ├── tauri.conf.json     # Tauri 配置
│   ├── src/
│   │   ├── lib.rs          # 入口，模块声明
│   │   ├── agent/          # NexusEngine, Agent Loop
│   │   ├── agent_server/   # HTTP Agent Server
│   │   ├── approval/       # 工具调用审批
│   │   ├── bandit/         # 安全策略
│   │   ├── commands/       # Tauri 命令 (chat, config, system, skills...)
│   │   ├── mcp/            # MCP 客户端
│   │   ├── providers/      # LLM Provider Router
│   │   ├── skill_store/    # 技能管理
│   │   ├── tools/          # 工具注册表 (shell, file, web...)
│   │   └── trace/          # 追踪系统
├── .github/workflows/      # CI/CD
└── public/                 # 静态资源
```

## ✦ Credits

- **[Lumio Research](https://github.com/Lumio-Research/hermes-agent-rs)** — Hermes Agent Rust 移植
- **[Nous Research](https://nousresearch.com)** — 原始 Hermes Agent
- **[Tauri](https://tauri.app)** — 桌面应用框架
- **[Yew](https://yew.rs)** — Rust WebAssembly 框架

## ✦ License

MIT — 详见 [LICENSE](LICENSE)

---

<p align="center">
  <sub>Built with Rust · Tauri · Yew &middot; Made with ❤️</sub>
</p>