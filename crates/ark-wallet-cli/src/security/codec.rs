#![allow(dead_code)]
//! 编码与校验：Base58Check。
use crate::security::errors::SecurityError;
use sha2::{Digest, Sha256};

pub fn b58check_encode(version: u8, payload: &[u8]) -> String {
    let mut data = Vec::with_capacity(1 + payload.len() + 4);
    data.push(version);
    data.extend_from_slice(payload);
    let chk = Sha256::digest(Sha256::digest(&data));
    data.extend_from_slice(&chk[..4]);
    bs58::encode(data).into_string()
}

pub fn b58check_decode(s: &str) -> Result<(u8, Vec<u8>), SecurityError> {
    let raw = bs58::decode(s)
        .into_vec()
        .map_err(|e| SecurityError::Decode(format!("base58 decode error: {}", e)))?;
    if raw.len() < 5 {
        return Err(SecurityError::Integrity);
    }
    let (head, tail) = raw.split_at(raw.len() - 4);
    let (ver, payload) = (head[0], &head[1..]);
    let chk = Sha256::digest(Sha256::digest(head));
    if &chk[..4] != tail {
        return Err(SecurityError::Integrity);
    }
    Ok((ver, payload.to_vec()))
}
