param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Run pre-commit on all files with fix
Write-Host 'Running pre-commit --all-files --show-diff --verbose --hook-stage pre-commit'
$proc = Start-Process -FilePath 'pre-commit' -ArgumentList 'run','--all-files','--show-diff','--verbose','--hook-stage','pre-commit' -NoNewWindow -PassThru -Wait -ErrorAction Stop
if ($proc.ExitCode -eq 0) { Write-Host 'pre-commit fixed everything (exit 0)' -ForegroundColor Green; exit 0 }

# If there were fixes, ask whether to apply them or auto-apply if config set
$autofix = git config --global hooks.autofix 2>$null
if ($autofix -eq 'true') {
    Write-Host 'Auto-apply enabled (git config hooks.autofix true). Staging and committing fixes.' -ForegroundColor Yellow
    git add -A
    git commit -m "chore: apply pre-commit autofixes"
    if ($LASTEXITCODE -ne 0) { Write-Host 'No changes to commit' }
    exit 0
} else {
    while ($true) {
        $resp = Read-Host "Apply fixes and commit? (y/N)"
        if ($resp -match '^[Yy]$') {
            git add -A
            git commit -m "chore: apply pre-commit autofixes"
            if ($LASTEXITCODE -ne 0) { Write-Host 'No changes to commit' }
            exit 0
        } elseif ($resp -match '^[Nn]?$') {
            Write-Host 'User declined to apply fixes. Aborting commit.' -ForegroundColor Red
            exit 1
        }
    }
}
