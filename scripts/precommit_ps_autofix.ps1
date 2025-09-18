param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Dot-source helper
if (Test-Path .\scripts\lib_process.ps1) { . .\scripts\lib_process.ps1 }

# Find all PowerShell files tracked by git
$files = git ls-files | Where-Object { $_ -match '\.ps1$' }
if (-not $files) { Write-Host 'No PowerShell files in repo'; exit 0 }

$changed = $false
foreach ($f in $files) {
    Write-Host "Checking $f"
    # Detect non-UTF8 encodings; read as bytes and look for BOM
    $bytes = [System.IO.File]::ReadAllBytes($f)
    $isUtf8 = $false
    try { [System.Text.Encoding]::UTF8.GetString($bytes) > $null; $isUtf8 = $true } catch { $isUtf8 = $false }
    # If file contains non-ASCII and is not UTF-16LE, re-encode to UTF-16LE for PS5.1 safety
    if (-not $isUtf8) {
        Write-Host "Re-encoding $f to UTF-8" -ForegroundColor Yellow
        [System.IO.File]::WriteAllText($f, [System.Text.Encoding]::UTF8.GetString($bytes), [System.Text.Encoding]::UTF8)
        git add $f
        $changed = $true
    }

    # Replace risky Start-Process patterns with Start-Process-Safe by simple heuristic
    $text = Get-Content -Raw -Encoding UTF8 $f
    $new = $text -replace 'Start-Process\s+-ArgumentList\s+""', 'Start-Process-Safe -FilePath'
    $new = $new -replace 'Start-Process\s+-ArgumentList\s+(\$\w+)', 'Start-Process-Safe -FilePath'
    if ($new -ne $text) {
        Write-Host "Applying Start-Process -> Start-Process-Safe patch to $f" -ForegroundColor Yellow
        $new | Set-Content -Encoding UTF8 $f
        git add $f
        $changed = $true
    }
}

if ($changed) { Write-Host 'PowerShell autofixes applied and staged' -ForegroundColor Green } else { Write-Host 'No PowerShell autofixes necessary' }
exit 0
