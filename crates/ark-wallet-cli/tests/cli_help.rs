//! 测试：`ark-wallet --help` 基本帮助信息是否可用（学习注释）
//! - 目标：CLI 能打印帮助/子命令列表，退出码为 0
//! - 方法：使用 assert_cmd 调用二进制，断言 stdout/stderr 含关键字
//! - 注意：仅校验帮助文字存在与否，不校验完整文案

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
