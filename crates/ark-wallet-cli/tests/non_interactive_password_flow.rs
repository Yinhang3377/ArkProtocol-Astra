use std::env;
use std::process::Command;

// This test invokes the CLI binary in a non-interactive fashion to ensure
// that when there's no TTY the code falls back to reading from stdin and
// doesn't deadlock. It spawns the process and supplies the password via stdin.
#[test]
fn non_tty_password_stdin_flow() {
    // Build a minimal create command that reads password from stdin
    let mut cmd = Command::new(env::current_exe().unwrap());
    // The test harness runs the binary compiled for tests; arguments below
    // match the CLI shape for creating a keystore with --password-stdin
    cmd.arg("keystore")
        .arg("create")
        .arg("--mnemonic")
        .arg(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        )
        .arg("--password-stdin")
        .arg("--out")
        .arg("test_ks.json");
    // Provide password on stdin
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("spawn failed");
    use std::io::Write;
    let mut s = child.stdin.take().unwrap();
    writeln!(s, "TestPwd#1").unwrap();
    drop(s);
    let status = child.wait().expect("wait failed");
    // Process should exit success
    assert!(status.success());
    // Cleanup file if created
    let _ = std::fs::remove_file("test_ks.json");
}
