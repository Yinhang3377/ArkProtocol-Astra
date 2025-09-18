<#
ci-remediate.ps1

Automates a safe remediation loop for failing GitHub Actions runs:
 - aborts unfinished merges (optional)
 - pulls origin/main
 - lists recent failed runs
 - downloads run logs, extracts last 20 lines of failed steps
 - matches common failure patterns and writes remediation suggestions
 - attempts to apply patch artifacts (format-fix.patch, clippy-fix.patch) if present
 - commits/amends and force-pushes fixes to current branch (prompted)
 - reruns the failing workflow run
 - loops until no failed runs remain or a manual decision is required

USAGE: Run from repo root in PowerShell with gh authenticated.
  .\scripts\ci-remediate.ps1 -RepoOwner 'Yinhang3377' -RepoName 'ArkProtocol-Astra' -Limit 5 -AutoAbortMerge -AutoApplyPatches

This script takes conservative steps and prompts before destructive actions (force-push). Use with care.
#>
param(
    [string]$RepoOwner = 'Yinhang3377',
    [string]$RepoName = 'ArkProtocol-Astra',
    [int]$Limit = 5,
    [switch]$AutoAbortMerge,
    [switch]$AutoApplyPatches
)

$repo = "$RepoOwner/$RepoName"
$logDir = Join-Path $PWD 'gh_logs'
$exDir = Join-Path $logDir 'extracted'
$fixFile = Join-Path $PWD 'ci-fixes.txt'

function Invoke-Safe { param($cmd) Invoke-Expression $cmd }

Write-Host "Starting CI remediation loop for $repo" -ForegroundColor Cyan

# Step 0: abort unfinished merge if requested
if (Test-Path .git\MERGE_HEAD) {
    if ($AutoAbortMerge) {
        Write-Host "Auto-aborting unfinished merge" -ForegroundColor Yellow
        git merge --abort
    } else {
        $ans = Read-Host "Unfinished merge detected. Type 'abort' to abort or 'continue' to leave it";
        if ($ans -eq 'abort') { git merge --abort }
    }
}

# Ensure up-to-date main
Write-Host "Fetching origin/main and updating local main" -ForegroundColor Cyan
Safe-Run 'git fetch origin main'
# Do a hard reset of local main to remote main to ensure we test merges against latest main
Safe-Run 'git checkout main' 2>$null
Safe-Run 'git reset --hard origin/main'

# Switch back to working branch if needed
$branch = (git rev-parse --abbrev-ref HEAD).Trim()
Write-Host "Current branch: $branch" -ForegroundColor Green

# Start the remediation loop
while ($true) {
    # list recent failed runs
    $runsOut = gh run list --repo $repo --limit $Limit --json id,name,head_branch,status,conclusion,html_url 2>$null
    if (-not $runsOut) { Write-Host "gh run list returned no output; ensure gh is authenticated and you have repo access" -ForegroundColor Red; break }
    $runs = $runsOut | ConvertFrom-Json
    $fails = $runs | Where-Object { $_.conclusion -ne $null -and $_.conclusion -ne 'success' }
    if (-not $fails) { Write-Host "No failed runs found. All good." -ForegroundColor Green; break }

    foreach ($r in $fails) {
        Write-Host "Processing failed run $($r.id) ($($r.name)) branch $($r.head_branch) conclusion $($r.conclusion)" -ForegroundColor Yellow
        "Run: $($r.id) $($r.name) $($r.head_branch) $($r.html_url) - conclusion: $($r.conclusion)" | Out-File -Append -FilePath $fixFile

        # download logs
        Remove-Item -Recurse -Force $logDir -ErrorAction SilentlyContinue
        gh run download $($r.id) --repo $repo -D $logDir 2>$null
        try {
            $runLogDir = Join-Path $PWD 'ci_artifacts\gh_runs'
            $match = Get-ChildItem -LiteralPath $runLogDir -Filter "run_$($r.id)*.log" -File -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($match) {
                $scanner = Join-Path $PSScriptRoot 'scan_gh_run_logs.ps1'
                if (Test-Path -LiteralPath $scanner) { & $scanner -LogPath $match.FullName } else { Write-Host "Scanner not found: $scanner" -ForegroundColor Yellow }
            } else {
                Write-Host "No run log found for $($r.id); skipping scanner invocation" -ForegroundColor DarkGray
            }
        } catch {
            Write-Warning "scan_gh_run_logs.ps1 invocation failed: $($_.ToString())"
        }
        Get-ChildItem $logDir -Filter '*.zip' -File -ErrorAction SilentlyContinue | ForEach-Object { Expand-Archive -Path $_.FullName -DestinationPath $exDir -Force }

        $txts = Get-ChildItem $exDir -Recurse -Filter '*.txt' -ErrorAction SilentlyContinue
        if (-not $txts) {
            Write-Host "No textual logs found for run $($r.id); skipping" -ForegroundColor Yellow
            continue
        }

        $cands = $txts | Sort-Object LastWriteTime -Descending | Select-Object -First 5
        foreach ($c in $cands) {
            Write-Host "Inspecting $($c.FullName)" -ForegroundColor Cyan
            $lines = Get-Content $c.FullName -Tail 200 -ErrorAction SilentlyContinue
            $snippet = $lines | Select-Object -Last 20
            $snippet | Out-File -Append -FilePath $fixFile
            $joined = $snippet -join "`n"

            # detect patterns
            if ($joined -match "GPG_PRIVATE_KEY" -or $joined -match "gpg: .*no secret key" -or $joined -match "no secret key") {
                "Detected GPG/secret issue in run $($r.id)" | Out-File -Append -FilePath $fixFile
                "Suggested: set secret GPG_PRIVATE_KEY and optionally GPG_PASSPHRASE via gh secret set or web UI" | Out-File -Append -FilePath $fixFile
                "Example (local):`n$g = Get-Content -Raw 'C:\path\to\gpg.key'; gh secret set GPG_PRIVATE_KEY -b $g --repo $repo" | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "github.event.release.tag_name|github.ref_name|tag .*not found") {
                "Detected tag/ref issue" | Out-File -Append -FilePath $fixFile
                "Suggested: create/push the expected tag (e.g., git tag -a v0.1.0 -m 'v0.1.0' && git push origin v0.1.0)" | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "rustfmt|cargo fmt --check|formatting" ) {
                "Detected formatting issue" | Out-File -Append -FilePath $fixFile
                "Suggested: run 'cargo fmt --all' locally; CI may produce format-fix.patch artifact to review." | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "clippy" -and $joined -match "error: aborting due to") {
                "Detected clippy errors" | Out-File -Append -FilePath $fixFile
                "Suggested: inspect clippy-fix.patch artifact or run 'cargo clippy -p ark-wallet-cli --fix -Z unstable-options' locally." | Out-File -Append -FilePath $fixFile
            }
            if ($joined -match "permission denied|access denied") {
                "Detected permission issue; check GITHUB_TOKEN and workflow permissions." | Out-File -Append -FilePath $fixFile
            }

            # attempt to apply format/clippy patches if present in extracted dir and AutoApplyPatches set
            $patches = Get-ChildItem $exDir -Recurse -Include 'format-fix.patch','clippy-fix.patch' -File -ErrorAction SilentlyContinue
            if ($AutoApplyPatches -and $patches) {
                foreach ($p in $patches) {
                    Write-Host "Found patch artifact $($p.FullName). Attempting to apply..." -ForegroundColor Cyan
                    try {
                        git apply $p.FullName
                        git add -A
                        try {
                            git commit -m "ci: apply patch $($p.Name) from CI triage"
                        } catch {
                            Write-Host "No commit created; maybe no changes" -ForegroundColor Yellow
                        }
                        # force push prompt
                        $ok = Read-Host "Patch applied locally. Force-push branch $branch? (yes/no)"
                        if ($ok -eq 'yes') {
                            git push --force-with-lease origin $branch
                        } else { Write-Host "Skipping push" }
                    } catch {
                        Write-Host "Failed to apply patch $($p.Name): $_" -ForegroundColor Red
                    }
                }
            }

            # after local remediation attempt, optionally rerun the workflow
            $rerun = Read-Host "Rerun this workflow run $($r.id)? (yes/no/skip-all)"
            if ($rerun -eq 'yes') {
                gh run rerun $($r.id) --repo $repo
                Write-Host "Rerun requested for run $($r.id)" -ForegroundColor Green
            } elseif ($rerun -eq 'skip-all') {
                Write-Host "Skipping reruns for remaining runs" -ForegroundColor Yellow
                break 2
            } else {
                Write-Host "Skipping rerun for this run" -ForegroundColor Yellow
            }
        }
    }
    # small pause before next loop
    Start-Sleep -Seconds 6
}

Write-Host "Remediation loop complete. See $fixFile for details." -ForegroundColor Green
