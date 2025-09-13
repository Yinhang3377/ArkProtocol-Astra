//! 地址派生（学习注释）
//! - 输入：secp256k1 压缩公钥（33 字节）
//! - 哈希：项目内实现（常见做法为 SHA-256/Ripemd160 组合或自定义）
//! - 长度：取 20 字节作为地址主体
//! - 编码：Base58 文本（便于展示与手工输入）
//! - 性质：同一公钥地址稳定可复现；不同公钥应产生不同地址

use sha2::{ Digest, Sha256 };

/// 从压缩公钥（33 字节）派生 Base58 地址的帮助函数。
/// - 参数：pk33 为 secp256k1 压缩公钥字节（长度应为 33）
/// - 返回：地址的 Base58 字符串表示
/// - 错误处理：本函数通常假定输入已校验，内部不做昂贵校验
pub fn from_pubkey(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    bs58::encode(&h[..20]).into_string()
}

/// 推荐：带版本与校验和的 Base58Check 地址
#[allow(dead_code)]
pub const ADDRESS_VERSION: u8 = 0x23;

#[allow(dead_code)]
pub fn from_pubkey_b58check(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    crate::security::b58check_encode(ADDRESS_VERSION, &h[..20])
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

// ...existing code inside Cmd::MnemonicImport branch where out JSON is built...
let address_b58check = wallet::address::from_pubkey_b58check(&pk);
let out = if full {
    serde_json::json!({
        "address": address,
        "address_b58check": address_b58check,
        "path": path,
        "xpub": xpub_str,
        "xprv": xprv_str.as_str(),
        "pubkey_hex": pub_hex,
        "privkey_hex": priv_hex
    })
} else {
    serde_json::json!({
        "address": address,
        "address_b58check": address_b58check,
        "path": path
    })
};
// ...existing code after constructing ks & out_abs...
let address_b58check = wallet::address::from_pubkey_b58check(&pk33);
if cli.json {
    let out_json =
        serde_json::json!({
        "address": ks.address,
        "address_b58check": address_b58check,
        "path": path_str,
        "file": out_abs.to_string_lossy()
    });
    println!("{}", serde_json::to_string_pretty(&out_json)?);
} else {
    println!("keystore saved: {}", out_abs.display());
}

let address_b58check = wallet::address::from_pubkey_b58check(&pk33);
let out =
    serde_json::json!({
    "address": address,
    "address_b58check": address_b58check,
    "path": ks.path,
    "pubkey_hex": wallet::keystore::hex_lower(&pk33),
    "keystore_pubkey_hex": ks.pubkey_hex,
    "match": ks.address == address && ks.pubkey_hex == wallet::keystore::hex_lower(&pk33),
});
