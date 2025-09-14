//! Keystore 加密与格式（学习注释）
//! - 明文：32 字节私钥（k256）
//! - KDF：PBKDF2 或 Scrypt（参数：iterations / n, r, p）
//! - 对称加密：AES-GCM（附带随机 nonce 与认证标签）
//! - JSON 字段：address、path、pubkey_hex、crypto（含 kdf 与 cipher 参数）
//! - 版本：通过 VERSION 常量控制兼容性；不兼容版本应拒绝解析
//! - 安全：敏感数据（priv32、派生密钥）使用 Zeroize 及时清理
//!
//! API 概览：
//! - encrypt(priv32, password, kdf, ...) -> (Crypto, nonce)
//! - decrypt(keystore, password) -> priv32
use crate::security::errors::SecurityError;
use aes_gcm::{ aead::{ Aead, KeyInit }, Aes256Gcm, Nonce };
use base64::{ engine::general_purpose::STANDARD as B64, Engine as _ };
use pbkdf2::pbkdf2_hmac;
use scrypt as scrypt_crate;
use scrypt_crate::Params as ScryptParams;
use sha2::Sha256;
use std::result::Result as StdResult;
use zeroize::Zeroize;
use rand::rngs::OsRng;
use rand::RngCore;

pub const VERSION: u32 = 1;
pub const CIPHER: &str = "AES-256-GCM";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Keystore {
    pub version: u32,
    pub created_at: String,
    pub address: String,
    #[serde(default)]
    pub path: Option<String>,
    pub pubkey_hex: String,
    pub crypto: Crypto,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Crypto {
    pub cipher: String, // "AES-256-GCM"
    pub kdf: String, // "scrypt" | "pbkdf2"
    pub kdfparams: KdfParams,
    pub nonce: String, // base64(12B)
    pub ciphertext: String, // base64(密文+GCM标签)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KdfParams {
    pub salt: String, // base64
    pub dklen: u32, // 32
    // PBKDF2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    // scrypt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>, // N = 2^log_n
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<u32>,
}

pub fn b64e(bytes: &[u8]) -> String {
    B64.encode(bytes)
}
pub fn b64d(s: &str) -> StdResult<Vec<u8>, SecurityError> {
    B64.decode(s).map_err(|e| SecurityError::Decode(format!("base64 decode error: {}", e)))
}

/// 将 32 字节私钥加密为 keystore::Crypto。
/// - 参数：
///   - priv32：明文私钥
///   - password：用户口令
///   - kdf：字符串 "scrypt" 或 "pbkdf2"
///   - iterations/n/r/p：对应 KDF 参数
/// - 返回：(Crypto, nonce)
/// - 注意：内部会生成随机盐/IV，并使用 Zeroize 清理中间派生密钥
/// - 加密 32B 私钥 -> Crypto
pub fn encrypt(
    privkey: &[u8; 32],
    password: &str,
    kdf: &str,
    pbkdf2_iters: u32,
    n: u32,
    r: u32,
    p: u32
) -> StdResult<(Crypto, [u8; 12]), SecurityError> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    {
        let mut rng = OsRng;
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);
    }

    let kdfparams = if kdf == "pbkdf2" {
        KdfParams {
            salt: b64e(&salt),
            dklen: 32,
            iterations: Some(pbkdf2_iters),
            n: None,
            r: None,
            p: None,
        }
    } else {
        KdfParams {
            salt: b64e(&salt),
            dklen: 32,
            iterations: None,
            n: Some(n),
            r: Some(r),
            p: Some(p),
        }
    };

    let key_bytes = derive_key(password, kdf, &kdfparams)?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e|
        SecurityError::Crypto(format!("cipher init error: {}", e))
    )?;
    let nonce_ga = Nonce::from_slice(&nonce);
    let ct = cipher
        .encrypt(nonce_ga, privkey.as_slice())
        .map_err(|_| SecurityError::Crypto("aead encrypt failed".into()))?;

    let mut key_mut = key_bytes;
    key_mut.zeroize();

    let crypto = Crypto {
        cipher: CIPHER.to_string(),
        kdf: kdf.to_string(),
        kdfparams,
        nonce: b64e(&nonce),
        ciphertext: b64e(&ct),
    };
    Ok((crypto, nonce))
}

/// 使用口令解密 keystore::Crypto，返回 32 字节私钥。
/// - 口令错误或数据损坏时返回错误
/// - 解密 -> 32B 私钥
pub fn decrypt(crypto: &Crypto, password: &str) -> StdResult<[u8; 32], SecurityError> {
    if crypto.cipher != CIPHER {
        return Err(SecurityError::Crypto(format!("unsupported cipher: {}", crypto.cipher)));
    }
    let key = derive_key(password, &crypto.kdf, &crypto.kdfparams)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e|
        SecurityError::Crypto(format!("cipher init error: {}", e))
    )?;
    let nonce_bytes = b64d(&crypto.nonce)?;
    let ct = b64d(&crypto.ciphertext)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let pt = cipher
        .decrypt(nonce, ct.as_ref())
        .map_err(|_| SecurityError::Crypto("aead decrypt failed".into()))?;
    if pt.len() != 32 {
        return Err(SecurityError::Crypto("invalid plaintext length".into()));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&pt);

    let mut key_mut = key;
    key_mut.zeroize();
    Ok(out)
}

/// 小写十六进制
pub fn hex_lower(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        use core::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// KDF 派生 32B key
fn derive_key(password: &str, kdf: &str, params: &KdfParams) -> StdResult<[u8; 32], SecurityError> {
    let salt = b64d(&params.salt)?;
    let mut key = [0u8; 32];

    match kdf {
        "pbkdf2" => {
            let iters = params.iterations.unwrap_or(600_000);
            pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, iters, &mut key);
        }
        "scrypt" => {
            let n = params.n.unwrap_or(32_768);
            let r = params.r.unwrap_or(8);
            let p = params.p.unwrap_or(1);
            let log_n = (31 - n.leading_zeros()) as u8;
            let sp = ScryptParams::new(log_n, r, p, params.dklen as usize).map_err(|e|
                SecurityError::Kdf(format!("scrypt params error: {}", e))
            )?;
            scrypt_crate
                ::scrypt(password.as_bytes(), &salt, &sp, &mut key)
                .map_err(|e| SecurityError::Kdf(format!("scrypt error: {}", e)))?;
        }
        other => {
            return Err(SecurityError::Kdf(format!("unsupported kdf: {other}")));
        }
    }

    let mut salt_mut = salt;
    salt_mut.zeroize();
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_lower_works() {
        let bytes: [u8; 4] = [0x00, 0xab, 0x10, 0xff];
        let h = hex_lower(&bytes);
        assert_eq!(h, "00ab10ff");
    }

    #[test]
    fn encrypt_decrypt_roundtrip_pbkdf2() {
        let privkey = [7u8; 32];
        let pwd = "TestPwd#1";
        let (crypto, _nonce) = encrypt(&privkey, pwd, "pbkdf2", 1000, 0, 0, 0).unwrap();
        let out = decrypt(&crypto, pwd).unwrap();
        assert_eq!(out, privkey);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_scrypt() {
        let privkey = [9u8; 32];
        let pwd = "TestPwd#2";
        let (crypto, _nonce) = encrypt(&privkey, pwd, "scrypt", 0, 1 << 15, 8, 1).unwrap();
        let out = decrypt(&crypto, pwd).unwrap();
        assert_eq!(out, privkey);
    }

    #[test]
    fn decrypt_wrong_password_fails() {
        let privkey = [5u8; 32];
        let (crypto, _nonce) = encrypt(&privkey, "GoodPwd", "scrypt", 0, 1 << 15, 8, 1).unwrap();
        let err = decrypt(&crypto, "BadPwd").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("aead decrypt failed"));
    }

    #[test]
    fn wrong_cipher_rejected() {
        let privkey = [1u8; 32];
        let (mut crypto, _nonce) = encrypt(&privkey, "pwd", "pbkdf2", 10, 0, 0, 0).unwrap();
        crypto.cipher = "AES-128-GCM".to_string();
        let err = decrypt(&crypto, "pwd").unwrap_err();
        assert!(format!("{err}").contains("unsupported cipher"));
    }

    #[test]
    fn base64_helpers() {
        let d = [1u8, 2, 3, 4, 5, 6];
        let s = b64e(&d);
        let back = b64d(&s).unwrap();
        assert_eq!(back, d);
    }
}
