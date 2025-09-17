# ArkProtocol-Astra

## AES-256-GCM Envelope Helpers

- Added `hot_prepare_envelope` and `hot_decrypt_envelope` for two-phase hot signing.
- Sensitive buffers (keys, nonces) are zeroized using the `zeroize` crate for memory safety.
- Usage: Prepare an envelope with `hot_prepare_envelope`, decrypt with `hot_decrypt_envelope`.
- Next: CLI subcommands for prepare/broadcast (planned).


Notes

This repository contains a CLI wallet (`crates/ark-wallet-cli`) used for demonstration and tooling. The AES-GCM helpers provide a secure envelope format for transferring signed payloads between offline (hot-sign) and online (broadcaster) devices.
