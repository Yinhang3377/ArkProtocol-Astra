# Audit summary (cargo-audit + cargo-geiger)

This draft audit collects the lockfile advisory scan and an unsafe-usage scan (cargo-geiger) for the workspace. The CI run attached to this branch will produce the following artifacts:

- `target/audit/audit.json` — advisory (cargo-audit) JSON output
- `target/geiger/geiger-full.json` — full cargo-geiger JSON output
- `target/geiger/extract_cli.txt` — concise extractor report for target crates
- `target/geiger/geiger-human.txt` — human-friendly geiger output

Next steps:
1. Review `extract_cli.txt` for the six target crates (getrandom, signal-hook-registry, secp256k1, backtrace, bytes, smallvec).
2. For crates with non-zero *used unsafe* entries, inspect upstream files/lines and decide: accept (documented) / upgrade / replace.
3. Apply conservative dependency upgrades where safe and open follow-up PRs for any invasive changes.

Short automated notes:
- This branch is a draft audit. No source history rewriting was performed for sensitive files.
- CI runs on `ubuntu-latest` to ensure stable cargo-geiger JSON output.
