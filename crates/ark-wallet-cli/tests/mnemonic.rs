use assert_cmd::prelude::*;
use predicates::str::{contains, is_match};
use std::process::Command;

#[test]
fn mnemonic_new_json_en_12() {
    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args(["--json", "mnemonic-new", "--lang", "en", "--words", "12"])
        .assert()
        .success();

    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    let m = v.get("mnemonic").and_then(|x| x.as_str()).unwrap();
    assert!(m.split_whitespace().count() == 12);
}

#[test]
fn mnemonic_import_minimal_and_full() {
    // 用固定助记词（英文词表合法）
    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // 最小输出：只包含 address/path
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "mnemonic-import",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--path",
            "m/44'/7777'/0'/0/0",
        ])
        .assert()
        .success()
        .stdout(contains("Address:"));

    // full 输出：包含多字段
    let assert = Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "mnemonic-import",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--path",
            "m/44'/7777'/0'/0/0",
            "--full",
        ])
        .assert()
        .success();

    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // 宽松检查几个关键字段
    assert!(out.contains("\"address\""));
    assert!(out.contains("\"xpub\""));
    assert!(out.contains("\"xprv\""));
    let re = is_match("\"pubkey_hex\"\\s*:\\s*\"[0-9a-f]{66}\"").unwrap();
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "mnemonic-import",
            "--mnemonic",
            mn,
            "--lang",
            "en",
            "--path",
            "m/44'/7777'/0'/0/0",
            "--full",
        ])
        .assert()
        .success()
        .stdout(re);
}
