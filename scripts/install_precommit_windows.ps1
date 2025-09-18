<#
Attempt to install pre-commit on Windows. This script tries a non-elevated user install first (pipx via pip --user), then falls back to pip --user.
It will print exact commands you can run as Administrator if a system install is required.
#>
Write-Host "Attempting non-elevated install of pre-commit (user scope)..."
try {
    python -m pip install --user pipx | Out-Null
    python -m pipx ensurepath | Out-Null
    Write-Host "Installed pipx (user). Attempting to install pre-commit via pipx..."
    pipx install pre-commit
    if (Get-Command pre-commit -ErrorAction SilentlyContinue) {
        Write-Host "pre-commit is installed and on PATH"
        pre-commit install
        Write-Host "pre-commit hooks installed"
        exit 0
    }
} catch {
    Write-Host "Non-elevated pipx install failed or pipx not available: $_"
}

Write-Host "Trying pip --user install as fallback..."
try {
    python -m pip install --user pre-commit | Out-Null
    # Try to run pre-commit via python -m pre_commit
    python -m pre_commit --version | Out-Null
    Write-Host "pre-commit module is available via python -m pre_commit"
    Write-Host "Installing git hooks via python -m pre_commit install"
    python -m pre_commit install
    Write-Host "pre-commit hooks installed via python -m pre_commit"
    exit 0
} catch {
    Write-Host "pip --user install failed: $_"
}

Write-Host "If the above failed, run the following as Administrator:"
Write-Host "choco install pipx --yes"
Write-Host "python -m pipx ensurepath"
Write-Host "pipx install pre-commit"
Write-Host "pre-commit install"

exit 1
