use assert_cmd::prelude::*;
use predicates::str::contains;
use std::{fs, process::Command};

fn tmp_dir(name: &str) -> std::path::PathBuf {
    let mut d = std::env::temp_dir();
    d.push(format!("ark_cli_{}_{}", name, uuid::Uuid::new_v4()));
    d
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
    Command::cargo_bin("ark-wallet")
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
        .success()
        .stdout(contains("private key hex saved"));

    let hex = fs::read_to_string(&priv_path).unwrap();
    assert!(hex.trim().len() == 64); // 32B -> 64 hex
    let _ = fs::remove_dir_all(dir);
}
