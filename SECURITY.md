# Security Policy

## Reporting a Vulnerability

如果你发现 Nexus 项目中的安全问题，请通过以下方式**私下**报告：

- **Email**: 164005568@qq.com
- **GitHub Security Advisory**: https://github.com/junagent/nexus/security/advisories/new

**请勿**通过公开 issue 报告安全问题。

### 报告内容

请在报告中包含：
1. 问题描述和类型
2. 复现步骤
3. 影响范围（哪些版本受影响）
4. 可能的修复建议（如有）

## 响应时间

- 收到报告后 **48 小时内**确认收到
- **7 天内**评估并回复修复计划
- 修复完成后公开披露

## 安全策略

### Agent 工具调用

Nexus 的所有工具调用（shell、文件写入、网络请求）都经过 **Approval Gateway** 审查：

- 敏感操作需要用户显式确认
- Bandit 模块进行安全策略检查
- 所有操作记录在 Trace 系统中可追溯

### API 安全

Agent Server (端口 18789) 的安全建议：
- 生产环境请通过防火墙限制访问
- 建议在内部网络使用
- 如需外部访问，请配置认证中间件

### LLM API Keys

API Keys 存储在 `~/.nexus/.env`（Linux/macOS）或 `%APPDATA%/nexus/.env`（Windows）中：
- 文件权限默认 600（仅所有者可读）
- 请定期检查密钥轮换

### 依赖管理

- 所有 Rust 依赖通过 `Cargo.toml` 锁定
- 建议定期运行 `cargo audit` 检查依赖漏洞（本地手动执行即可）

## 已知问题

当前版本已知安全问题：

- 暂无公开已知问题
