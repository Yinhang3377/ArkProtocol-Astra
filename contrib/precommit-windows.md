# Installing pre-commit on Windows (recommended steps)

This repo provides a `.pre-commit-config.yaml` that uses local PowerShell scripts and Rust formatters. Follow these steps to enable hooks on Windows.

Recommended (pipx, per-user, no admin):
1. Ensure Python is installed and `python` is on PATH (the Windows Store Python shim works, but installing CPython from python.org is recommended).
2. Open PowerShell (regular user) and run:

```powershell
python -m pip install --user pipx
python -m pipx ensurepath
# Close and re-open your PowerShell to pick up pipx on PATH, or run the ensurepath output manually.
pipx install pre-commit
pre-commit install
pre-commit run --all-files
```

Alternative (Chocolatey, admin recommended):

```powershell
# Run as Administrator
choco install pipx --yes
# Open a new PowerShell after installation
python -m pipx ensurepath
pipx install pre-commit
pre-commit install
pre-commit run --all-files
```

Fallback (pip --user):

```powershell
python -m pip install --user pre-commit
# Make sure user scripts are on PATH, e.g. add:
# $env:PATH += ';' + "$env:USERPROFILE\AppData\Roaming\Python\Scripts"
pre-commit install
```

Troubleshooting
- "pipx not found" — ensure you ran `python -m pipx ensurepath` and re-opened PowerShell.
- If you installed with Chocolatey and used a system-wide location, restart your shell or log out/in.
- If using Windows Store Python, some commands may behave silently. Prefer CPython from python.org when possible.

If you want, run `.	ools\install_precommit_windows.ps1` (created in this repo) which will attempt the non-elevated install steps and print exact commands to run as admin if needed.
