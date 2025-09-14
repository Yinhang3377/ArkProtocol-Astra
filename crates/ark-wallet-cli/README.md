# ark-wallet-cli

示例（Windows PowerShell）
- 创建（交互确认）
  cargo run -p ark-wallet-cli -- keystore create --mnemonic "<助记词>" --lang en --password-prompt --password-confirm --out .\keystore.json --overwrite
- 创建（STDIN）
  echo "MyPwd123!" | cargo run -p ark-wallet-cli -- keystore create --mnemonic "<助记词>" --lang en --password-stdin --out .\keystore.json --overwrite
- 导入（STDIN）
  echo "MyPwd123!" | cargo run -p ark-wallet-cli -- keystore import --file .\keystore.json --password-stdin --full
- 导出（JSON）
  cargo run -p ark-wallet-cli -- keystore export --file .\keystore.json --password-prompt --json
- 导出到文件（默认名）
  cargo run -p ark-wallet-cli -- keystore export --file .\keystore.json --password-prompt --out-priv --json

JSON 输出
- create --json: { "address", "path", "file" }
- export --json: { "privkey_hex" } 或 { "privkey_hex", "file" }

## 导出私钥

- 非 JSON（默认）：当提供 `--out-priv` 时，CLI 将私钥十六进制写入文件，stdout 仅打印该文件的规范化绝对路径。
```powershell
ark-wallet keystore export --file keystore.json --password "pwd" --out-priv priv.hex
# 输出示例：
C:\path\to\project\priv.hex
```

- JSON 模式：stdout 打印包含 `file` 和 `privkey_hex` 的 JSON（若设置 `--out-priv` 也会写文件）。
```powershell
ark-wallet --json keystore export --file keystore.json --password "pwd" --out-priv priv.hex
```
```json
{
  "file": "C:\\path\\to\\project\\priv.hex",
  "privkey_hex": "abcd..."
}
```

注意
- `--password-stdin` 与 `--password-prompt` 互斥于 `--password`。
- 非交互环境使用 `--password-prompt` 时会从 STDIN 读取；设置 `ARK_WALLET_WARN_NO_TTY=1` 可显示提示。

## License / Usage
This repository is source-available for viewing only. All rights reserved.
You may not use, copy, modify, distribute, or publish this code without
explicit written permission from the author.

## 安全路线与改进待办

已记录当前进度与后续事项。当前状态

- 新增 security 模块：fs/codec/kdf/errors
- create/export 已接入 secure_atomic_write，并用 dunce::canonicalize 修正 Windows 路径；全部测试通过。
- codec/kdf/errors/address 中有临时 #[allow(dead_code)]，等待接入后移除。

建议把待办保存为仓库内 TODO，便于后续继续。

进度
- [x] security 模块引入 (fs/codec/kdf/errors)
- [x] keystore/导出写盘改为 secure_atomic_write，Windows 路径规范化 (dunce)
- [x] 测试全绿

下一步（优先级从高到低）
- [ ] 接入 KDF 校验：使用 security::validate_kdf_choice/params；新增弱参数拒绝测试
- [ ] 地址校验：启用 Base58Check (from_pubkey_b58check)，补充 checksum 失败测试与 CLI 输出选项
- [ ] 加密随机性测试：同明文+口令多次加密密文应不同（nonce/salt 唯一）
- [ ] 错误口径与脱敏：统一错误消息/枚举，屏蔽 AEAD 细节；新增 cli_password_errors 与敏感信息不外泄测试
- [ ] 去除 #[allow(dead_code)]：上述功能接入后清理
- [ ] HD 健壮性测试：非法路径、passphrase 影响、属性测试（proptest）
- [ ] README 安全设计小节：原子写入/权限、路径规范化、随机性、错误脱敏与 Zeroize
- [ ] 备份/恢复与清理子命令（可选，加分项）

备注
- 安全写盘已在 create/export 两处接入，确认未来新增写文件路径也统一调用 security::secure_atomic_write

建议提交保存
- git add crates/ark-wallet-cli/README.md crates/ark-wallet-cli/TODO.md
- git commit -m "docs: add TODO for security roadmap"
- git push

下次继续时，我会按此清单推进（从 KDF 校验与 Base58Check 接入开始）。