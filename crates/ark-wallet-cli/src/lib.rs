// Library crate for ark-wallet-cli: re-export internal modules so examples
// and integration tests can access helpers like `cli::hot_prepare_envelope`.

pub mod cli;
pub mod security;
pub mod wallet;
