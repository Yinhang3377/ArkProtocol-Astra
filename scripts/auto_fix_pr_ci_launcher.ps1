# Launcher for auto_fix_pr_ci.ps1
# Ensures working directory is repo root and forwards all args to the watcher.
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location -LiteralPath $RepoRoot
$watcher = Join-Path -Path $RepoRoot -ChildPath 'scripts\auto_fix_pr_ci.ps1'
if(-not (Test-Path $watcher)){
    Write-Error "Watcher script not found at $watcher"
    exit 2
}
# Forward all args
& $watcher @args
