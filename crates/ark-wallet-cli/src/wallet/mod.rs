//! wallet 模块（学习注释）
//! - address：从压缩公钥（33 字节）派生 20 字节地址，并以 Base58 表示
//! - hd：助记词 -> 种子 -> BIP32 扩展私钥/公钥派生
//! - keystore：私钥加密存储（PBKDF2/Scrypt + AES-GCM），JSON 读写
//!
//! 设计说明：
//! - 保持与 CLI 解耦，纯逻辑、易测试；错误信息尽量清晰
//! - 对敏感数据（seed/privkey）使用 Zeroize/Zeroizing，降低泄露风险
//! - 尽量使用简单稳定的输入/输出类型（如 [u8; 32]、Vec<u8>、String）

pub mod address;
pub mod hd;
pub mod keystore;
