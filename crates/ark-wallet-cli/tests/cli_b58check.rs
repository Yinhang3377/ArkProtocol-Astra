//! Tests for Base58Check address generation and import validation

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;

#[test]
fn create_and_import_with_b58check_roundtrip() {
    let dir = std::env::temp_dir().join(format!("ark_cli_b58_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let ks = dir.join("k_b58.json");

    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    // create with b58check
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
            "TestPass123!",
            "--kdf",
            "pbkdf2",
            "--iterations",
            "600000",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
            "--b58check",
        ])
        .assert()
        .success();

    // import with b58check should succeed
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "import",
            "--file",
            ks.to_str().unwrap(),
            "--password",
            "TestPass123!",
            "--b58check",
        ])
        .assert()
        .success();

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn import_rejects_tampered_b58check() {
    let dir = std::env::temp_dir().join(format!("ark_cli_b58_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let ks = dir.join("k_b58_t.json");

    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    // create with b58check
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
            "TestPass123!",
            "--kdf",
            "pbkdf2",
            "--iterations",
            "600000",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
            "--b58check",
        ])
        .assert()
        .success();

    // tamper with keystore address
    let mut raw = fs::read_to_string(&ks).unwrap();
    raw = raw.replace("\"address\": \"", "\"address\": \"X");
    fs::write(&ks, raw).unwrap();

    // import with b58check should fail
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "import",
            "--file",
            ks.to_str().unwrap(),
            "--password",
            "TestPass123!",
            "--b58check",
        ])
        .assert()
        .failure()
        .stderr(
            contains("keystore address is not valid Base58Check")
                .or(contains("checksum/payload mismatch")),
        );

    let _ = fs::remove_dir_all(dir);
}
