/// Wallet bridge adapter — calls into the ark-wallet crate for cold signing.
use anyhow::Result;

/// Sign the lock transaction hash via the cold wallet
pub fn sign_lock(hash: &[u8]) -> Result<Vec<u8>> {
    // Forward to the ark-wallet crate's public API
    let sig = ark_wallet::cold_sign(hash).map_err(|e| anyhow::anyhow!("cold_sign error"))?;
    Ok(sig)
}
