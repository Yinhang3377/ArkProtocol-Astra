#![allow(dead_code)]
//! 统一错误（占位，可逐步替换 anyhow 文本）。
#[derive(thiserror::Error, Debug)]
pub enum SecurityError {
    #[error("invalid parameters: {0}")] InvalidParams(String),
    #[error("io error: {0}")] Io(#[from] std::io::Error),
    #[error("integrity check failed")]
    Integrity,
    #[error("randomness error: {0}")] Rand(String),
    #[error("crypto error: {0}")] Crypto(String),
    #[error("decode error: {0}")] Decode(String),
    #[error("parse error: {0}")] Parse(String),
    #[error("kdf error: {0}")] Kdf(String),
}
