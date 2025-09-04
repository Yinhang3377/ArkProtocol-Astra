#![allow(dead_code)]
//! 编码与校验：Base58Check。
use sha2::{Digest, Sha256};

pub fn b58check_encode(version: u8, payload: &[u8]) -> String {
    let mut data = Vec::with_capacity(1 + payload.len() + 4);
    data.push(version);
    data.extend_from_slice(payload);
    let chk = Sha256::digest(Sha256::digest(&data));
    data.extend_from_slice(&chk[..4]);
    bs58::encode(data).into_string()
}

pub fn b58check_decode(s: &str) -> Option<(u8, Vec<u8>)> {
    let raw = bs58::decode(s).into_vec().ok()?;
    if raw.len() < 5 {
        return None;
    }
    let (head, tail) = raw.split_at(raw.len() - 4);
    let (ver, payload) = (head[0], &head[1..]);
    let chk = Sha256::digest(Sha256::digest(head));
    if &chk[..4] != tail {
        return None;
    }
    Some((ver, payload.to_vec()))
}
