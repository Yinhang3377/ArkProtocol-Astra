CI auto-fix helper scripts

This folder contains conservative helpers to assist maintainers in diagnosing and proposing fixes for recurring CI failures.

auto_fix_ci.ps1
- PowerShell helper that:
  - lists recent failed or cancelled GH Actions runs (via `gh`)
  - downloads logs to `./tmp_logs`
  - scans logs for a short set of known issues (POSIX `mkdir -p` on Windows, `.sccache` conflicts, `cargo fmt` failures)
  - creates draft PRs that propose YAML snippets or suggest running `cargo fmt` locally (no automatic in-file edits)

Usage (Windows PowerShell):

```powershell
# dry-run (no changes pushed)
.\.github\scripts\auto_fix_ci.ps1 -DryRun

# actually create PRs (pushes branches) - ensure gh is authenticated and you have push rights
.\.github\scripts\auto_fix_ci.ps1
```

Notes
- The script is intentionally conservative. It creates PRs with suggested fixes rather than applying automated patches directly to workflow files.
- Review PRs before merging.
