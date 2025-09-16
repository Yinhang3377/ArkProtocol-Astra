use threshold_crypto::{ SecretKey, SecretKeyShare, SecretKeySet };
use std::fs;

/// Generate 3 shares from a 2-of-3 threshold scheme and write them out as files.
/// Input: 32-byte secret (private key) as raw bytes.
pub fn generate_shares(secret: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
    // For demo, interpret secret as seed to generate a SecretKey (not ideal cryptographic practice,
    // but sufficient for an example). threshold-crypto's SecretKey::from_bytes expects 32 bytes.
    let sk = SecretKey::from_bytes(
        secret.try_into().map_err(|_| anyhow::anyhow!("secret must be 32 bytes"))?
    )?;

    // Create a SecretKeySet with threshold = 1 (2-of-3)
    let sks = SecretKeySet::from(sk, 1);

    // Generate 3 shares
    let mut shares = Vec::new();
    for i in 0..3u64 {
        let share: SecretKeyShare = sks.secret_key_share(i as usize);
        shares.push(share.to_bytes().to_vec());
    }
    Ok(shares)
}

pub fn write_shares(path_prefix: &str, shares: &[Vec<u8>]) -> anyhow::Result<()> {
    for (i, s) in shares.iter().enumerate() {
        let fname = format!("{}{}.share", path_prefix, i + 1);
        fs::write(&fname, s)?;
    }
    Ok(())
}
