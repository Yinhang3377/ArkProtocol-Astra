#![allow(dead_code)]
//! KDF 选择与参数下限校验。
use crate::security::errors::SecurityError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfKind {
    Scrypt,
    Pbkdf2,
}

pub fn validate_kdf_choice(kdf: &str) -> Result<KdfKind, SecurityError> {
    match kdf.to_lowercase().as_str() {
        "scrypt" => Ok(KdfKind::Scrypt),
        "pbkdf2" => Ok(KdfKind::Pbkdf2),
        other => Err(SecurityError::InvalidParams(format!(
            "invalid kdf: {other}. allowed: scrypt, pbkdf2"
        ))),
    }
}

pub fn validate_kdf_params(
    kind: KdfKind,
    iterations: u32,
    n: u32,
    r: u32,
    p: u32,
) -> Result<(), SecurityError> {
    match kind {
        KdfKind::Scrypt => {
            if n < 1 << 15 || r < 8 || p < 1 {
                return Err(SecurityError::InvalidParams(
                    "scrypt params too weak (min n=32768, r=8, p=1)".into(),
                ));
            }
        }
        KdfKind::Pbkdf2 => {
            if iterations < 50_000 {
                return Err(SecurityError::InvalidParams(
                    "pbkdf2 iterations too low (min 50000)".into(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_choice_accepts_known() {
        assert_eq!(validate_kdf_choice("scrypt").unwrap(), KdfKind::Scrypt);
        assert_eq!(validate_kdf_choice("pbkdf2").unwrap(), KdfKind::Pbkdf2);
    }

    #[test]
    fn validate_choice_rejects_unknown() {
        let e = validate_kdf_choice("weakkdf").unwrap_err();
        match e {
            SecurityError::InvalidParams(_) => (),
            _ => panic!("expected InvalidParams"),
        }
    }

    #[test]
    fn validate_params_rejects_weak() {
        assert!(validate_kdf_params(KdfKind::Pbkdf2, 10, 0, 0, 0).is_err());
        assert!(validate_kdf_params(KdfKind::Scrypt, 0, 1 << 14, 4, 0).is_err());
    }
}
