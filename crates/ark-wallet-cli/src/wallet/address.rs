#![ignore] // 暂不启用；已迁移为单元测试

//! 地址派生相关函数。
//!
//! 设计说明：
//! 1. 输入：secp256k1 压缩公钥（33 字节）。
//! 2. 基础地址（from_pubkey）：对压缩公钥做 Sha256，然后取前 20 字节，再 Base58 编码（无版本 / 无校验和）。
//!    - 仅用于当前项目内部已有逻辑（保持兼容，避免破坏现有测试）。
//! 3. Base58Check 地址（from_pubkey_b58check）：在 1 字节版本号 + 20 字节哈希后附加双 Sha256 前 4 字节校验和，再 Base58。
//!    - 便于发现输入错误，是更安全的对外交互格式（后续 CLI 可增加输出字段或选项）。
//! 4. 本文件不包含任何 JSON / CLI 输出逻辑；那些代码应放在 main.rs（或对应命令处理模块）中。
//!
//! 后续接入步骤（建议）：
//! - 在 keystore create / import / mnemonic import 的 JSON 输出旁增加 address_b58check 字段，调用 from_pubkey_b58check。
//! - 保留原有 address 字段以兼容已存在测试，再添加新的测试校验 address_b58check 是否存在。
//!
//! 注意：本文件只做纯函数与单元测试，避免引入 IO / 全局可变状态，便于审计。

use sha2::{ Digest, Sha256 };

/// 基础地址：对压缩公钥做 Sha256，取前 20 字节，Base58 编码（无版本 + 校验和）。
/// 兼容旧逻辑，不建议新场景对外单独使用（缺少校验和，容错差）。
pub fn from_pubkey(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    bs58::encode(&h[..20]).into_string()
}

/// Base58Check 地址版本号（可根据需要调整；保持常量以便未来做迁移策略）。
#[allow(dead_code)]
pub const ADDRESS_VERSION: u8 = 0x23;

/// 带版本与校验和的 Base58Check 地址：
/// layout = version(1B) || hash20(20B) || checksum(4B of double-Sha256) -> Base58
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
        // 构造一个固定“公钥”示例（真实情况应为合法的压缩公钥）
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

        // 成功解码
        let (ver, payload) = crate::security::b58check_decode(&addr_b58c).expect("valid b58check");
        assert_eq!(ver, ADDRESS_VERSION);
        assert_eq!(payload.len(), 20);

        // 校验 payload 内容
        let h = Sha256::digest(&pk);
        assert_eq!(&payload[..], &h[..20]);

        // 篡改最后一个字节 -> 校验失败
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
