//! 测试：非 JSON 模式默认文件名与 JSON 模式返回路径的一致性（学习注释）
//! - 目标：当使用 `--out-priv` 且不提供值时，默认写入 `privkey.hex`；JSON 输出的 `file` 字段应为规范化绝对路径
//! - 步骤：
//!   1) 在临时目录创建 keystore
//!   2) 使用 `--json --out-priv` 导出私钥
//!   3) 解析 stdout JSON，比较 `file` 与期望路径（做路径归一化）
//!   4) 断言文件存在且内容与 `privkey_hex` 一致

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn export_out_priv_default_filename() {
    // 创建临时目录与测试所需的变量（keystore 路径、口令、固定助记词）
    // 使用固定助记词保证测试确定性
    let dir = tempfile::tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let pwd = "PwD_123456";
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // 1) 先创建 keystore（覆盖已存在文件以保证幂等）
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "keystore",
            "create",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--password-stdin",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
        ])
        .write_stdin(pwd)
        .assert()
        .success();

    // 2) 导出私钥（`--out-priv` 无显式值 -> 默认文件名 privkey.hex；同时开启 `--json`）
    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "keystore",
            "export",
            "--file",
            ks.to_str().unwrap(),
            "--password-stdin",
            "--out-priv",
            "--json",
        ])
        .write_stdin(pwd)
        .assert()
        .success();

    // 3) 从 JSON stdout 读取 file 字段，并与期望路径比较（路径归一化后比较）
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    let file_str = v.get("file").and_then(|x| x.as_str()).unwrap();

    // 期望文件是临时目录下的 privkey.hex
    let expected = dir.path().join("privkey.hex");

    // 路径归一化后比较（兼容 macOS /var ↔ /private/var、Windows 前缀 \\?\ 场景）
    let printed_abs =
        fs::canonicalize(PathBuf::from(file_str)).unwrap_or_else(|_| PathBuf::from(file_str));
    let expected_abs = fs::canonicalize(&expected).unwrap_or(expected.clone());
    assert_eq!(printed_abs, expected_abs);

    // 4) 断言文件存在，且文件内容与 JSON 中的 `privkey_hex` 完全一致
    assert!(expected.exists());
    let file_hex = fs::read_to_string(&expected).unwrap();
    let json_hex = v
        .get("privkey_hex")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();
    assert_eq!(file_hex.trim(), json_hex);

    // 清理临时目录（忽略错误）
    dir.close().ok();
}
