//! 测试：KDF 参数下限校验（安全加固）
//! - scrypt: n < 32768 或 r < 8 或 p < 1 应失败
//! - pbkdf2: iterations < 50000 应失败
use assert_cmd::Command;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("ark-wallet").unwrap()
}

const MN: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn scrypt_params_too_weak_should_fail() {
    let td = tempdir().unwrap();
    let out = td.path().join("weak_scrypt.json");
    let mut cmd = bin();
    cmd.args([
        "keystore",
        "create",
        "--mnemonic",
        MN,
        "--lang",
        "en",
        "--password",
        "TestPass123!",
        "--kdf",
        "scrypt",
        "--n",
        "1024", // 弱 n
        "--r",
        "1", // 弱 r
        "--p",
        "1",
        "--out",
        out.to_str().unwrap(),
        "--overwrite",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("scrypt params too weak"));
}

#[test]
fn pbkdf2_iterations_too_low_should_fail() {
    let td = tempdir().unwrap();
    let out = td.path().join("weak_pbkdf2.json");
    let mut cmd = bin();
    cmd.args([
        "keystore",
        "create",
        "--mnemonic",
        MN,
        "--lang",
        "en",
        "--password",
        "TestPass123!",
        "--kdf",
        "pbkdf2",
        "--iterations",
        "10000", // 弱迭代
        "--out",
        out.to_str().unwrap(),
        "--overwrite",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("pbkdf2 iterations too low"));
}
