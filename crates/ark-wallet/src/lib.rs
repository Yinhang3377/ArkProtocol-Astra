pub mod key_manager;
pub mod mpc_split;
pub mod mpc_sign;

pub use key_manager::{ cold_sign, WalletError };
pub use mpc_split::split_key;
pub use mpc_sign::sign_from_shards;
