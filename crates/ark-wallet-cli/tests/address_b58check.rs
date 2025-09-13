//! Base58Check 地址校验测试
use k256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::{Digest, Sha256};

#[test]
fn base58check_roundtrip_and_checksum_fail() {
    // 构造一个确定性“公钥”（这里直接用 33 字节假数据或真实派生都可）
    // 为简单起见，用全零+前缀 0x02（非真实密钥，但我们只测地址函数行为）
    let mut pk = [0u8; 33];
    pk[0] = 0x02;
    pk[32] = 0xaa;

    let addr = ark_wallet_cli::wallet::address::from_pubkey_b58check(&pk);
    // 解码（使用 security 模块）
    let (ver, payload) = ark_wallet_cli::security::b58check_decode(&addr).expect("valid");
    assert_eq!(ver, ark_wallet_cli::wallet::address::ADDRESS_VERSION);
    assert_eq!(payload.len(), 20);
    let h = Sha256::digest(&pk);
    assert_eq!(&payload[..], &h[..20]);

    // 破坏最后一个字符（仍可能是合法 base58，但 checksum 应失败）
    let mut bytes = addr.into_bytes();
    *bytes.last_mut().unwrap() ^= 0x01;
    let tampered = String::from_utf8(bytes).unwrap();
    assert!(ark_wallet_cli::security::b58check_decode(&tampered).is_none());
}
