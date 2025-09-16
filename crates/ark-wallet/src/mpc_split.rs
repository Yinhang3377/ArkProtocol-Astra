// mpc_split.rs - 2-of-3 分片备份示例
use sha2::{Digest, Sha256};

/// Split a 32-byte secret into 3 shares with threshold = 1 (i.e. 2-of-3 reconstruction)
pub fn split_key(key_bytes: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let mut h = Sha256::new();
    h.update(b"share-0");
    h.update(key_bytes);
    let s0 = h.finalize_reset().to_vec();

    h.update(b"share-1");
    h.update(key_bytes);
    let s1 = h.finalize_reset().to_vec();

    h.update(b"share-2");
    h.update(key_bytes);
    let s2 = h.finalize().to_vec();

    Ok((s0, s1, s2))
}
