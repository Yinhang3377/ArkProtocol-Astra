# Wallet Snapshot Release: v0.1.0-wallet

This release publishes the wallet snapshot including CLI, keystore, and security hardening.

## Highlights

- CLI: `ark-wallet` with subcommands for `mnemonic` and `keystore` (create/import/export).
- Keystore: AES-256-GCM encryption, supports PBKDF2 and scrypt KDFs.
- Security:
  - Unified `SecurityError` boundary and deterministic exit codes.
  - Password handling uses `zeroize::Zeroizing<String>` and explicitly zeroizes sensitive memory.
  - Addresses use Base58Check by default for storage and output.
  - `secure_atomic_write` for durable atomic file writes (fsync+rename+dir-fsync).
- Engineering:
  - Replaced direct `getrandom` calls with `rand::rngs::OsRng` for compatibility.
  - Comprehensive unit and integration tests for wallet functionality.
  - CI format/lint fixes applied and workflow validated across platforms.

## Basic usage examples

- Generate a 12-word mnemonic (English):

  ```bash
  ark-wallet mnemonic new --lang en --words 12
  ```

- Create a keystore from mnemonic (prompt for password):

  ```bash
  ark-wallet keystore create --mnemonic "..." --password-prompt --password-confirm
  ```

- Export private key hex to file (JSON output):

  ```bash
  ark-wallet keystore export --file keystore.json --password-stdin --out-priv priv.hex --json
  ```

## Notes for maintainers

- Tests: `cargo test -p ark-wallet-cli`.
- Format: `cargo fmt --all` (CI enforces formatting).
- Tagging: this file is created and the release tag `v0.1.0-wallet` will be pushed to remote.

