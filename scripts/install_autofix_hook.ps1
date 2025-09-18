param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$hookPath = Join-Path -Path (git rev-parse --git-dir) -ChildPath 'hooks\pre-commit'
Write-Host "Installing autofix pre-commit hook to $hookPath"
$content = @'
#!/usr/bin/env pwsh
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts\autofix_runner.ps1
'@

Set-Content -Path $hookPath -Value $content -Encoding UTF8
# Ensure hook is executable on POSIX systems (if applicable)
try { icacls $hookPath /grant Everyone:RX } catch { }
Write-Host 'Pre-commit autofix hook installed.'
exit 0
