use assert_cmd::prelude::*;
use predicates::str::contains;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

fn tmp_dir(name: &str) -> std::path::PathBuf {
    let mut d = std::env::temp_dir();
    d.push(format!("ark_cli_{}_{}", name, uuid::Uuid::new_v4()));
    d
}

fn stdout_path(stdout: &str) -> PathBuf {
    let t = stdout.trim();
    if t.starts_with('{') {
        let v: Value = serde_json::from_str(t).expect("json");
        PathBuf::from(v.get("file").and_then(|x| x.as_str()).expect("file field"))
    } else {
        let p = t.strip_prefix("private key hex saved: ").unwrap_or(t);
        PathBuf::from(p)
    }
}

#[test]
fn keystore_roundtrip_pbkdf2() {
    roundtrip("pbkdf2");
}

#[test]
fn keystore_roundtrip_scrypt() {
    roundtrip("scrypt");
}

fn roundtrip(kdf: &str) {
    let dir = tmp_dir(kdf);
    fs::create_dir_all(&dir).unwrap();
    let ks_path = dir.join("keystore.json");

    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let pwd = "StrongPwd_123!";

    // create
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "create",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--path",
            "m/44'/7777'/0'/0/0",
            "--password",
            pwd,
            "--kdf",
            kdf,
            "--iterations",
            "600000",
            "--n",
            "32768",
            "--r",
            "8",
            "--p",
            "1",
            "--out",
            ks_path.to_str().unwrap(),
            "--overwrite",
        ])
        .assert()
        .success()
        .stdout(contains("keystore saved"));

    // import (full 输出校验匹配)
    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "import",
            "--file",
            ks_path.to_str().unwrap(),
            "--password",
            pwd,
            "--full",
        ])
        .assert()
        .success();

    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        out.contains("\"match\": true"),
        "import should match pubkey/address"
    );

    // export 私钥 hex
    let priv_path = dir.join("priv.hex");
    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "export",
            "--file",
            ks_path.to_str().unwrap(),
            "--password",
            pwd,
            "--out-priv",
            priv_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let printed = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let printed_abs =
        std::fs::canonicalize(stdout_path(&printed)).unwrap_or_else(|_| stdout_path(&printed));
    let expected_abs = std::fs::canonicalize(&priv_path).unwrap_or(priv_path.clone());
    assert_eq!(printed_abs, expected_abs);

    let hex = fs::read_to_string(&priv_path).unwrap();
    assert!(hex.trim().len() == 64); // 32B -> 64 hex
    let _ = fs::remove_dir_all(dir);
}
