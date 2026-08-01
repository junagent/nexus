# Contributing to Nexus

感谢你对 Nexus 项目的兴趣！以下是贡献指南。

## 环境准备

### 必需工具

- **Rust** 1.97+（推荐 `rustup` 安装）
- **wasm-pack**：`cargo install wasm-pack`
- **Tauri CLI**：`cargo install tauri-cli --version "^2"`

### 平台依赖

| 平台 | 依赖 |
|------|------|
| Windows | Visual Studio Build Tools (msvc linker) |
| macOS | Xcode command-line tools |
| Linux | `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` |

## 开发

```bash
# 克隆
git clone https://github.com/junagent/nexus.git
cd nexus

# 开发模式（热重载）
cargo tauri dev

# 生产构建
cargo tauri build
```

## 代码规范

### Rust

```bash
# 格式化
cargo fmt

# 代码检查（CI 会严格检查）
cargo clippy --all-targets --all-features -- -D warnings

# 运行测试
cargo test --all
```

PR 前必须通过 `cargo fmt` 和 `cargo clippy -D warnings`。

### 提交信息

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
type: description

fix: correct .gitignore pattern
docs: update README architecture diagram
feat: add new tool handler
ci: add lint-and-test job
```

类型：`fix` / `feat` / `docs` / `ci` / `chore` / `refactor` / `test`

## 项目结构

```
nexus/
├── Cargo.toml              # Workspace 定义
├── frontend/               # Yew WASM 前端
│   └── src/lib.rs
├── src-tauri/              # Tauri 后端
│   ├── tauri.conf.json
│   └── src/
│       ├── agent/          # 核心 Agent Engine
│       ├── agent_server/   # HTTP API Server (:18789)
│       ├── approval/       # 工具调用审批
│       ├── bandit/         # 安全策略
│       ├── commands/       # Tauri 命令
│       ├── mcp/            # MCP 客户端
│       ├── providers/      # LLM Provider Router
│       ├── skill_store/    # 技能管理
│       ├── tools/          # 工具注册表
│       └── trace/          # 追踪系统
└── .github/workflows/      # CI/CD
```

## 提交 PR

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feat/your-feature`
3. 提交变更并推送
4. 确保 CI 通过（lint + test + build）
5. 提交 Pull Request，描述变更内容

## 安全报告

发现安全问题？请勿公开 issue。参见 [SECURITY.md](./SECURITY.md)。
