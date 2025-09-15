## Triage: Top unsafe crates (quick geiger report)

Generated from the CI per-crate quick report (runner-produced `geiger-quick-report`) on branch `audit/geiger-top30-draft-pr`.

This is a conservative, action-oriented triage to prioritize follow-up work. The runner produced an empty `geiger-full.json` in this run, so this summary uses the downloaded quick-report and previously-generated aggregated CSV (`tmp_logs/geiger_top30_aggregated.csv`).

### Top crates by unsafe expressions used

| Rank | crate | version | unsafe_expr_used | functions_used | impls_used |
|---:|:---|:---:|---:|---:|---:|
| 1 | generic-array | 0.14.7 | 1995 | 7 | 140 |
| 2 | memchr | 2.7.5 | 1973 | 27 | 2 |
| 3 | sha2 | 0.10.9 | 1424 | 64 | 0 |
| 4 | aes | 0.8.4 | 765 | 18 | 0 |
| 5 | windows-sys | 0.60.2 | 708 | 0 | 0 |
| 6 | ppv-lite86 | 0.2.21 | 632 | 2 | 0 |
| 7 | ryu | 1.0.20 | 572 | 7 | 0 |
| 8 | anyhow | 1.0.99 | 464 | 16 | 3 |
| 9 | bitcoin_hashes | 0.13.0 | 409 | 1 | 0 |
| 10 | zerocopy | 0.8.27 | 403 | 5 | 40 |
| 11 | console | 0.16.0 | 380 | 2 | 0 |
| 12 | encode_unicode | 1.0.0 | 223 | 0 | 0 |
| 13 | inout | 0.1.4 | 126 | 0 | 0 |

Notes:
- These counts are from a per-crate quick report and an aggregated CSV created locally; they reflect *usage of unsafe code* (expressions / functions / impls) within those crates, not necessarily vulnerabilities.
- Many widely-used crates (e.g., `generic-array`, `memchr`, `sha2`) implement unsafe blocks for performance or low-level operations; unsafe usage alone is not a finding but a signal for additional review.

### Recommended, conservative next steps (prioritized)

1) Top-5 review (immediate, high value)
  - Crates: `generic-array`, `memchr`, `sha2`, `aes`, `windows-sys`.
  - Actions:
    - Audit how the crate is used in our code (which public APIs are imported/relied upon).
    - Search for direct re-exports or wrappers in our own crates (grep for `use generic_array` etc.).
    - Add focused unit tests around code paths that touch these crates (fuzz targets if applicable).
    - Where feasible, update to the latest patch/minor release (read changelog for safety-critical changes).
    - If the crate is a transitive dependency, consider pinning the direct dependency that pulls it or adding a `Cargo.toml` entry with an explicit version to reduce surprise upgrades.

2) Medium-priority (next week)
  - Crates: `ppv-lite86`, `ryu`, `anyhow`, `bitcoin_hashes`, `zerocopy`.
  - Actions:
    - For crypto-related crates (`bitcoin_hashes`, `zerocopy` used in serialization), ensure dependency versions are the latest stable and scan changelogs for security fixes.
    - For `anyhow` and utils, prefer limiting surface area and avoid exposing internal `unsafe`-backed types across FFI boundaries.

3) Low-priority / housekeeping
  - Crates like `console`, `encode_unicode`, and small helpers typically use unsafe for internal optimizations; keep them pinned and monitored.

### Conservative remediation PR pattern (what I will prepare if you ask)

- Create a small set of draft PRs (one per crate or small group) that contain only conservative changes:
  1. Add an explicit `Cargo.toml` pin or bump to a safe patch release (no code changes).
  2. Add a short `SECURITY` note in the crate that uses it (or repo-level `SECURITY/triage_top_unsafe.md`) summarizing why we touched the dependency and what to watch for.
  3. Add focused unit tests (if feasible) exercising the paths that depend on the crate; do not rewrite internal crate code.

Example PR commit contents (minimal):
- `crates/ark-foo/Cargo.toml` - bump `sha2` = "0.10.11" (example)
- `SECURITY/triage_top_unsafe.md` - add note for the bumped crate and rationale

This pattern is intentionally low-risk: it avoids modifying upstream code, avoids vendoring, and focuses on observability and reproducible builds.

### What I'll do next if you approve

- Draft the conservative PRs (one per top-5 crate) and push them on the `audit/geiger-top30-draft-pr` branch as separate commits. I will not open the PRs automatically; I'll push the branch updates and show the diffs for your approval, or I can open the PRs if you say so.
- If you prefer, I can instead only prepare the diffs (no push) for review.

### Caveats and notes

- The runner-created `geiger-full.json` artifact was empty in this CI run; the quick reports and the aggregated CSV are sufficient for an initial triage, but when available the full JSON should be used to finalize prioritization.
- Unsafe usage is a signal for review, not proof of vulnerability. Each crate should be reviewed in context: how we use it and whether unsafe blocks are behind well-tested APIs.

---
Last updated: 2025-09-15
