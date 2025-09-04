//! 统一错误（占位，可逐步替换 anyhow 文本）。
#[derive(thiserror::Error, Debug)]
pub enum SecurityError {
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("integrity check failed")]
    Integrity,
}
