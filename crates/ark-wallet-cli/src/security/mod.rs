//! 安全能力集中出口：文件原子写入/权限、编码校验、KDF 校验、统一错误。
pub mod codec;
pub mod errors;
pub mod fs;
pub mod kdf;

pub use codec::{b58check_decode, b58check_encode};
pub use fs::secure_atomic_write;
pub use kdf::{validate_kdf_choice, validate_kdf_params, KdfKind};
