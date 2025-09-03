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
