# ArkProtocol-Astra

## AES-256-GCM Envelope Helpers

- Added `hot_prepare_envelope` and `hot_decrypt_envelope` for two-phase hot signing.
- Sensitive buffers (keys, nonces) are zeroized using the `zeroize` crate for memory safety.
- Usage: Prepare an envelope with `hot_prepare_envelope`, decrypt with `hot_decrypt_envelope`.
- Next: CLI subcommands for prepare/broadcast (planned).


Notes

This repository contains a CLI wallet (`crates/ark-wallet-cli`) used for demonstration and tooling. The AES-GCM helpers provide a secure envelope format for transferring signed payloads between offline (hot-sign) and online (broadcaster) devices.

### Usage example

Rust example (prepare -> decrypt):

```rust
use ark_wallet_cli::cli::{hot_prepare_envelope, hot_decrypt_envelope};
use zeroize::Zeroize;

// Prepare envelope on offline hot-sign device
let (envelope_json, ephemeral_key_b64) = hot_prepare_envelope(&tx, mnemonic)
	.expect("prepare failed");

// Transfer `envelope_json` to online broadcaster and deliver `ephemeral_key_b64` via a
// secure channel (QR, or one-click relay when configured).

// Decrypt on broadcaster (or for test)
let signed_tx = hot_decrypt_envelope(&envelope_json, &ephemeral_key_b64)
	.expect("decrypt failed");

// Zeroize ephemeral key after use
let mut ephemeral = ephemeral_key_b64;
ephemeral.zeroize();
```

### Tests & Security

- Unit test `test_hot_envelope_roundtrip` (in `crates/ark-wallet-cli`) verifies AES-GCM envelope round-trip.
- Sensitive data (derived private keys, ephemeral symmetric key and nonces, signed JSON) are zeroized in memory using the `zeroize` crate.

See implementation in `crates/ark-wallet-cli/src/cli.rs` for `hot_prepare_envelope`, `hot_decrypt_envelope` and associated tests.

### Example: envelope_demo

An example program is included at `examples/envelope_demo.rs`. It demonstrates preparing an AES-GCM envelope from a hot-sign mnemonic and decrypting it again (test/demo only).

Run the example with:

```bash
cargo run --example envelope_demo --package ark-wallet-cli
```

Note: the example uses a fixed test mnemonic and is intended for local testing only. Do not use this mnemonic in production.
