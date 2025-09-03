use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

#[test]
fn shows_help() {
    Command::cargo_bin("ark-wallet")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Ark Wallet CLI"));
}
