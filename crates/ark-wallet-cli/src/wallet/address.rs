//! 地址派生函数。
//!
//! - from_pubkey: Sha256(pubkey) 前 20 字节 -> Base58（无版本/校验和，旧逻辑兼容）。
//! - from_pubkey_b58check: version(1B)+hash20(20B)+checksum(4B double-Sha256) -> Base58。
//!
//! 地址派生与 Base58Check 辅助函数。
use sha2::{ Digest, Sha256 };

/// Deprecated legacy helper: takes a compressed public key (33 bytes) and
/// returns a plain Base58-encoded 20-byte hash. Not recommended for new code.
#[allow(dead_code)]
fn _legacy_from_pubkey(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    bs58::encode(&h[..20]).into_string()
}

/// Base58Check version byte used by the wallet.
pub const ADDRESS_VERSION: u8 = 0x23;

/// Compute Base58Check(address) from a compressed public key (33 bytes).
pub fn from_pubkey_b58check(pk_compressed: &[u8]) -> String {
    let h = Sha256::digest(pk_compressed);
    crate::security::codec::b58check_encode(ADDRESS_VERSION, &h[..20])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_base58_and_len20_bytes() {
        let mut pk = [0u8; 33];
        pk[0] = 0x02;
        pk[32] = 0xaa;
        let addr = _legacy_from_pubkey(&pk);
        let decoded = bs58::decode(&addr).into_vec().expect("valid base58");
        assert_eq!(decoded.len(), 20);
    }

    #[test]
    fn address_is_deterministic_and_changes_with_input() {
        let mut pk1 = [0u8; 33];
        pk1[0] = 0x02;
        let mut pk2 = pk1;
        pk2[32] = 1;
        let a1 = _legacy_from_pubkey(&pk1);
        let a1b = _legacy_from_pubkey(&pk1);
        let a2 = _legacy_from_pubkey(&pk2);
        assert_eq!(a1, a1b);
        assert_ne!(a1, a2);
    }

    #[test]
    fn b58check_roundtrip_and_checksum() {
        let mut pk = [0u8; 33];
        pk[0] = 0x02;
        pk[32] = 0xab;
        let addr_b58c = from_pubkey_b58check(&pk);
        let (ver, payload) = crate::security::codec
            ::b58check_decode(&addr_b58c)
            .expect("valid b58check");
        assert_eq!(ver, ADDRESS_VERSION);
        assert_eq!(payload.len(), 20);
        let h = sha2::Sha256::digest(pk);
        assert_eq!(&payload[..], &h[..20]);

        // tamper -> checksum fails
        let mut bytes = addr_b58c.into_bytes();
        *bytes.last_mut().unwrap() ^= 0x01;
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(crate::security::codec::b58check_decode(&tampered).is_err());
    }
}
