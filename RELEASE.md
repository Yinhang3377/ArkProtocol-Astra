# Release v0.4.2 — Wallet CLI security and release snapshot

This release finalizes the security roadmap for the wallet CLI and prepares a tagged snapshot for distribution.

## Highlights

- CLI: `ark-wallet` with subcommands for `mnemonic` and `keystore` (create/import/export).
- Keystore: AES-256-GCM encryption, supports PBKDF2 and scrypt KDFs.
- Security:
  - Unified `SecurityError` boundary and deterministic exit codes.
  - Zeroized password handling for memory safety (passwords stored in `Zeroizing<String>` and zeroed after use).
  - Addresses use Base58Check by default for storage and output (legacy import flags available).
  - `secure_atomic_write` for durable atomic file writes (temp file in same dir, fsync+rename+dir-fsync, tightened permissions on Unix).
  - Validated KDF choices and parameters early to avoid weak configurations.
- Engineering:
  - Replaced direct `getrandom` calls with `rand::rngs::OsRng` for compatibility.
  - Comprehensive unit and integration tests for wallet functionality.
  - CI format/lint fixes applied and workflow validated across platforms.
  - Dependency updates: bumped `console` from 0.15.11 to 0.16.0 (Dependabot PR #2), CI verified and merged into `main`.

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

## Try it (quick examples)

- Create a 12-word English mnemonic and write it to stdout:

  ```bash
  ark-wallet mnemonic new --lang en --words 12
  ```

- Create a keystore using interactive password prompts and save as JSON:

  ```bash
  ark-wallet keystore create --mnemonic "<mnemonic>" --password-prompt --password-confirm --file keystore.json
  ```

- Export the private key hex to a file (JSON output with absolute paths):

  ```bash
  ark-wallet keystore export --file keystore.json --password-stdin --out-priv priv.hex --json
  ```

## Migration notes

- Addresses are stored and printed using Base58Check by default. If you have older tooling that expects plain Base58 (no checksum), update it to use Base58Check or use the provided legacy import flags during migration.
- KDF parameters are validated at keystore creation time; older weak parameters (very low PBKDF2 iterations or low scrypt N/r/p) will be rejected — regenerate keystores with stronger parameters if necessary.

## Security considerations

- Passwords are held in memory as `zeroize::Zeroizing<String>` and zeroized when dropped, but ensure you use secure input methods (avoid keeping passwords in shell history). Use `--password-stdin` for automation.
- Keystore files are written atomically with `secure_atomic_write` (temp file in same directory, flush & fsync, rename, dir sync); on Unix the final file permission is tightened to `0o600` where supported.
- Randomness for temporary file suffixes and nonces uses OS RNG (`OsRng`) rather than direct getrandom calls to improve portability across platforms.

If you have questions about the release process or need a backport to a different branch, open an issue or tag a maintainer in this PR.

