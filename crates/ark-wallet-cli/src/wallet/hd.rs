use anyhow::Result;
use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use k256::ecdsa::SigningKey;

/// 从助记词与 BIP32 路径派生 32B 私钥与压缩公钥
pub fn derive_priv_from_mnemonic(
    lang: Language,
    mnemonic_text: &str,
    passphrase: &str,
    path: &str,
) -> Result<([u8; 32], [u8; 33], String)> {
    let m = Mnemonic::parse_in(lang, mnemonic_text)?;
    let seed = m.to_seed(passphrase);
    let dp: DerivationPath = path.parse()?;
    let xprv = XPrv::derive_from_path(seed, &dp)?;

    // private_key().to_bytes() -> GenericArray -> 拷贝到 [u8;32]
    let fb = xprv.private_key().to_bytes();
    let mut priv32 = [0u8; 32];
    priv32.copy_from_slice(&fb);

    // 压缩公钥 33B
    let pk = xprv.public_key().to_bytes();
    let mut pk33 = [0u8; 33];
    pk33.copy_from_slice(&pk);

    Ok((priv32, pk33, m.to_string()))
}

/// 由 32B 私钥计算压缩公钥
pub fn pubkey_from_privkey_secp256k1(priv32: &[u8; 32]) -> Result<[u8; 33]> {
    // SigningKey::from_bytes 需要 FieldBytes 引用（priv32.into()）
    let sk = SigningKey::from_bytes(priv32.into())?;
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
        let (priv32, pk33, _m) =
            derive_priv_from_mnemonic(Language::English, mn, "", path).unwrap();

        let pk_from_priv = pubkey_from_privkey_secp256k1(&priv32).unwrap();
        assert_eq!(pk33, pk_from_priv);

        // 地址可由公钥稳定生成
        let addr1 = address::from_pubkey(&pk33);
        let addr2 = address::from_pubkey(&pk_from_priv);
        assert_eq!(addr1, addr2);
    }
}
