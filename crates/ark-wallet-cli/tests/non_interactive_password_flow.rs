use assert_cmd::Command;

// This test invokes the built CLI binary in a non-interactive fashion to ensure
// that when there's no TTY the code falls back to reading from stdin and
// doesn't deadlock. It uses assert_cmd to run the packaged binary.
#[test]
fn non_tty_password_stdin_flow() {
    // Build a minimal create command that reads password from stdin
    let mut cmd = Command::cargo_bin("ark-wallet").unwrap();
    cmd.arg("keystore")
        .arg("create")
        .arg("--mnemonic")
        .arg(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        )
        .arg("--password-stdin")
        .arg("--out")
        .arg("test_ks.json");

    // Ensure any previous artifact is removed so the test runs idempotently.
    let created = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_ks.json");
    if created.exists() {
        let _ = std::fs::remove_file(&created);
    }

    // Provide password on stdin and assert success
    cmd.write_stdin("TestPwd#1\n").assert().success();

    // Ensure the keystore file was created in the crate directory and clean up.
    assert!(created.exists(), "expected keystore file to be created: {:?}", created);
    let _ = std::fs::remove_file(created);
}
