# Creates a safe PR body file and opens a draft PR for an existing branch
$branch = 'auto-fix/ci-mkdir-windows-992162228'
$safe = $branch -replace '[\\/]','-'
$bodyFile = Join-Path (Join-Path $PSScriptRoot '..') "tmp_logs\$safe-body.txt"
if (-not (Test-Path (Split-Path $bodyFile -Parent))) { New-Item -ItemType Directory -Path (Split-Path $bodyFile -Parent) | Out-Null }
$yml = @'
        - name: Create dist directory (Unix)
            if: runner.os != 'Windows'
            run: mkdir -p dist
            shell: bash

        - name: Create dist directory (Windows)
            if: runner.os == 'Windows'
            run: New-Item -ItemType Directory -Path "$env:GITHUB_WORKSPACE\\dist" -Force | Out-Null
            shell: pwsh

        - name: Create local sccache dir (Unix)
            if: runner.os != 'Windows'
            run: mkdir -p ${{ github.workspace }}/.sccache
            shell: bash

        - name: Create local sccache dir (Windows)
            if: runner.os == 'Windows'
            run: New-Item -ItemType Directory -Path "$env:GITHUB_WORKSPACE\\.sccache" -Force | Out-Null
            shell: pwsh
'@

$body = @'
I detected a CI failure related to 'mkdir-windows' in logs ('./tmp_logs/17710465613.log').

Suggested workflow snippet to add (please review before merging):

```yaml
'@ + "`n" + $yml + "`n" + @'
```

This PR is auto-created as a conservative proposal; it does not modify repository files automatically to avoid risky edits.
'@

Set-Content -Path $bodyFile -Value $body -Encoding UTF8 -Force
Write-Host "Wrote PR body to: $bodyFile"

# Create the PR (draft) for the existing branch
$cmd = "gh pr create --title 'ci: propose fix for mkdir-windows detected in CI logs' --body-file `"$bodyFile`" --base main --head $branch --draft"
Write-Host "> $cmd"
Invoke-Expression $cmd
