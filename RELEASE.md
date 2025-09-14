# Release v0.4.2 — Wallet CLI security and release snapshot

This release finalizes the security roadmap for the wallet CLI and prepares a tagged snapshot for distribution.

## Highlights

- Unified security error handling (`SecurityError`) across wallet CLI modules.
- Zeroized password handling for memory safety (passwords stored in `Zeroizing<String>` and zeroed after use).
- Enforced Base58Check addresses for stored/printed addresses to avoid legacy weak formats.
- Implemented `secure_atomic_write` for safe, atomic file writes (temp file in same dir, fsync, rename, permissions tightened on Unix).
- Replaced direct `getrandom` usage with OS RNG (`rand::rngs::OsRng`) and added secure random suffix for temp files.
- Validated KDF choices and parameters early to avoid weak configurations.

## Testing & CI

- Local: `cargo fmt`, `cargo clippy`, `cargo test` (ark-wallet-cli) — all passing.
- CI: GitHub Actions matrix (msrv, macOS, Ubuntu, Windows) passes for the pushed commit.

## Notes for reviewers

- Focus review on `crates/ark-wallet-cli/src/security` and `crates/ark-wallet-cli/src/wallet` changes (errors, fs, kdf, keystore, address, main).
- The tag `v0.4.2` will be created for this snapshot.

## Changelog (summary)

- Security: unify SecurityError and map top-level errors to deterministic exit codes.
- Security: zeroize password handling across CLI flows.
- Security: Base58Check output and import-time compatibility helpers.
- Security: atomic file writing and tightened file permissions on Unix.
- Misc: add `rand = "0.8"` and `hex = "0.4"` dependencies for secure temp suffix.

