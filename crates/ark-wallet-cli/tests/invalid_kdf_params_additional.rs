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
        .arg("TestPwd#1");
    use std::path::PathBuf;
    let out_path: PathBuf = std::env::temp_dir().join("should_not_create.json");
    let out_str = out_path.to_string_lossy().to_string();
    cmd.arg("--out").arg(&out_str);
    let out = cmd.output().expect("failed to run");
    // Save child stdout/stderr for debugging in CI
    let _ = std::fs::create_dir_all("../../ci_artifacts");
    let _ = std::fs::write(
        "../../ci_artifacts/test_invalid_kdf_stdout.txt",
        &out.stdout,
    );
    let _ = std::fs::write(
        "../../ci_artifacts/test_invalid_kdf_stderr.txt",
        &out.stderr,
    );
    // Expect non-zero exit and that the output file was not created.
    assert!(!out.status.success());
    assert!(
        !out_path.exists(),
        "output file was created unexpectedly: {}",
        out_path.display()
    );
}
