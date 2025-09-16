// Delegate signing to the dedicated `ark-wallet` crate (cold wallet implementation).
// This removes hot-signing from the CLI crate and forwards signatures to the cold signer.
// Re-export the cold signer API from the separate `ark-wallet` crate.
use wallet as ark_wallet;

#[allow(unused_imports)]
pub use ark_wallet::cold_sign;
#[allow(unused_imports)]
pub use ark_wallet::WalletError;
