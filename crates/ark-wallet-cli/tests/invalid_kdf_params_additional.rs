use std::env;
use std::process::Command;

// Run CLI create with intentionally weak scrypt params and expect a failure
#[test]
fn create_with_weak_scrypt_params_should_fail() {
    let mut cmd = Command::new(env::current_exe().unwrap());
    cmd.arg("keystore")
        .arg("create")
        .arg("--mnemonic")
        .arg(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        );
    // Provide weak scrypt params: n too small
    cmd.arg("--kdf")
        .arg("scrypt")
        .arg("--n")
        .arg("16384")
        .arg("--r")
        .arg("4")
        .arg("--p")
        .arg("1")
        .arg("--password")
        .arg("TestPwd#1")
        .arg("--out")
        .arg("/tmp/should_not_create.json");
    let out = cmd.output().expect("failed to run");
    // Expect non-zero exit status and stderr contains reason
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(
        stderr.contains("scrypt params too weak")
            || stderr.contains("invalid kdf")
            || stderr.contains("too weak")
    );
}
