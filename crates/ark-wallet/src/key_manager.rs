use secp256k1::{Message, Secp256k1, SecretKey};
use std::fs;
use zeroize::Zeroize;

/// Wallet error for cold signing
pub enum WalletError {
    InvalidKey,
    InvalidHash,
    IoError(std::io::Error),
}

impl From<std::io::Error> for WalletError {
    fn from(e: std::io::Error) -> Self {
        WalletError::IoError(e)
    }
}

/// Cold-sign a 32-byte transaction hash using a secret key read from disk.
pub fn cold_sign(tx_hash: &[u8]) -> Result<Vec<u8>, WalletError> {
    // Read the private key from a file called `cold.key` in the crate root.
    let mut key_bytes = fs::read("cold.key")?;

    // Construct a SecretKey from bytes
    let sk = SecretKey::from_slice(&key_bytes).map_err(|_| WalletError::InvalidKey)?;

    // Prepare the message
    let msg = Message::from_slice(tx_hash).map_err(|_| WalletError::InvalidHash)?;

    // Sign the message
    let secp = Secp256k1::signing_only();
    let sig = secp.sign_ecdsa(&msg, &sk);

    // Zeroize the key bytes buffer immediately
    key_bytes.zeroize();

    Ok(sig.serialize_der().to_vec())
}
