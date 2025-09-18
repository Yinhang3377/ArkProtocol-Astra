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
            $matchesByRun[$runId] += @{ file = $file.Name; line = $lineNum; text = $m.Value }
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

foreach ($rid in $matchesByRun.Keys) {
    $rec = [ordered]@{
        runId = $rid
        lines = $matchesByRun[$rid]
        # construct a reasonable logZip link placeholder (adjust repo path if needed)
        logZip = "https://github.com/<owner>/<repo>/actions/runs/$rid.zip"
    }
    $json = $rec | ConvertTo-Json -Depth 5
    $outJsonFile = Join-Path $artifactsDir ("run_${rid}_summary.json")
    Set-Content -LiteralPath $outJsonFile -Value $json -Encoding utf8
    Write-Output "Wrote JSON summary for run $rid -> $outJsonFile"
}
