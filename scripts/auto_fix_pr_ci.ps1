<#
Auto CI fixer for PRs — conservative automated repairs
- Detects failed GH Actions runs for a PR/branch
- Downloads run logs to ci_artifacts/gh_runs/
- Looks for rustfmt/clippy/UTF-8 patterns
- Tries conservative fixes:
  * cargo fmt --all
  * cargo fix (non --clippy)
  * convert textual source files (.rs,.toml,.json,.md) to UTF-8 if non-UTF8
- If fixes change files, commits and pushes the branch (force-with-lease not used; standard push)

Usage:
  .\auto_fix_pr_ci.ps1 [-PrNumber <int>] [-IntervalSeconds <int>] [-MaxIterations <int>]

Requires: git, gh, cargo on PATH and authenticated gh (gh auth login)
#>
param(
    [int]$PrNumber = 0,
    [int]$IntervalSeconds = 60,
    [int]$MaxIterations = 0
)

Set-StrictMode -Version Latest
# Prefer using an explicit environment-provided REPO_ROOT when available. This
# avoids git/cmd encoding issues on Windows where `git rev-parse` may return
# paths in an encoding that PowerShell misinterprets. If REPO_ROOT is not set,
# fall back to `git rev-parse` or the script directory.
if ($env:REPO_ROOT) {
    $RepoRoot = $env:REPO_ROOT
} else {
    $gitTop = git rev-parse --show-toplevel 2>$null
    if($LASTEXITCODE -eq 0 -and $gitTop){
        $RepoRoot = $gitTop.Trim()
    } else {
        # fallback to script parent dir
        $RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
    }
}
# Use LiteralPath to avoid PowerShell interpreting special characters in paths.
Push-Location -LiteralPath $RepoRoot

Function Log([string]$s){
    $ts = (Get-Date).ToString('s')
    $line = "[$ts] $s"
    Write-Host $line
        $logdir = Join-Path -Path $RepoRoot -ChildPath "ci_artifacts"
    if(-not (Test-Path $logdir)){ New-Item -ItemType Directory -Path $logdir | Out-Null }
        $logfile = Join-Path -Path $logdir -ChildPath "auto_fix_pr_ci.log"
    Add-Content -Path $logfile -Value $line
}

Function Ensure-GH(){
    try{ Get-Command gh -ErrorAction Stop > $null; return $true } catch { return $false }
}

# Safe Start-Process wrapper: only pass -ArgumentList when non-empty.
function Start-Process-Safe {
    param(
        [Parameter(Mandatory=$true)] [string] $FilePath,
        [Parameter(Mandatory=$false)] [object] $ArgumentList,
        [switch] $NoNewWindow,
        [switch] $Wait,
        [switch] $PassThru
    )
    $params = @{ FilePath = $FilePath }
    if ($ArgumentList -ne $null) {
        # treat empty string or empty array as 'no args'
        if (-not ([string]::IsNullOrWhiteSpace([string]$ArgumentList))) {
            $params['ArgumentList'] = $ArgumentList
        }
    }
    if ($NoNewWindow.IsPresent) { $params['NoNewWindow'] = $true }
    if ($Wait.IsPresent) { $params['Wait'] = $true }
    if ($PassThru.IsPresent) { $params['PassThru'] = $true }
    Start-Process @params
}

if(-not (Ensure-GH)){
    Log "gh CLI not found in PATH. Aborting."
    Exit 2
}

# Determine PR number and branch
if($PrNumber -eq 0){
    try{
        $prJson = gh pr view --json number,headRefName -q ".{number,headRefName}" 2>$null | ConvertFrom-Json
        if($prJson -ne $null -and $prJson.number){
            $PrNumber = [int]$prJson.number
            $Branch = $prJson.headRefName
        }
    } catch {
        # fallback to local branch name
        $Branch = (git rev-parse --abbrev-ref HEAD).Trim()
    }
} else {
    try{ $prJson = gh pr view $PrNumber --json headRefName -q ".headRefName" 2>$null; $Branch = $prJson } catch { $Branch = (git rev-parse --abbrev-ref HEAD).Trim() }
}

if(-not $Branch){
    Log "Could not determine branch for PR. Exiting."
    Exit 3
}

Log "Watching PR #$PrNumber on branch '$Branch' (interval ${IntervalSeconds}s). MaxIterations=$MaxIterations"

$processedRuns = @{}
$iter = 0
while($MaxIterations -eq 0 -or $iter -lt $MaxIterations){
    $iter++
    Log ("Iteration {0}: listing recent runs for branch {1}" -f $iter, $Branch)
    try{
        $runsJson = gh run list --branch $Branch --limit 50 --json databaseId,id,name,status,conclusion -q . 2>$null | ConvertFrom-Json
    } catch {
        # fallback: parse plain output
        Log "gh run list JSON parse failed. Trying plain list"
        $plain = gh run list --branch $Branch --limit 50 2>$null
        $runsJson = @()
    }

    foreach($run in $runsJson){
        $runId = $run.id
        $dbId = $run.databaseId
        $conclusion = $run.conclusion
        $status = $run.status
        if($conclusion -ne 'success' -and -not $processedRuns.ContainsKey($runId)){
            Log "Found non-success run id=$runId status=$status conclusion=$conclusion"
            $outdir = Join-Path $RepoRoot 'ci_artifacts\gh_runs'
            if(-not (Test-Path $outdir)){ New-Item -ItemType Directory -Path $outdir | Out-Null }
            $logfile = Join-Path $outdir "run_${dbId}_${runId}.log"
            Log "Downloading logs to $logfile"
            gh run view $runId --log > $logfile 2>&1
            # mark processed
            $processedRuns[$runId] = $true

            # analyze log for patterns
            $logText = Get-Content $logfile -Raw -ErrorAction SilentlyContinue
            if(-not $logText){
                Log "Log file empty or not readable: $logfile"
                continue
            }

            $needsCommit = $false
            $actionsTaken = @()

            if($logText -match "error: rustfmt.*" -or $logText -match "warning: formatting" -or $logText -match "rustfmt"){
                Log "Detected rustfmt-related failure. Running 'cargo fmt --all'"
                Start-Process-Safe -FilePath cargo -ArgumentList 'fmt','--all' -NoNewWindow -Wait
                $actionsTaken += 'cargo fmt'
                $needsCommit = $true
            }

            if($logText -match "clippy" -or $logText -match "warning:" -and $logText -match "clippy"){
                Log "Detected clippy-related messages. Running 'cargo fix' (conservative)"
                # cargo fix without --clippy is conservative; may apply suggestions
                Start-Process-Safe -FilePath cargo -ArgumentList 'fix' -NoNewWindow -Wait
                $actionsTaken += 'cargo fix'
                $needsCommit = $true
            }

            # Detect UTF-8 decode errors
            if($logText -match "stream did not contain valid UTF-8" -or $logText -match "invalid UTF-8"){
                Log "Detected UTF-8 decoding issue in logs. Attempting to re-encode known textual source files to UTF-8 (rs/toml/json/md)."
                $textPatterns = @('*.rs','*.toml','*.json','*.md')
                foreach($pat in $textPatterns){
                    Get-ChildItem -Path $RepoRoot -Recurse -Include $pat -File -ErrorAction SilentlyContinue | ForEach-Object {
                        $file = $_.FullName
                        try{
                            $bytes = [System.IO.File]::ReadAllBytes($file)
                            $enc = $null
                            try{ [System.Text.Encoding]::UTF8.GetString($bytes) > $null; $enc = 'utf8' } catch { $enc = $null }
                            if(-not $enc){
                                # read as default and rewrite as UTF8
                                Log "Converting $file to UTF-8"
                                $content = Get-Content -Path $file -Encoding Default -Raw
                                Set-Content -Path $file -Value $content -Encoding utf8
                                $actionsTaken += "reencode:$file"
                                $needsCommit = $true
                            }
                        } catch {
                            Log ("Failed to inspect/convert {0}: {1}" -f $file, $_)
                        }
                    }
                }
            }

            # If we made changes, commit and push
            if($needsCommit){
                # show git diff for logging
                $changedFile = Join-Path $RepoRoot 'ci_artifacts\auto_fix_changed_files.txt'
                git --no-pager diff --name-only | Out-File -FilePath $changedFile -Encoding utf8
                $changed = Get-Content $changedFile -ErrorAction SilentlyContinue
                if($changed -and $changed.Count -gt 0){
                    Log ("Changes to commit: {0}" -f ($changed -join ', '))
                    git add -A
                    $msg = ("ci(auto-fix): apply automated fixes from run {0}/{1}: {2}" -f $dbId, $runId, ($actionsTaken -join ', '))
                    git commit -m $msg
                    if($LASTEXITCODE -eq 0){
                        Log "Committed auto-fixes. Attempting to push branch $Branch"
                        git push origin HEAD:$Branch
                        if($LASTEXITCODE -eq 0){
                            Log "Pushed fixes to origin/$Branch"
                        } else {
                            Log "Push failed with exit code $LASTEXITCODE"
                        }
                    } else {
                        Log "Nothing to commit or commit failed. exit=$LASTEXITCODE"
                    }
                } else {
                    Log "No changed files detected after fixes. Skipping commit."
                }
            } else {
                Log "No automatic fixes applied for run $runId"
            }

        }
    }

    Log "Sleeping for $IntervalSeconds seconds before next check..."
    Start-Sleep -Seconds $IntervalSeconds
}

Pop-Location
Log "Watcher exiting after iterations"
