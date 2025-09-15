<#
Auto-fix helper for CI failures (PowerShell)
Usage: .\auto_fix_ci.ps1 [-DryRun]

This safer helper will:
 - list recent failed/cancelled workflow runs using the GitHub CLI (gh)
 - download their logs into ./tmp_logs
 - scan logs for a small set of known problems (POSIX `mkdir -p` run under PowerShell, `.sccache` conflicts, `cargo fmt` check failures)
 - for detected issues it will create a draft Pull Request proposing a conservative textual patch (YAML snippet) that a maintainer can review and merge

This approach avoids making fragile in-place regex edits to YAML and keeps changes human-reviewed.
#>

param(
    [switch]$DryRun
)

function RunCmd([string]$cmd) {
    Write-Host "> $cmd"
    if (-not $DryRun) { & cmd /c $cmd }
}

# 1) list recent failed or cancelled runs
$runsJson = gh run list --limit 100 --json databaseId,conclusion,headBranch,name,event,createdAt 2>$null
$runs = $runsJson | ConvertFrom-Json
if (-not $runs) {
    Write-Host "No workflow runs returned by gh. Ensure gh is authenticated and repo is correct."
    exit 0
}

$interesting = $runs | Where-Object { $_.conclusion -eq 'failure' -or $_.conclusion -eq 'cancelled' }
if (-not $interesting) {
    Write-Host "No recent failed or cancelled runs found."
    exit 0
}

Write-Host "Found $(($interesting).Count) failed/cancelled runs. Downloading logs into ./tmp_logs"
if (-not (Test-Path .\tmp_logs)) { New-Item -ItemType Directory -Path .\tmp_logs | Out-Null }

foreach ($r in $interesting) {
    $id = $r.databaseId
    $outfile = Join-Path -Path .\tmp_logs -ChildPath ("$id.log")
    Write-Host "Downloading run $id -> $outfile"
    if (-not $DryRun) { gh run view $id --log > $outfile }
}

# 2) scan logs for patterns and prepare suggested fixes
$suggested = @()
Get-ChildItem .\tmp_logs -Filter *.log | ForEach-Object {
    $text = Get-Content $_.FullName -Raw
    if ($text -match "mkdir -p" -and $text -match "\\.sccache") {
        $suggested += @{file='.github/workflows/release-artifacts.yml'; issue='mkdir-sccache-windows'; log=$_.FullName}
    }
    elseif ($text -match "mkdir -p") {
        $suggested += @{file='.github/workflows/release-artifacts.yml'; issue='mkdir-windows'; log=$_.FullName}
    }
    if ($text -match "cargo fmt -- --check") {
        $suggested += @{file='workspace'; issue='rustfmt'; log=$_.FullName}
    }
}

if (-not $suggested) {
    Write-Host "No auto-fixable issues found in logs. Check ./tmp_logs manually."
    exit 0
}

# YAML snippet suggestion for workflows to handle mkdir/.sccache on Windows
$ymlSnippet = @'
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

foreach ($s in $suggested | Select-Object -Unique) {
        $issue = $s.issue
        $log = $s.log
        Write-Host "Preparing PR for issue: $issue (evidence: $log)"

    $branch = "auto-fix/ci-$issue-$(Get-Random)"
    $title = "ci: propose fix for $issue detected in CI logs"

    Write-Host "Will create branch: $branch and a PR: $title"
    if (-not $DryRun) {
        # if branch already exists locally, just checkout it; otherwise create it
        & git rev-parse --verify $branch 2>$null
        if ($LASTEXITCODE -ne 0) {
            RunCmd("git checkout -b $branch")
        } else {
            RunCmd("git checkout $branch")
        }

        $cmd = "git commit --allow-empty -m " + '"' + $title + '"'
        RunCmd($cmd)

        $cmd = "git push --set-upstream origin " + $branch
        RunCmd($cmd)

        # Ensure tmp_logs exists and sanitize branch for filename (replace any path separators)
        if (-not (Test-Path .\tmp_logs)) { New-Item -ItemType Directory -Path .\tmp_logs | Out-Null }
        $safeBranch = $branch -replace '[\\/]', '-'
        $bodyFile = Join-Path .\tmp_logs ("$safeBranch-body.txt")

        $bodyLines = @(
            "I detected a CI failure related to '$issue' in logs (`$log`).",
            "",
            "Suggested workflow snippet to add (please review before merging):",
            "",
            '```yaml',
            $ymlSnippet,
            '```',
            "",
            "This PR is auto-created as a conservative proposal; it does not modify repository files automatically to avoid risky edits."
        )

        Set-Content -Path $bodyFile -Value $bodyLines -Force -Encoding UTF8
        $cmd = "gh pr create --title " + '"' + $title + '"' + " --body-file " + '"' + $bodyFile + '"' + " --base main --draft"
        RunCmd($cmd)
    }
}

Write-Host "Done. Review created PRs (or check the suggested changes in the PR body)."
