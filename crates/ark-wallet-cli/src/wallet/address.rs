use sha2::{Digest, Sha256};

/// 简单占位地址规则：sha256(压缩公钥)[..20] -> base58
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
