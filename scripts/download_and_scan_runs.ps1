#!/usr/bin/env pwsh
# Download the last 10 GH Actions runs for branch 'fix-hot-sign-security' and scan their logs
# Saves raw logs to ci_artifacts/gh_runs/ and writes a summary to ci_artifacts/gh_run_errors_summary.txt

# Replace file contents with a clean, UTF-8 safe script
# Resolve repository root (parent directory of this scripts folder)
$repoRoot = Split-Path -Parent $PSScriptRoot
$runListPath = Join-Path $repoRoot 'ci_artifacts\gh_run_list_all.json'
if(-not (Test-Path $runListPath)){
    Write-Host "Run list not found: $runListPath"
    Write-Host 'Please run: gh run list --branch fix-hot-sign-security --json databaseId,headBranch,workflowName,startedAt,number,conclusion > ci_artifacts/gh_run_list_all.json'
    exit 0
}

$runs = Get-Content -Path $runListPath -Raw -Encoding utf8 | ConvertFrom-Json | Where-Object { $_.headBranch -eq 'fix-hot-sign-security' } | Sort-Object -Property startedAt -Descending | Select-Object -First 10
if(-not $runs){ Write-Host 'No runs found for branch'; exit 0 }

$ghRunsDir = Join-Path $repoRoot 'ci_artifacts\gh_runs'
if(-not (Test-Path $ghRunsDir)){ New-Item -ItemType Directory -Path $ghRunsDir | Out-Null }

$summary = @()
foreach($r in $runs){
    $db = $r.databaseId
    $num = $r.number
    $wf = $r.workflowName
    $file = Join-Path $ghRunsDir "run_${db}_${num}.log"
    Write-Host "Downloading run $db (number $num) -> $file"
    gh run view $db --log > $file 2>&1
    $summary += [PSCustomObject]@{ databaseId=$db; number=$num; workflow=$wf; file=$file }
}

# Patterns to scan for (English only to avoid encoding pitfalls in scripts)
$patterns = @('ParameterBindingValidationException','Start-Process','ArgumentList','Argument list','Null or empty')

# Accumulate matches
$scanMatches = @()
foreach($s in $summary){
    $m = Select-String -Path $s.file -Pattern $patterns -SimpleMatch -ErrorAction SilentlyContinue
    if($m){
        foreach($mm in $m){
            $scanMatches += [PSCustomObject]@{ databaseId=$s.databaseId; number=$s.number; workflow=$s.workflow; Path=$mm.Path; LineNumber=$mm.LineNumber; Line=$mm.Line }
        }
    }
}

$outSummary = Join-Path $repoRoot 'ci_artifacts\gh_run_errors_summary.txt'
$outIds = Join-Path $repoRoot 'ci_artifacts\gh_run_matched_ids.txt'
if($scanMatches.Count -gt 0){
    $scanMatches | Sort-Object databaseId,number,Path,LineNumber | Format-Table -AutoSize | Out-String | Out-File -FilePath $outSummary -Encoding utf8
    $scanMatches | Select-Object -ExpandProperty databaseId -Unique | ForEach-Object { $_ } | Out-File -FilePath $outIds -Encoding utf8
    Write-Host 'Matches found; saved summary to' $outSummary 'and ids to' $outIds
} else {
    'No matches found' | Out-File -FilePath $outSummary -Encoding utf8
    Write-Host 'No matches found; wrote empty summary to' $outSummary
}

Write-Host 'Done'
