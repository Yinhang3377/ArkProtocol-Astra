Pin `memchr` to 2.7.5

Rationale:
- `memchr` appears high in the quick geiger report; pinning to 2.7.5 keeps dependency graph stable while we audit usage.

Change:
- Add `memchr = "2.7.5"` to workspace dependencies.

No code changes.
