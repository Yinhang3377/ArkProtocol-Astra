/// Bridge module - exposes lock() which should involve signing at the wallet layer.
use anyhow::Result;

/// Lock an amount — this function will request the wallet to sign the lock transaction hash.
pub fn lock(amount: u64) -> Result<Vec<u8>> {
    // In the real system the lock() would construct a transaction and compute its hash.
    // For the demo, compute a fake 32-byte hash derived from amount.
    use sha2::{ Digest, Sha256 };
    let mut h = Sha256::new();
    h.update(&amount.to_le_bytes());
    let hash = h.finalize();

    // Delegate signing to the wallet module (which will call ark-wallet::cold_sign)
    crate::wallet::sign_lock(&hash)
}
