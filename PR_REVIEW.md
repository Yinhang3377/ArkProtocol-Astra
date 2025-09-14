PR 审查与合并说明（中文）

精简摘要
- 引入统一的安全错误类型 `SecurityError`，在 CLI 顶层用 `anyhow` 包装并映射为确定的退出码；
- 强制 CLI 使用 Base58Check 地址格式（新增 `from_pubkey_b58check`），并私有化旧的无校验和 API 以便逐步弃用；
- 实现 `secure_atomic_write` 用于 keystore 与导出操作，降低写入中断导致的密钥泄露或文件损坏风险；
- 已在本地对 `ark-wallet-cli` 运行 `cargo build` 与 `cargo test`（本地通过）；分支：`security/error-refactor`。

详细审查意见（合并前必读）

概览
本分支对安全边界与错误传递进行了重构，目标明确：统一安全错误、在 CLI 提供确定性退出码、提升地址编码安全与改进磁盘写入原子性。实现方向正确，代码清晰，使用 `Zeroizing` 等安全用法符合最佳实践。合并前需要解决 CI 失败并补充少量文档。

主要变更
- 新增 `SecurityError` 并在 KDF、编码、文件 IO、钱包相关模块逐步替换旧错误类型。
- CLI 顶层改为 `run() -> anyhow::Result<()>`，并实现 `exit_code_for_error` 将错误映射为确定的退出码。
- 强制 Base58Check 地址解析（新增 `from_pubkey_b58check`），旧 API 私有化为 `_legacy_*`。
- 实现 `secure_atomic_write`（临时文件写入 -> flush/fsync -> 重命名 -> 目录 fsync），用于 keystore、导出私钥等路径。

安全与兼容性建议
- 在 `README` 或 `CHANGELOG` 中加入退出码表与 Base58Check 迁移说明，帮助运营/用户了解变更。
- 为常见外部错误（如 `serde_json::Error`、`bs58::decode::Error`）实现 `From` 到 `SecurityError`，以保证错误映射一致并保留原始原因。
- 检查 `secure_atomic_write` 在 Windows runner 上的行为，确保目录同步与权限处理与 Unix 的实现兼容。

CI 与测试
- 本地：`cargo fmt`、`cargo build -p ark-wallet-cli`、`cargo test -p ark-wallet-cli` 在本地通过。
- 远程：GitHub Actions 对最近提交有失败 job（请在 PR Checks 中查看失败 job 的日志并粘贴关键错误给我，我会定位并修复）。

合并前必做清单
1. 确保所有 CI job 通过（先定位失败日志并修复）。
2. 在仓库文档中添加迁移说明与退出码表。
3. 在 Windows runner 上验证 `secure_atomic_write` 的行为并修复可能的跨平台差异。
4. 为外部常见错误实现 `From` 转换以保证错误一致性。

合并后建议
- 在其它 crates 中逐步采用类似的错误策略或建立统一错误约定。
- 在 CI 中增加针对 Windows 与 Linux 的 `secure_atomic_write` 测试。
- （可选）提供一个兼容标志 `--allow-legacy-addresses` 以便用户平滑迁移。

如果你想我直接把这些审查意见作为 PR Review 发布：
- 我可以用 `gh` CLI 直接提交 Review（需先在本机完成 `gh auth login`）；或
- 我可以把这些内容留在仓库（已完成），你可以在 PR 页直接打开 `PR_REVIEW.md`，复制粘贴为 Review。

—— 结束 ——
