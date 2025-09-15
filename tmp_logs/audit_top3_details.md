Audit notes for top-3 unsafe dependencies

1) sha2 (v0.10.9)
- Resolved version: 0.10.9 (see Cargo.lock)
- Where used: `crates/ark-wallet-cli` imports `sha2` directly (workspace dependency); also transitively pulled by `bip32` and other crypto crates.
- Direct / Transitive: Direct in workspace (workspace dependency), also transitive via crypto crates (bip32, pbkdf2 usage through digest).
- Known CVEs: No known advisories reported by `cargo audit` in the previous run (cargo-audit reported 0 vulnerabilities for the wallet crate). Recommend double-checking upstream advisories for `sha2` and `digest` crates.
- Risk notes: `sha2` contains optimized implementations with occasional unsafe blocks for performance; geiger reports many unsafe expressions (1424 aggregated) primarily in platform-optimized code paths.
- Suggested mitigations:
  - Pin to the resolved micro-version (0.10.9) in workspace to avoid accidental upgrades.
  - Add focused unit tests validating hashes and boundary conditions used by wallet code (double-Sha256 checksums, pbkdf2 inputs).
  - Consider adding a small audit harness that re-implements critical hash checks via pure-safe wrappers (for cross-checking) if performance allows.

2) bitcoin_hashes (v0.13.0)
- Resolved version: 0.13.0
- Where used: Transitive dependency; used by `bip39` and other Bitcoin-related crates (bip39 listed bitcoin_hashes dependency in Cargo.lock). No direct `bitcoin_hashes =` lines in workspace Cargo.toml.
- Direct / Transitive: Transitive
- Known CVEs: No CVEs found by `cargo audit` in the earlier run for the wallet crate; re-run cargo-audit in CI for PRs.
- Risk notes: The crate is small (geiger shows 409 unsafe expressions aggregated) and used for constructing/validating cryptocurrency hashes; careful boundary testing required around decoding/encoding functions.
- Suggested mitigations:
  - Add unit tests for address/encoding path that exercise `bitcoin_hashes` usage.
  - Pin transitive versions in `Cargo.lock` (already locked) and add CI `cargo-audit` checks.

3) zerocopy (v0.8.27)
- Resolved version: 0.8.27
- Where used: Transitive — found in the dependency graph (e.g., `ppv-lite86` depends on `zerocopy`; other crates may use derive macros `zerocopy-derive`).
- Direct / Transitive: Transitive
- Known CVEs: None reported by `cargo audit` earlier.
- Risk notes: `zerocopy` provides zero-copy parsing with `unsafe` for performance and explicit `FromBytes/AsBytes` traits; misuse can lead to UB if assumptions about alignment/endianness/size are violated.
- Suggested mitigations:
  - Identify code paths that read untrusted input and ensure proper bounds checks before interpreting memory as typed structures.
  - Add Miri runs for tests covering parsing/unpacking code paths using zerocopy to catch UB.
  - Where possible, wrap zerocopy usage with safe abstractions that perform explicit bounds/length checks.

Next steps I can take automatically:
- Add these notes to the existing PR as `tmp_logs/audit_top3_details.md` and push the update to branch `audit/geiger-top30-draft-pr`.
- Add a lightweight GitHub Actions workflow to run `cargo-audit` + `cargo-geiger` on PRs and push it together in the same PR.

Would you like me to (A) push the detailed audit notes to the PR only, or (B) push notes + a draft CI workflow (cargo-audit + cargo-geiger) into the same PR?