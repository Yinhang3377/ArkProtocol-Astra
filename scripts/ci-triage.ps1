<#
ci-triage.ps1

Usage: Run from repo root in PowerShell (VS Code integrated terminal)
  .\scripts\ci-triage.ps1 -RepoOwner 'Yinhang3377' -RepoName 'ArkProtocol-Astra'

What it does (safe mode):
 - Checks and optionally aborts unfinished merges (asks first)
 - Fetches recent GH Actions runs (limit 10)
 - Finds failed runs and downloads logs
 - Extracts last 20 lines of failing steps
 - Attempts to classify common failures and writes suggested fixes to ci-fixes.txt
 - Optionally writes a fix script (do not run without review)

This script requires the `gh` CLI to be available and authenticated.
#>
param(
    [string]$RepoOwner = 'Yinhang3377',
    [string]$RepoName = 'ArkProtocol-Astra',
    [int]$Limit = 10,
    [switch]$AutoAbortMerge,
    [switch]$DryRun
)

$repo = "$RepoOwner/$RepoName"
$fixFile = Join-Path $PWD 'ci-fixes.txt'
$logDir = Join-Path $PWD 'gh_logs'
$exDir = Join-Path $logDir 'extracted'

Write-Host "CI triage starting for $repo" -ForegroundColor Cyan

# Step 0: detect unfinished merge
if (Test-Path .git\MERGE_HEAD) {
    Write-Host "Detected unfinished merge (MERGE_HEAD present)." -ForegroundColor Yellow
    if ($AutoAbortMerge) {
        git merge --abort
        Write-Host "Aborted merge (git merge --abort)" -ForegroundColor Green
    } else {
        $choice = Read-Host "Unfinished merge detected. Type 'abort' to abort merge, 'commit' to complete merge, or 'skip' to continue without touching git"
        if ($choice -eq 'abort') {
            git merge --abort
            Write-Host "Merge aborted" -ForegroundColor Green
        } elseif ($choice -eq 'commit') {
            Write-Host "Please stage any resolved files, then press Enter to continue..."
            Read-Host
            git commit -m "Finish merge from triage"
            Write-Host "Merge commit created" -ForegroundColor Green
        } else {
            Write-Host "Leaving merge state alone" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "No in-progress merge detected" -ForegroundColor Green
}

# Step 1: fetch and list recent runs
Write-Host "Listing recent workflow runs (limit $Limit)" -ForegroundColor Cyan
$runsOut = gh run list --repo $repo --limit $Limit --json id,name,head_branch,status,conclusion,html_url 2>$null
if (-not $runsOut) { Write-Host "gh run list returned no output. Ensure gh auth status." -ForegroundColor Red; exit 1 }
$runs = $runsOut | ConvertFrom-Json
$fails = $runs | Where-Object { $_.conclusion -ne $null -and $_.conclusion -ne 'success' }
if (-not $fails) { Write-Host "No failed runs found" -ForegroundColor Green; exit 0 }

# Ensure clean log directories
if (Test-Path $logDir) { Remove-Item -Recurse -Force $logDir }
New-Item -ItemType Directory -Path $logDir | Out-Null

# prepare fixes file
"CI triage fixes generated on $(Get-Date -Format o) for $repo`n" | Out-File -FilePath $fixFile -Encoding utf8

foreach ($r in $fails) {
    Write-Host "Processing run id $($r.id) name $($r.name) branch $($r.head_branch) conclusion $($r.conclusion)" -ForegroundColor Yellow
    "Run: $($r.id) $($r.name) $($r.head_branch) $($r.html_url) - conclusion: $($r.conclusion)" | Out-File -Append -FilePath $fixFile

    # download run logs
    Write-Host "Downloading logs for run $($r.id) ..." -ForegroundColor Cyan
    gh run download $($r.id) --repo $repo -D $logDir 2>$null

    # run the GH run log scanner only if the downloaded run log exists and is readable
    try {
        # find the downloaded run log file (first match) and pass it directly to the scanner
        $runLogDir = Join-Path $PWD 'ci_artifacts\gh_runs'
        $match = Get-ChildItem -LiteralPath $runLogDir -Filter "run_$($r.id)*.log" -File -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($match) {
            $logPath = $match.FullName
            # try to read the file; retry up to SCAN_RETRY times with SCAN_BACKOFF backoff if locked/not ready
            $maxAttempts = if ($env:SCAN_RETRY) { [int]$env:SCAN_RETRY } else { 3 }
            $attempt = 0
            $ready = $false
            while ($attempt -lt $maxAttempts -and -not $ready) {
                try {
                    $null = Get-Content -LiteralPath $logPath -Raw -ErrorAction Stop
                    $ready = $true
                } catch {
                    $attempt++
                    if ($attempt -lt $maxAttempts) { $backoff = if ($env:SCAN_BACKOFF) { [int]$env:SCAN_BACKOFF } else { 3000 }; Start-Sleep -Milliseconds $backoff }
                }
            }
            if (-not $ready) {
                Write-Error "Run log $logPath not readable after $maxAttempts attempts"
                exit 2
            }

            $scanner = Join-Path $PSScriptRoot 'scan_gh_run_logs.ps1'
            if ($DryRun) {
                # print which lines would be fixed: read per-run summary json if present
                $summaryJson = Join-Path $PWD ("ci_artifacts\run_${($r.id)}_summary.json")
                if (Test-Path -LiteralPath $summaryJson) {
                    $j = Get-Content -LiteralPath $summaryJson -Raw | ConvertFrom-Json
                    Write-Host "DryRun: would fix run $($j.runId):" -ForegroundColor Yellow
                    foreach ($l in $j.lines) { Write-Host "  $($l.file):$($l.line) - $($l.text)" }
                } else {
                    Write-Host "DryRun: no per-run JSON, falling back to summary file" -ForegroundColor DarkGray
                    $sum = Get-Content -LiteralPath (Join-Path $PWD 'ci_artifacts\gh_run_errors_summary.txt') -ErrorAction SilentlyContinue
                    if ($sum) { Write-Host $sum } else { Write-Host "DryRun: no summary available" }
                }
            } else {
                if (Test-Path -LiteralPath $scanner) { & $scanner $logPath } else { Write-Host "Scanner not found: $scanner" -ForegroundColor Yellow }
                # react to scanner result
                if ($LASTEXITCODE -ne 0) {
                    Write-Host 'Issue found; running remediation' -ForegroundColor Red
                    & $PSScriptRoot\ci-remediate.ps1 -RepoOwner $RepoOwner -RepoName $RepoName -Limit 1 -AutoAbortMerge
                } else {
                    Write-Host 'All green' -ForegroundColor Green
                }
            }
        } else {
            Write-Host "No run log found for $($r.id); skipping scanner invocation" -ForegroundColor DarkGray
        }
    } catch {
        Write-Warning "scan_gh_run_logs.ps1 invocation failed: $($_.ToString())"
    }

    # expand zips
    Get-ChildItem $logDir -Filter '*.zip' -File -ErrorAction SilentlyContinue | ForEach-Object {
        try { Expand-Archive -Path $_.FullName -DestinationPath $exDir -Force } catch { }
    }

    # find txt logs and pick those referencing failed jobs
    $txts = Get-ChildItem $exDir -Recurse -Filter '*.txt' -ErrorAction SilentlyContinue
    if (-not $txts) {
        Write-Host "No text logs found for run $($r.id)" -ForegroundColor Yellow
        "No text logs found for run $($r.id)" | Out-File -Append -FilePath $fixFile
        continue
    }

    # heuristics: find files with 'error' string or last modified
    $cands = $txts | Sort-Object LastWriteTime -Descending | Select-Object -First 5
    foreach ($c in $cands) {
        "LogFile: $($c.FullName)" | Out-File -Append -FilePath $fixFile
        $lastLines = Get-Content $c.FullName -Tail 200 -ErrorAction SilentlyContinue
        if ($lastLines) {
            $snippet = $lastLines | Select-Object -Last 20
            "---- Last 20 lines of $($c.Name):" | Out-File -Append -FilePath $fixFile
            $snippet | Out-File -Append -FilePath $fixFile

            # simple pattern matches
            $joined = $snippet -join "`n"
            if ($joined -match "GPG_PRIVATE_KEY" -or $joined -match "gpg: .*no secret key") {
                "Detected GPG/secret issue" | Out-File -Append -FilePath $fixFile
                "Suggested remediation: add secret GPG_PRIVATE_KEY and optionally GPG_PASSPHRASE via 'gh secret set' or web UI" | Out-File -Append -FilePath $fixFile
                "Example: gh secret set GPG_PRIVATE_KEY -b (Get-Content -Raw 'C:\path\to\gpg.key') --repo $repo" | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "tag.*not found|github.event.release.tag_name" -or $joined -match "github.ref_name") {
                "Detected tag/ref issue" | Out-File -Append -FilePath $fixFile
                "Suggested remediation: ensure the release tag exists. Example: git tag -a v0.1.0 -m 'v0.1.0' && git push origin v0.1.0" | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "rustfmt|rustfmt --check|formatting" -or $joined -match "error: could not find" ) {
                "Detected formatting or toolchain issue" | Out-File -Append -FilePath $fixFile
                "Suggested remediation: run 'cargo fmt --all' locally, produce patch, and push branch. CI will now produce format-fix.patch if it failed." | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "clippy|warning: " -and $joined -match "error: aborting due to" ) {
                "Detected Clippy error" | Out-File -Append -FilePath $fixFile
                "Suggested remediation: inspect clippy-fix.patch artifact or run 'cargo clippy -p ark-wallet-cli --fix -Z unstable-options' locally and review changes." | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "permission denied" -or $joined -match "access denied") {
                "Detected permission denied; likely missing GITHUB_TOKEN or insufficient permissions on workflow" | Out-File -Append -FilePath $fixFile
                "Suggested remediation: check workflow permissions and GITHUB_TOKEN scope." | Out-File -Append -FilePath $fixFile
            }
            "" | Out-File -Append -FilePath $fixFile
        }
    }

}

Write-Host "Triage complete. See $fixFile for suggested remediation entries." -ForegroundColor Green