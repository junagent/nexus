# Nexus — Development Roadmap

> **"Nexus. The Core of Your AGI."**

Based on analysis of top Rust AI agent open-source projects (2026).

## 🏆 Competitor Landscape

| Project | Stars | Key Differentiator | What to Steal |
|---------|-------|-------------------|---------------|
| [Moltis](https://github.com/moltis-org/moltis) | 2,793 | 59-crate personal agent server, sandboxed | Sidecar arch, Voice, Sandbox, Vault |
| [aichat](https://github.com/sigoden/aichat) | 10,275 | CLI tool with RAG, Agents, Shell | RAG system, Shell integration |
| [Hermes-Agent-RS](https://github.com/Lumio-Research/hermes-agent-rs) | 74 | 17 crates, 30+ tools, 17 platforms | Full provider/tool/gateway stack |
| [Hermes-RS](https://github.com/eikarna/hermes-rs) | 63 | Streaming-first, autonomous mode | XML parser, autonomous loop |
| [OpenPawz](https://github.com/OpenPawz/openpawz) | 75 | Tauri v2 + Rust, MCP Bridge, 5MB | Fleet mgmt, MCP Bridge, Knowledge Graph |
| [Bodhi-AI](https://github.com/bigduu/Bodhi-AI) | 14 | Tauri sidecar, workflow scheduling | Sidecar pattern, Global hotkey |
| [Godcoder](https://github.com/eli-labz/Godcoder) | 239 | Local-first coding agent | Local-first BYOK pattern |

## 🎯 Architectural Insights

### Pattern 1: Sidecar Engine (Bodhi, Moltis)
Desktop shell (Tauri) manages a separate agent engine binary lifecycle.
- **Why**: Clean separation, independent scaling, engine restart without UI restart
- **Nexus**: Split into `nexus-desktop` (Tauri shell) + `nexus-engine` (agent binary)

### Pattern 2: MCP Bridge (OpenPawz, Moltis)
Connect to community MCP tool servers for extensibility.
- **Why**: Tap into growing MCP ecosystem (500+ servers), no reinvention
- **Nexus**: Add MCP client + MCP server exposure

### Pattern 3: Memory Graph (OpenPawz)
Visual knowledge graph with force-directed layout.
- **Why**: Persistent recall, cross-session context, agent learning
- **Nexus**: SQLite + vector embeddings + graph explorer

### Pattern 4: Layered Tools (Hermes-RS, Hermes-Agent-RS)
Tool registry → schema generation → sandboxed execution → result validation.
- **Why**: Reliability, type safety, audit trail
- **Nexus**: `ToolRegistry` trait, auto schema from structs, result validation

### Pattern 5: RAG + Local (aichat, Godcoder)
Embed documents → vector store → semantic search → inject into context.
- **Why**: Grounded responses, local knowledge, reduced hallucinations
- **Nexus**: `text-embeddings-inference` + `usearch/lance`

## 🗺️ Development Roadmap

### Phase 1: Core Engine Upgrade (Now)
- [ ] **MCP Client** — Connect to existing MCP tool servers
- [ ] **Tool Registry** — Typed tools with auto schema generation
- [ ] **Memory System** — SQLite-backed conversation history + vector store
- [ ] **File Ops** — Read/write/edit files via LLM tools
- [ ] **Shell Commands** — Sandboxed command execution

### Phase 2: Sidecar Architecture
- [ ] **Engine Binary** — Extract agent loop into `nexus-engine`
- [ ] **Desktop Shell** — Tauri manages engine lifecycle
- [ ] **HTTP Bridge** — Desktop ↔ Engine over localhost HTTP
- [ ] **Auto-update** — Self-updating engine binary

### Phase 3: Advanced Features
- [ ] **RAG System** — Document ingestion + semantic search
- [ ] **Voice I/O** — STT + TTS pipeline
- [ ] **Multi-Agent** — Fleet management, subagents
- [ ] **Knowledge Graph** — Visual memory explorer
- [ ] **Autonomous Mode** — Self-directed task loop (Hermes-RS style)

### Phase 4: Ecosystem
- [ ] **MCP Server** — Expose Nexus as MCP server
- [ ] **Plugin System** — Community extensions
- [ ] **Gateway Hub** — Multi-platform messaging (Telegram/Discord/WeChat)
- [ ] **Scheduling** — Cron-based automated tasks

## 📦 Current State

```
Nexus v0.1.0
├── ✅ Tauri + React desktop shell
├── ✅ NEXUS brand identity (X-core logo)
├── ✅ Multi-provider LLM (OpenAI/DeepSeek/OpenRouter/Anthropic/Google)
├── ✅ Streaming chat
├── ✅ Session management
├── ✅ CI/CD pipeline (Windows/macOS/Linux)
├── ⬜ MCP client integration
├── ⬜ Tool registry with auto-schema
├── ⬜ Memory system (SQLite + vectors)
├── ⬜ File operations tools
├── ⬜ Shell execution (sandboxed)
├── ⬜ Sidecar architecture
└── ⬜ RAG system
```

## 🔗 References

- [Moltis Architecture](https://github.com/moltis-org/moltis) — 59-crate workspace, sandboxed
- [OpenPawz Features](https://github.com/OpenPawz/openpawz) — Tauri v2, MCP Bridge, Fleet
- [Hermes-Agent-RS](https://github.com/Lumio-Research/hermes-agent-rs) — 17-crate monorepo
- [Hermes-RS](https://github.com/eikarna/hermes-rs) — Streaming-first, autonomous
- [Bodhi-AI](https://github.com/bigduu/Bodhi-AI) — Tauri sidecar pattern
- [aichat](https://github.com/sigoden/aichat) — RAG + Shell + Agents
