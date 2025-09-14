# Changelog

All notable changes to this project will be documented in this file.

## [v0.4.2] - 2025-09-14
### Added
- `secure_atomic_write` in `crates/ark-wallet-cli/src/security/fs.rs` for durable atomic writes.
- Base58Check enforcement for addresses and import-time compatibility helpers.
- `rand = "0.8"` and `hex = "0.4"` dependencies for secure randomness and temp suffix generation.

### Changed
- Unify `SecurityError` across wallet CLI modules; top-level mapping to deterministic exit codes.
- Passwords handled with `zeroize::Zeroizing<String>` and explicit zeroization after use.
- KDF parameter validation to reject weak configurations early.
 - Bumped `console` from 0.15.11 to 0.16.0 (Dependabot PR #2) — recorded in `RELEASE.md` and verified by CI (commit ea11d6e).

### Fixed
- Formatting/lint issues across CLI codebase; CI formatting enforced.

