use assert_cmd::Command;
use serde_json::Value;
use std::fs;

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

    // 不带值的 --out-priv，期待默认写入 privkey.hex
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

    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    let file = v
        .get("file")
        .and_then(|x| x.as_str())
        .unwrap();
    let expected = dir.path().join("privkey.hex");
    assert_eq!(file, expected.to_str().unwrap());

    let printed = out.trim(); // 程序打印出来的路径字符串
    let printed_path = std::path::PathBuf::from(printed);
    let printed_abs = std::fs::canonicalize(&printed_path).unwrap_or(printed_path);
    let expected_abs = std::fs::canonicalize(&expected).unwrap_or(expected.clone());
    assert_eq!(printed_abs, expected_abs);

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
