Pin `aes` to 0.8.4

Rationale:
- `aes` is used via crypto crates; pinning avoids implicit transitive updates during the audit.

Change:
- Add `aes = "0.8.4"` to workspace dependencies.

No code changes.
