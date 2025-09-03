use assert_cmd::prelude::*;
use predicates::str::contains;
use std::{fs, process::Command};

#[test]
fn mnemonic_import_requires_input() {
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args(["mnemonic-import", "--lang", "en"])
        .assert()
        .failure()
        .stderr(contains("either --mnemonic or --mnemonic-file is required"));
}

#[test]
fn import_wrong_password_should_fail() {
    // 先创建
    let dir = std::env::temp_dir().join(format!("ark_cli_err_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let ks = dir.join("k.json");

    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
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
            "GoodPwd_123!",
            "--kdf",
            "pbkdf2",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
        ])
        .assert()
        .success();

    // 用错口令导入，应失败
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "import",
            "--file",
            ks.to_str().unwrap(),
            "--password",
            "WrongPwd",
            "--full",
        ])
        .assert()
        .failure();

    let _ = fs::remove_dir_all(dir);
}
