use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn export_out_priv_default_filename() {
    let dir = tempfile::tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let pwd = "PwD_123456";
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // 先创建 keystore
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

    // 导出私钥（--out-priv 无值，默认 privkey.hex；同时 --json）
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

    // 从 JSON 读取 file 字段
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    let file_str = v.get("file").and_then(|x| x.as_str()).unwrap();

    let expected = dir.path().join("privkey.hex");

    // 路径归一化后比较（兼容 macOS /var ↔ /private/var、Windows \\?\）
    let printed_abs =
        fs::canonicalize(PathBuf::from(file_str)).unwrap_or_else(|_| PathBuf::from(file_str));
    let expected_abs = fs::canonicalize(&expected).unwrap_or(expected.clone());
    assert_eq!(printed_abs, expected_abs);

    // 文件内容与 JSON 中的 hex 一致
    assert!(expected.exists());
    let file_hex = fs::read_to_string(&expected).unwrap();
    let json_hex = v
        .get("privkey_hex")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();
    assert_eq!(file_hex.trim(), json_hex);

    dir.close().ok();
}
