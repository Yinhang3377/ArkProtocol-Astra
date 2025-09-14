//! 加密随机性测试
//! 目标：同一输入(助记词+路径+密码+KDF参数) 多次 keystore create
//!      生成的 keystore 中 salt / nonce / ciphertext 均应变化
//! 说明：
//! - 调用 CLI 两次 (--json 读取输出中的 "file" 字段获取真实写盘路径)。
//! - 解析 keystore JSON，提取 crypto.kdf.params.salt / crypto.cipher_params.nonce / crypto.ciphertext。
//! - 断言三者均不相同；若相同则随机性不足（潜在安全风险）。

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

const MN: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PASSWORD: &str = "TestPass123!"; // >=8，满足当前密码长度要求

fn bin() -> Command {
    Command::cargo_bin("ark-wallet").expect("binary built")
}

fn run_create(output_name: &str, kdf_args: &[&str]) -> (String, Value) {
    let td = tempdir().unwrap();
    let outfile = td.path().join(output_name);
    let mut cmd = bin();
    // 基本参数
    let mut args = vec![
        "keystore",
        "create",
        "--mnemonic",
        MN,
        "--lang",
        "en",
        "--password",
        PASSWORD,
        "--path",
        "m/44'/7777'/0'/0/0",
        "--out",
        outfile.to_str().unwrap(),
        "--overwrite",
        "--json",
    ];
    // 追加 KDF 参数
    args.extend_from_slice(kdf_args);
    cmd.args(args);

    let output = cmd.assert().success().get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&output).expect("json output");
    let file_path = v
        .get("file")
        .expect("file field")
        .as_str()
        .expect("string")
        .to_string();
    // 读取 keystore JSON 实体
    let raw = fs::read_to_string(&file_path).expect("read keystore");
    let ks: Value = serde_json::from_str(&raw).expect("parse keystore");
    (file_path, ks)
}

fn extract_crypto_fields(ks: &Value) -> (String, String, String) {
    let crypto = ks.get("crypto").expect("crypto");
    // Support both legacy shape: crypto.kdf.params and current shape: crypto.kdfparams
    let kdf_params = crypto
        .get("kdfparams")
        .or_else(|| crypto.get("kdf").and_then(|k| k.get("params")))
        .expect("kdf.params or kdfparams");
    let salt = kdf_params
        .get("salt")
        .or_else(|| kdf_params.get("salt_hex"))
        .expect("salt")
        .as_str()
        .expect("salt str")
        .to_string();
    // nonce may be stored directly as crypto.nonce (current) or under crypto.cipher_params.nonce (legacy)
    let nonce = crypto
        .get("nonce")
        .and_then(|n| n.as_str())
        .or_else(|| {
            crypto
                .get("cipher_params")
                .and_then(|cp| cp.get("nonce"))
                .or_else(|| crypto.get("cipher_params").and_then(|cp| cp.get("iv")))
                .and_then(|v| v.as_str())
        })
        .expect("nonce")
        .to_string();
    let ciphertext = crypto
        .get("ciphertext")
        .expect("ciphertext")
        .as_str()
        .expect("ciphertext str")
        .to_string();
    (salt, nonce, ciphertext)
}

#[test]
fn randomness_scrypt_multiple_creates_differ() {
    // scrypt 明确给出参数（使用通过校验的最小安全参数）
    let (_f1, ks1) = run_create(
        "scrypt_a.json",
        &["--kdf", "scrypt", "--n", "32768", "--r", "8", "--p", "1"],
    );
    let (_f2, ks2) = run_create(
        "scrypt_b.json",
        &["--kdf", "scrypt", "--n", "32768", "--r", "8", "--p", "1"],
    );

    let (salt1, nonce1, ct1) = extract_crypto_fields(&ks1);
    let (salt2, nonce2, ct2) = extract_crypto_fields(&ks2);

    assert_ne!(salt1, salt2, "salt should differ between runs");
    assert_ne!(nonce1, nonce2, "nonce should differ between runs");
    assert_ne!(ct1, ct2, "ciphertext should differ (random nonce/salt)");
}

#[test]
fn randomness_pbkdf2_multiple_creates_differ() {
    // pbkdf2 使用默认较大迭代（保持与 CLI 默认一致或显式指定）
    let (_f1, ks1) = run_create(
        "pbkdf2_a.json",
        &["--kdf", "pbkdf2", "--iterations", "600000"],
    );
    let (_f2, ks2) = run_create(
        "pbkdf2_b.json",
        &["--kdf", "pbkdf2", "--iterations", "600000"],
    );

    let (salt1, nonce1, ct1) = extract_crypto_fields(&ks1);
    let (salt2, nonce2, ct2) = extract_crypto_fields(&ks2);

    assert_ne!(salt1, salt2, "salt should differ between runs");
    assert_ne!(nonce1, nonce2, "nonce should differ between runs");
    assert_ne!(ct1, ct2, "ciphertext should differ");
}
