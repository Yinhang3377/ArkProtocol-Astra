#![allow(dead_code)]
//! KDF 选择与参数下限校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfKind {
    Scrypt,
    Pbkdf2,
}

pub fn validate_kdf_choice(kdf: &str) -> Result<KdfKind, String> {
    match kdf.to_lowercase().as_str() {
        "scrypt" => Ok(KdfKind::Scrypt),
        "pbkdf2" => Ok(KdfKind::Pbkdf2),
        other => Err(format!("invalid kdf: {other}. allowed: scrypt, pbkdf2")),
    }
}

pub fn validate_kdf_params(
    kind: KdfKind,
    iterations: u32,
    n: u32,
    r: u32,
    p: u32,
) -> Result<(), String> {
    match kind {
        KdfKind::Scrypt => {
            if n < 1 << 15 || r < 8 || p < 1 {
                return Err("scrypt params too weak (min n=32768, r=8, p=1)".into());
            }
        }
        KdfKind::Pbkdf2 => {
            if iterations < 50_000 {
                return Err("pbkdf2 iterations too low (min 50000)".into());
            }
        }
    }
    Ok(())
}
