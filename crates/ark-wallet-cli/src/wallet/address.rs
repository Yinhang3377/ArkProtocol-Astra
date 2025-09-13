#![ignore] // 暂不启用；已迁移为单元测试

//! 地址派生函数。
//!
//! 说明：
//! - from_pubkey: 兼容旧逻辑（Sha256(pubkey) 前20字节 -> Base58，无版本/校验和）。
//! - from_pubkey_b58check: 版本(1B)+hash20(20B)+checksum(4B double-Sha256) -> Base58（推荐未来外部展示）。
//! - 这里只放纯函数与单元测试；不要放 CLI JSON 构造代码。

use sha2::{ Digest, Sha256 };

/// 简单 Base58 地址（无版本/校验和）
pub fn from_pubkey(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    bs58::encode(&h[..20]).into_string()
}

/// Base58Check 版本号
#[allow(dead_code)]
pub const ADDRESS_VERSION: u8 = 0x23;

/// 带版本与校验和的 Base58Check 地址
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

    #[test]
    fn b58check_roundtrip_and_checksum() {
        let mut pk = [0u8; 33];
        pk[0] = 0x02;
        pk[32] = 0xab;

        let addr_b58c = from_pubkey_b58check(&pk);
        let (ver, payload) = crate::security::b58check_decode(&addr_b58c).expect("valid b58check");
        assert_eq!(ver, ADDRESS_VERSION);
        assert_eq!(payload.len(), 20);

        let h = sha2::Sha256::digest(&pk);
        assert_eq!(&payload[..], &h[..20]);

        // 篡改 -> checksum 失败
        let mut bytes = addr_b58c.into_bytes();
        *bytes.last_mut().unwrap() ^= 0x01;
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(crate::security::b58check_decode(&tampered).is_none());
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
