use assert_cmd::Command;
use tempfile::tempdir;

const MN: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn create_with_password_confirm_mismatch_should_fail() {
    let dir = tempdir().unwrap();
    let ks = dir.path().join("ks.json");

    // 第一次输入与确认不一致
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "keystore",
            "create",
            "--mnemonic",
            MN,
            "--lang",
            "en",
            "--password-prompt",
            "--password-confirm",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
        ])
        .write_stdin("StrongPwd_123!\nStrongPwd_1234!\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("passwords do not match"));
}

#[test]
fn create_with_password_confirm_ok_should_succeed() {
    let dir = tempdir().unwrap();
    let ks = dir.path().join("ks.json");

    // 两次输入一致
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "keystore",
            "create",
            "--mnemonic",
            MN,
            "--lang",
            "en",
            "--password-prompt",
            "--password-confirm",
            "--out",
            ks.to_str().unwrap(),
            "--overwrite",
        ])
        .write_stdin("StrongPwd_123!\nStrongPwd_123!\n")
        .assert()
        .success();

    assert!(ks.exists());
    dir.close().ok();
}
