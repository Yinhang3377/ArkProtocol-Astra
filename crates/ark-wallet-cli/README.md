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

注意
- 非 TTY 下 --password-prompt 会回退从 STDIN 读取（设 ARK_WALLET_WARN_NO_TTY=1 可提示）。
- 未加 --overwrite 且目标已存在会失败。