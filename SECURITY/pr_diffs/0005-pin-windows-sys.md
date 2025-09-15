Pin `windows-sys` to 0.60.2

Rationale:
- `windows-sys` shows many platform-specific unsafe implementations; pinning reduces unexpected changes on Windows targets during the audit.

Change:
- Add `windows-sys = "0.60.2"` to workspace dependencies.

No code changes.
