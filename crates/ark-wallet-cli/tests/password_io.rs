//! 测试：STDIN 口令输入（学习注释）
//! - 目标：--password-stdin 可创建并随后成功导入 keystore
//! - 方法：通过管道传入口令，分别执行 create 与 import 并断言成功

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;

fn tmp_dir(name: &str) -> std::path::PathBuf {
    let mut d = std::env::temp_dir();
    d.push(format!("ark_cli_pwd_{}_{}", name, uuid::Uuid::new_v4()));
    d
}

#[test]
fn create_and_import_with_password_stdin() {
    let dir = tmp_dir("stdin");
    fs::create_dir_all(&dir).unwrap();
    let ks_path = dir.join("ks.json");

    let mn =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let pwd = "StdInPwd_123";

    // create: 密码从 STDIN
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
            "--password-stdin",
            "--out",
            ks_path.to_str().unwrap(),
            "--overwrite",
        ])
        .write_stdin(pwd)
        .assert()
        .success()
        .stdout(contains("keystore saved"));

    // import: 密码从 STDIN
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "import",
            "--file",
            ks_path.to_str().unwrap(),
            "--password-stdin",
            "--full",
        ])
        .write_stdin(pwd)
        .assert()
        .success()
        .stdout(contains("\"match\": true"));

    let _ = fs::remove_dir_all(dir);
}
