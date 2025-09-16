// mpc_sign.rs - 用 2 份分片离线签名（示例）
use anyhow::Result;
use secp256k1::{ Message, Secp256k1, SecretKey };
use sha2::{ Sha256, Digest };

/// 示例：从两份 share 重建（示例性）并产生签名（注：threshold-crypto 与 secp256k1 不直接互通，
/// 下面是示范性质的伪实现，应在生产中用配套的门限签名库）
pub fn sign_from_shards(shard1: &[u8], shard2: &[u8], msg: &[u8]) -> Result<Vec<u8>> {
    // Interpret shard bytes as the inner representation from SecretKeyShare
    // threshold_crypto::SecretKeyShare doesn't expose a from_bytes API; in this demo we treat
    // the input slices as raw bytes and hash them to derive a 32-byte seed for secp256k1.
    let mut hasher = Sha256::new();
    hasher.update(shard1);
    hasher.update(shard2);
    let key32 = hasher.finalize();

    let sk = SecretKey::from_slice(&key32).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let secp = Secp256k1::signing_only();
    let m = Message::from_slice(&msg[0..(32).min(msg.len())]).map_err(|e|
        anyhow::anyhow!(e.to_string())
    )?;
    let sig = secp.sign_ecdsa(&m, &sk);
    Ok(sig.serialize_der().to_vec())
}
