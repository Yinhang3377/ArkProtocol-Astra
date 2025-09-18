# ark-wallet-cli TODOs and security notes

This file contains short-term TODOs, missing tests and security notes discovered during an automated audit.

Security notes
- Clearly document threat model for `--json` outputs and `--out-priv` operations. Exporting private keys should be discouraged in production and documented.
- Document the relay threat model; warn that posting ephemeral keys to untrusted relays weakens security.
- Recommend default KDF parameters and explain trade-offs; consider stronger defaults for production (increase PBKDF2 iterations / scrypt N).

Suggested tests to add (covered by tests/ but recommended for CI):
1. keystore_tamper.rs — Negative test: tampered ciphertext should fail decryption (integrity check)
2. keystore_invalid_nonce.rs — Negative test: invalid base64/nonce should return decode error
3. invalid_kdf_params_additional.rs — Additional negative KDF parameter tests for CLI paths
4. non_interactive_password_flow.rs — Simulation of non-TTY password input behavior (document expected behavior)

Other improvements
- Add a short `docs/security.md` describing best practices and recommended CLI usage in CI.
- Add explicit CLI examples showing safe workflows (hardware wallet, offline signing, QR envelope flows).

PR policy for auto-fixes
- Small style fixes (linting, unused mut removal) are safe, but push/merge should be gated by a human review. The audit scripts will not push further changes without explicit confirmation.
# Ark Wallet CLI 安全与改进待办

进度
- [x] security 模块引入（fs/codec/kdf/errors）
- [x] keystore/导出写盘改为 secure_atomic_write，Windows 路径规范化（dunce）
- [x] 测试全绿

下一步（优先级从高到低）
- [ ] 接入 KDF 校验：使用 security::validate_kdf_choice/params；新增弱参数拒绝测试
- [ ] 地址校验：启用 Base58Check（from_pubkey_b58check），补充 checksum 失败测试与 CLI 输出选项
- [ ] 加密随机性测试：同明文+口令多次加密密文应不同（nonce/salt 唯一）
- [ ] 错误口径与脱敏：统一错误消息/枚举，屏蔽 AEAD 细节；新增 cli_password_errors 与敏感信息不外泄测试
- [ ] 去除 #[allow(dead_code)]：上述功能接入后清理
- [ ] HD 健壮性测试：非法路径、passphrase 影响、属性测试（proptest）
- [ ] README 安全设计小节：原子写入/权限、路径规范化、随机性、错误脱敏与 Zeroize
- [ ] 备份/恢复与清理子命令（可选，加分项）

备注
- 安全写盘已在 create/export 两处接入，确认未来新增写文件路径也统一调用 security::secure_atomic_write