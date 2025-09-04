//! 测试：KDF 与口令确认的错误分支（学习注释）
//! - password_confirm_requires_prompt：仅当使用 --password-prompt 时允许 --password-confirm
//! - create_with_invalid_kdf_should_fail_fast：无效 KDF 值应尽早失败（不进入加密流程）
//! - 方法：断言进程退出失败且包含预期错误消息

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn create_with_invalid_kdf_should_fail_fast() {
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
            "--password-stdin",
            "--kdf",
            "argon2",
            "--overwrite",
        ])
        .write_stdin("Pwd#123456")
        .assert()
        .failure()
        .stderr(contains("invalid kdf"));
}

#[test]
fn password_confirm_requires_prompt() {
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
            "--password-stdin",
            "--password-confirm",
            "--overwrite",
        ])
        .write_stdin("Pwd#123456")
        .assert()
        .failure()
        .stderr(contains("requires --password-prompt"));
}
