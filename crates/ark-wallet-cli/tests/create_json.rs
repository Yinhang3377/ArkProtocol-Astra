//! 测试：keystore create 的 JSON 输出（学习注释）
//! - 目标：--json 时应包含 address/path/file 字段
//! - 方法：解析 stdout JSON，断言字段存在与基本格式正确

use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn create_json_prints_address_path_file() {
    let dir = tempfile::tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let pwd = "PwD_123456";
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let path = "m/44'/7777'/0'/0/0";

    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "--json",
            "keystore",
            "create",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--path",
            path,
            "--password-stdin",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
        ])
        .write_stdin(pwd)
        .assert()
        .success();

    // 校验 stdout JSON
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    let addr = v.get("address").and_then(|x| x.as_str()).unwrap();
    let path_out = v.get("path").and_then(|x| x.as_str()).unwrap();
    let file_out = v.get("file").and_then(|x| x.as_str()).unwrap();

    assert!(!addr.is_empty());
    assert_eq!(path_out, path);
    // Canonicalize both paths to handle platform differences (macOS uses /private/var/...)
    let file_out_path = std::path::Path::new(file_out);
    let file_out_canon = fs::canonicalize(file_out_path).unwrap();
    let ks_canon = fs::canonicalize(&ks).unwrap();
    assert_eq!(file_out_canon, ks_canon);

    // 读取 keystore 文件，校验地址一致
    let body = fs::read_to_string(&ks).unwrap();
    let kv: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(kv.get("address").and_then(|x| x.as_str()).unwrap(), addr);

    dir.close().ok();
}
