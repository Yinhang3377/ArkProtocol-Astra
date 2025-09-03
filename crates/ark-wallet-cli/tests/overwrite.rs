use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

const MN: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn create_without_overwrite_should_fail() {
    let dir = tempdir().unwrap();
    let ks = dir.path().join("ks.json");
    let pwd = "PwD_123456";

    // 第一次创建（成功）
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "create",
            "--mnemonic",
            MN,
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

    // 第二次创建（不带 --overwrite，应失败）
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "create",
            "--mnemonic",
            MN,
            "--lang",
            "en",
            "--password-stdin",
            "--out",
            ks.to_str().unwrap(),
        ])
        .write_stdin(pwd)
        .assert()
        .failure()
        .stderr(contains("file exists"));

    // 带 --overwrite，应成功
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .args([
            "keystore",
            "create",
            "--mnemonic",
            MN,
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

    dir.close().ok();
}
