//! 地址派生（学习注释）
//! - 输入：secp256k1 压缩公钥（33 字节）
//! - 哈希：项目内实现（常见做法为 SHA-256/Ripemd160 组合或自定义）
//! - 长度：取 20 字节作为地址主体
//! - 编码：Base58 文本（便于展示与手工输入）
//! - 性质：同一公钥地址稳定可复现；不同公钥应产生不同地址

use sha2::{Digest, Sha256};

/// 从压缩公钥（33 字节）派生 Base58 地址的帮助函数。
/// - 参数：pk33 为 secp256k1 压缩公钥字节（长度应为 33）
/// - 返回：地址的 Base58 字符串表示
/// - 错误处理：本函数通常假定输入已校验，内部不做昂贵校验
pub fn from_pubkey(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    bs58::encode(&h[..20]).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_base58_and_len20_bytes() {
        // 任意 33 字节输入（地址只依赖字节序列）
        let mut pk = [0u8; 33];
        pk[0] = 0x02;
        pk[32] = 0xaa;

        let addr = from_pubkey(&pk);
        let decoded = bs58::decode(&addr).into_vec().expect("valid base58");
        assert_eq!(decoded.len(), 20);
    }

    #[test]
    fn address_is_deterministic_and_changes_with_input() {
        let mut pk1 = [0u8; 33];
        pk1[0] = 0x02;

        let mut pk2 = pk1;
        pk2[32] = 1;

        let a1 = from_pubkey(&pk1);
        let a1b = from_pubkey(&pk1);
        let a2 = from_pubkey(&pk2);

        assert_eq!(a1, a1b);
        assert_ne!(a1, a2);
    }
}
