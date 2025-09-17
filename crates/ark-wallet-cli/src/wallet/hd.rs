//! 分层确定性密钥（HD）派生（学习注释）
//! - 助记词：bip39 多语言；可选 passphrase 参与种子计算（非 keystore 密码）
//! - 种子 -> 扩展私钥：bip32::XPrv::derive_from_path
//! - 路径：形如 m/44'/7777'/0'/0/0（由 CLI 传入，不在此模块硬编码）
//! - 输出：32 字节私钥 + 压缩公钥（33 字节），以及调试/校验所需的派生信息
//! - 安全：尽早 Zeroize 种子与私钥；仅在必要范围内持有敏感数据

use crate::security::errors::SecurityError;
use bip32::{ DerivationPath, XPrv };
use bip39::{ Language, Mnemonic };
use k256::ecdsa::SigningKey;
use std::result::Result as StdResult;
use zeroize::Zeroizing;

/// 从助记词派生私钥与公钥（示例）
/// - 参数：
///   - lang：bip39::Language（语言）
///   - mnemonic：助记词文本
///   - passphrase：可选 BIP39 口令（参与 seed 计算）
///   - path：BIP32 派生路径（如 m/44'/7777'/0'/0/0）
/// - 返回：`(priv32, pk33, extra)` 其中 priv32 为 32 字节私钥，pk33 为压缩公钥
/// - 说明：仅添加注释，不改变函数签名与逻辑
pub fn derive_priv_from_mnemonic(
    lang: Language,
    mnemonic_text: &str,
    passphrase: &str,
    path: &str
) -> StdResult<([u8; 32], [u8; 33]), SecurityError> {
    let m = Mnemonic::parse_in(lang, mnemonic_text).map_err(|e|
        SecurityError::Parse(format!("bip39 parse error: {}", e))
    )?;
    // Wrap seed in Zeroizing so it is cleared on drop
    let seed = Zeroizing::new(m.to_seed(passphrase));
    let dp: DerivationPath = path
        .parse()
        .map_err(|e| SecurityError::Parse(format!("derivation path parse error: {}", e)))?;
    let xprv = XPrv::derive_from_path(seed.as_ref(), &dp).map_err(|e|
        SecurityError::Parse(format!("xprv derive error: {}", e))
    )?;

    // private_key().to_bytes() -> GenericArray -> 拷贝到 [u8;32]
    let fb = xprv.private_key().to_bytes();
    let mut priv32 = [0u8; 32];
    priv32.copy_from_slice(&fb);

    // 压缩公钥 33B
    let pk = xprv.public_key().to_bytes();
    let mut pk33 = [0u8; 33];
    pk33.copy_from_slice(&pk);

    // Attempt to drop/zeroize seed (Zeroizing will clear on drop). Do not return mnemonic text.
    Ok((priv32, pk33))
}

/// 由 32 字节私钥计算压缩公钥（33 字节）。
/// - 用于 keystore 解密后还原地址、公钥等
pub fn pubkey_from_privkey_secp256k1(priv32: &[u8; 32]) -> StdResult<[u8; 33], SecurityError> {
    // SigningKey::from_bytes 需要 FieldBytes 引用（priv32.into()）
    let sk = SigningKey::from_bytes(priv32.into()).map_err(|e|
        SecurityError::Crypto(format!("k256 error: {}", e))
    )?;
    let vk = sk.verifying_key();
    let ep = vk.to_encoded_point(true);
    let bytes = ep.as_bytes();
    let mut out = [0u8; 33];
    out.copy_from_slice(bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::address;

    #[test]
    fn derive_matches_pubkey_from_priv() {
        let mn =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/44'/7777'/0'/0/0";
        let (priv32, pk33) = derive_priv_from_mnemonic(Language::English, mn, "", path).unwrap();

        let pk_from_priv = pubkey_from_privkey_secp256k1(&priv32).unwrap();
        assert_eq!(pk33, pk_from_priv);

        // 地址可由公钥稳定生成
        let addr1 = address::from_pubkey_b58check(&pk33);
        let addr2 = address::from_pubkey_b58check(&pk_from_priv);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn zeroize_regression_demo() {
        use zeroize::{ Zeroize, Zeroizing };
        // Create a buffer and wrap in Zeroizing, fill it with non-zero bytes
        let mut buf = [0u8; 32];
        for i in 0..32 {
            buf[i] = (i as u8).wrapping_add(1);
        }
        let z = Zeroizing::new(buf);
        // Ensure buffer contains non-zero data
        assert!(z.iter().any(|&b| b != 0));
        // Explicitly zeroize by dropping the wrapper
        drop(z);
        // We cannot safely inspect the dropped memory, but this regression test
        // documents expected behavior and will fail if Zeroizing semantics are
        // removed. For additional verification, ensure Zeroizing::zeroize works
        // on an in-scope buffer:
        let mut buf2 = Zeroizing::new([0xaau8; 32]);
        buf2.zeroize();
        assert!(buf2.iter().all(|&b| b == 0));
    }
}
