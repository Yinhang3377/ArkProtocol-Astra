<#
Scan downloaded GitHub Actions run logs for known PowerShell Start-Process / ArgumentList errors.
Writes concise summary lines to ../ci_artifacts/gh_run_errors_summary.txt in the format:
  Run <id>: <filename>: line <n> - <matched text>

If no matches are found the file will contain the single line:
  No critical failures–
#>

$ErrorActionPreference = 'Continue'

$runsDir = Join-Path -Path $PSScriptRoot -ChildPath '..\ci_artifacts\gh_runs'
$runsDir = (Resolve-Path -LiteralPath $runsDir -ErrorAction SilentlyContinue).ProviderPath
if (-not (Test-Path -LiteralPath $runsDir)) {
    Write-Error "Runs directory not found: $runsDir"
    exit 2
}

$pattern = 'ParameterBindingValidationException|参数为 Null 或空|ArgumentList\s+is\s+null|ArgumentList\s+is\s+empty|ArgumentList\s+is\s+""'
$outLines = [System.Collections.Generic.List[string]]::new()

# collect structured matches by run id
$matchesByRun = @{}

Get-ChildItem -LiteralPath $runsDir -Filter '*.log' -File -Recurse | ForEach-Object {
    $file = $_
    try {
        $content = Get-Content -LiteralPath $file.FullName -Raw -ErrorAction Stop
    } catch {
        # Skip unreadable files
        continue
    }

    if ([string]::IsNullOrEmpty($content)) {
        # empty file, skip
        continue
    }

    $scanMatches = [regex]::Matches($content, $pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    if ($scanMatches.Count -gt 0) {
        foreach ($m in $scanMatches) {
            $prefix = $content.Substring(0, $m.Index)
            $lineNum = ($prefix -split "\r?\n").Count
            $runMatch = [regex]::Match($file.Name, 'run_(\d+)_')
            $runId = if ($runMatch.Success) { $runMatch.Groups[1].Value } else { 'unknown' }
            $outLines.Add("Run ${runId}: $($file.Name): line $lineNum - $($m.Value)")
            if (-not $matchesByRun.ContainsKey($runId)) { $matchesByRun[$runId] = @() }
            $matchesByRun[$runId] += @{ file = $file.Name; path = $file.FullName; line = $lineNum; text = $m.Value }
        }
    }
}

$outPath = Join-Path -Path $PSScriptRoot -ChildPath '..\ci_artifacts\gh_run_errors_summary.txt'
if ($outLines.Count -eq 0) {
    Set-Content -LiteralPath $outPath -Value 'No critical failures -' -Encoding utf8
    Write-Output "Wrote: $outPath (no matches)"
} else {
    $outLines | Set-Content -LiteralPath $outPath -Encoding utf8
    Write-Output "Wrote: $outPath ($($outLines.Count) matches)"
}

# Emit per-run JSON summaries into ../ci_artifacts/
 $artifactsDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\ci_artifacts') -ErrorAction SilentlyContinue).ProviderPath
if (-not (Test-Path -LiteralPath $artifactsDir)) { New-Item -ItemType Directory -Path $artifactsDir | Out-Null }
# determine owner/repo from git remote if possible
$owner = $null; $repoName = $null
try {
    $remoteUrl = (git remote get-url origin) -as [string]
    if ($remoteUrl) {
        # support git@github.com:owner/repo.git and https://github.com/owner/repo.git
        $parts = $remoteUrl -split '[/:]'
        if ($parts.Length -ge 2) {
            $owner = $parts[-2]
            $repoName = ($parts[-1] -replace '\.git$','')
        }
    }
} catch { }

foreach ($rid in $matchesByRun.Keys) {
    $lines = $matchesByRun[$rid]
    # ensure matches dir exists under ci_artifacts/gh_runs/matches
    $matchesDir = Join-Path $artifactsDir 'gh_runs\matches'
    if (-not (Test-Path -LiteralPath $matchesDir)) { New-Item -ItemType Directory -Path $matchesDir -Force | Out-Null }

    # compress matched log files (unique paths)
    $paths = $lines | ForEach-Object { $_.path } | Select-Object -Unique
    if ($paths.Count -gt 0) {
        $zipDst = Join-Path $matchesDir ("run_${rid}.zip")
        try {
            Compress-Archive -Path $paths -DestinationPath $zipDst -Force
        } catch {
            Write-Warning ("Failed to compress matched logs for run {0}: {1}" -f $rid, $_.ToString())
        }
    }

    $logZipUrl = if ($owner -and $repoName) { "https://github.com/$owner/$repoName/actions/runs/$rid.zip" } else { "https://github.com/<owner>/<repo>/actions/runs/$rid.zip" }

    $rec = [ordered]@{
        runId = $rid
        lines = $lines
        logZip = $logZipUrl
        localArchive = if ($paths.Count -gt 0) { $zipDst } else { $null }
    }
    $json = $rec | ConvertTo-Json -Depth 5
    $outJsonFile = Join-Path $artifactsDir ("run_${rid}_summary.json")
    Set-Content -LiteralPath $outJsonFile -Value $json -Encoding utf8
    Write-Output "Wrote JSON summary for run $rid -> $outJsonFile"
    if ($paths.Count -gt 0) { Write-Output "Compressed matched logs for run $rid -> $zipDst" }
}
