/// Wallet bridge adapter — calls into the ark-wallet crate for cold signing.
use anyhow::Result;

/// Sign the lock transaction hash via the cold wallet
pub fn sign_lock(hash: &[u8]) -> Result<Vec<u8>> {
    // Forward to the ark-wallet crate's public API (lib name = "wallet")
    let sig = wallet::cold_sign(hash).map_err(|_| anyhow::anyhow!("cold_sign error"))?;
    Ok(sig)
}

// Notes:
// - This crate demonstrates calling `ark-wallet`'s cold_sign API from the bridge layer.
// - If you apply this change in a different repository, update the `ark-wallet` path
//   in Cargo.toml accordingly.
