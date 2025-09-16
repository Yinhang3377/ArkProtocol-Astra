sha2 usage and audit note

Rationale:
- `sha2` (v0.10.9 reported) shows many unsafe expressions (optimized code paths). We're not changing `sha2` in this commit beyond the workspace pin (already present as `sha2 = "0.10"`).

Change:
- No version bump applied in this commit. We recommend a follow-up to bump to the latest patch release if available after reading changelogs.

Action:
- Add focused unit tests around hash usage and double-check upstream advisories.
