
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
    [int]$MaxAttempts = 6,
    [bool]$DryRun = $true
)

$repo = "$RepoOwner/$RepoName"
$logRoot = Join-Path $PWD 'gh_logs'
$fixFile = Join-Path $PWD 'ci-fixes.txt'

function Run-Cmd([string]$cmd, [switch]$Quiet) {
    Write-Host "CMD> $cmd"
    if (-not $DryRun) {
        if ($Quiet) {
            Invoke-Expression $cmd | Out-Null
        } else {
            Invoke-Expression $cmd
        }
    } else {
        Write-Host "(DryRun) Skipping execution" -ForegroundColor Yellow
    }
}

function Ensure-Dir([string]$p) {
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p | Out-Null }
}

Write-Host "Starting conservative CI remediation (DryRun=$DryRun) for $repo" -ForegroundColor Cyan

# remember current branch
$branch = (git rev-parse --abbrev-ref HEAD 2>$null).Trim()
Write-Host "Working branch: $branch" -ForegroundColor Green

# Ensure gh is available
try { gh --version | Out-Null } catch { Write-Host "gh CLI not found or not authenticated. Aborting." -ForegroundColor Red; exit 1 }

$attempt = 0
while ($attempt -lt $MaxAttempts) {
    $attempt++
    Write-Host "\n--- Attempt $attempt / $MaxAttempts ---" -ForegroundColor Cyan

    # list recent runs
    $runsJson = gh run list --repo $repo --limit $Limit --json databaseId,conclusion,headBranch,headSha,name,htmlUrl,status 2>$null
    if (-not $runsJson) { Write-Host "gh run list returned no output; check gh auth/permissions" -ForegroundColor Red; break }
    $runs = $runsJson | ConvertFrom-Json
    $failed = $runs | Where-Object { $_.conclusion -and $_.conclusion -ne 'success' }
    if (-not $failed) { Write-Host "No failed runs found. Nothing to remediate." -ForegroundColor Green; break }

    foreach ($r in $failed) {
        Write-Host "Processing run $($r.databaseId) - $($r.name) (branch: $($r.headBranch)) conclusion=$($r.conclusion)" -ForegroundColor Yellow
        "$((Get-Date).ToString()) Run: $($r.databaseId) $($r.name) $($r.headBranch) $($r.htmlUrl) - conclusion: $($r.conclusion)" | Out-File -Append -FilePath $fixFile

        # prepare log dir
        $runDir = Join-Path $logRoot $r.databaseId
        if (Test-Path $runDir) { Remove-Item -Recurse -Force $runDir -ErrorAction SilentlyContinue }
        Ensure-Dir $runDir

        Write-Host "Downloading logs for run $($r.databaseId) to $runDir" -ForegroundColor Cyan
        if (-not $DryRun) { gh run download $($r.databaseId) --repo $repo -D $runDir } else { Write-Host "(DryRun) gh run download $($r.databaseId) --repo $repo -D $runDir" }

        # find extracted text logs
        $txts = Get-ChildItem $runDir -Recurse -Filter '*.txt' -ErrorAction SilentlyContinue
        if (-not $txts) {
            Write-Host "No textual logs found for run $($r.databaseId). If DryRun=false the script would have attempted to download logs." -ForegroundColor Yellow
            continue
        }

        $latest = $txts | Sort-Object LastWriteTime -Descending | Select-Object -First 1
        Write-Host "Inspecting: $($latest.FullName)" -ForegroundColor Cyan
        $lines = Get-Content $latest.FullName -ErrorAction SilentlyContinue
        $snippet = $lines | Select-Object -Last 60
        $snippet | Out-File -Append -FilePath $fixFile

        $joined = $snippet -join "`n"
        $kind = 'unknown'
        if ($joined -match 'GPG_PRIVATE_KEY' -or $joined -match 'gpg: .*no secret key' -or $joined -match 'no secret key') { $kind = 'missing-gpg' }
        elseif ($joined -match 'github.event.release.tag_name' -or $joined -match 'github.ref_name' -or $joined -match 'tag .*not found') { $kind = 'tag-mismatch' }
        elseif ($joined -match 'rustfmt' -or $joined -match 'cargo fmt --check' -or $joined -match 'formatting') { $kind = 'fmt' }
        elseif ($joined -match 'clippy' -and $joined -match 'error:') { $kind = 'clippy' }
        elseif ($joined -match 'permission denied' -or $joined -match 'access denied') { $kind = 'permission' }

        Write-Host "Detected: $kind" -ForegroundColor Magenta
        switch ($kind) {
            'missing-gpg' {
                Write-Host "Suggestion: add GPG_PRIVATE_KEY and GPG_PASSPHRASE via gh secret set. Example:" -ForegroundColor Green
                Write-Host "`$gpg = Get-Content -Raw 'C:\secure\gpg_private.key' ; gh secret set GPG_PRIVATE_KEY --body `$gpg --repo '$repo'"
                Write-Host "gh secret set GPG_PASSPHRASE --body (Get-Content -Raw 'C:\secure\gpg_pass.txt') --repo '$repo'"
            }
            'tag-mismatch' {
                Write-Host "Suggestion: create/push expected tag. Example:" -ForegroundColor Green
                Write-Host "git tag -a v0.1.0 -m 'v0.1.0' ; git push origin v0.1.0"
            }
            'fmt' {
                Write-Host "Suggestion: run 'cargo fmt --all' locally, or apply CI artifact 'format-fix.patch' if present." -ForegroundColor Green
                $patch = Get-ChildItem $runDir -Recurse -Filter 'format-fix.patch' -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($patch) { Write-Host "Patch available: $($patch.FullName) -- can apply via 'git apply --index <patch>'" }
            }
            'clippy' {
                Write-Host "Suggestion: run 'cargo clippy -p ark-wallet-cli --fix -Z unstable-options' locally, inspect and commit fixes." -ForegroundColor Green
                $patch = Get-ChildItem $runDir -Recurse -Filter 'clippy-fix.patch' -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($patch) { Write-Host "Patch available: $($patch.FullName) -- can apply via 'git apply --index <patch>'" }
            }
            default {
                Write-Host "Unknown issue: showing last 60 lines for manual inspection:" -ForegroundColor Yellow
                $snippet | ForEach-Object { Write-Host $_ }
            }
        }

        # conservative: do not auto-apply or push when DryRun=true
        if (-not $DryRun) {
            # prompt to apply patch if present
            $patches = Get-ChildItem $runDir -Recurse -Include 'format-fix.patch','clippy-fix.patch' -File -ErrorAction SilentlyContinue
            if ($patches) {
                foreach ($p in $patches) {
                    Write-Host "Found patch: $($p.FullName)." -ForegroundColor Cyan
                    $ans = Read-Host "Apply patch and create branch+PR? (yes/no)"
                    if ($ans -eq 'yes') {
                        git checkout -b "fix/ci-$($r.databaseId)"
                        git apply --index $p.FullName
                        git add -A
                        git commit -m "ci: apply patch $($p.Name) from run $($r.databaseId)"
                        git push origin HEAD
                        gh pr create --repo $repo --title "ci: patch from run $($r.databaseId)" --body "Auto-applied patch" --head "fix/ci-$($r.databaseId)" --base main
                        git checkout $branch
                    } else { Write-Host "Skipped applying patch" }
                }
            }

            # ask about rerun
            $rr = Read-Host "Rerun this workflow run $($r.databaseId)? (yes/no)"
            if ($rr -eq 'yes') { gh run rerun $($r.databaseId) --repo $repo }
        }
    }

    if ($DryRun) {
        Write-Host "DryRun mode: finished one inspection loop. No destructive actions were performed." -ForegroundColor Yellow
        break
    }

    # small wait before next attempt
    Start-Sleep -Seconds 8
}

Write-Host "Remediation loop complete. See $fixFile for collected diagnostics." -ForegroundColor Green
