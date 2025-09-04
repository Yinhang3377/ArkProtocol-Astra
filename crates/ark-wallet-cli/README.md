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