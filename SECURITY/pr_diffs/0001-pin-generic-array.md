Pin `generic-array` to 0.14.7

Rationale:
- `generic-array` showed the highest number of unsafe expressions in the quick geiger report. This pin is conservative: it ensures reproducible builds and avoids surprise transitive upgrades.

Change:
- Add `generic-array = "0.14.7"` to workspace dependencies.

No code changes.
