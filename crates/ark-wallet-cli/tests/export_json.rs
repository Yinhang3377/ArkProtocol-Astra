//! 测试：keystore export 的 JSON 输出（学习注释）
//! - export_json_prints_privkey：--json 时 stdout 含 privkey_hex
//! - export_json_writes_file_path：提供 --out-priv 时，JSON 含 file 且写入文件成功
//! - 注意：不在测试中打印私钥明文到日志

use assert_cmd::Command;
use serde_json::Value;
use std::{fs, path::Path};

fn create_keystore(ks: &Path, pwd: &str, mn: &str) {
    Command::cargo_bin("ark-wallet")
        .unwrap()
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
}

#[test]
fn export_json_prints_privkey() {
    let dir = tempfile::tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let pwd = "PwD_123456";
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    create_keystore(&ks, pwd, mn);

    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "export",
            "--file",
            ks.to_str().unwrap(),
            "--password-stdin",
            "--json",
        ])
        .write_stdin(pwd)
        .assert()
        .success();

    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    let hex = v.get("privkey_hex").and_then(|x| x.as_str()).unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    dir.close().ok();
}

#[test]
fn export_json_writes_file_path() {
    let dir = tempfile::tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let out = dir.path().join("priv.hex");
    let pwd = "PwD_123456";
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    create_keystore(&ks, pwd, mn);

    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "export",
            "--file",
            ks.to_str().unwrap(),
            "--password-stdin",
            "--out-priv",
            out.to_str().unwrap(),
            "--json",
        ])
        .write_stdin(pwd)
        .assert()
        .success();

    let body = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    // Canonicalize both paths to avoid macOS /private vs /var mismatch
    let file_out = v.get("file").and_then(|x| x.as_str()).unwrap();
    let file_out_canon = fs::canonicalize(std::path::Path::new(file_out)).unwrap();
    let out_canon = fs::canonicalize(&out).unwrap();
    assert_eq!(file_out_canon, out_canon);
    assert!(out.exists());

    // 文件内容和 JSON 中的 hex 一致
    let file_hex = fs::read_to_string(&out).unwrap();
    let json_hex = v
        .get("privkey_hex")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();
    assert_eq!(file_hex.trim(), json_hex);
    dir.close().ok();
}
