# Download last 10 runs for branch fix-hot-sign-security and scan for Start-Process/ArgumentList errors
$runs = Get-Content ..\ci_artifacts\gh_run_list_all.json | ConvertFrom-Json | Where-Object { $_.headBranch -eq 'fix-hot-sign-security' } | Sort-Object -Property startedAt -Descending | Select-Object -First 10
if(-not $runs){ Write-Host 'No runs found for branch'; exit 0 }
if(-not (Test-Path ..\ci_artifacts\gh_runs)){ New-Item -ItemType Directory -Path ..\ci_artifacts\gh_runs | Out-Null }
# Download last 10 runs for branch fix-hot-sign-security and scan for Start-Process/ArgumentList errors
$runs = Get-Content ..\ci_artifacts\gh_run_list_all.json | ConvertFrom-Json | Where-Object { $_.headBranch -eq 'fix-hot-sign-security' } | Sort-Object -Property startedAt -Descending | Select-Object -First 10
if(-not $runs){ Write-Host 'No runs found for branch'; exit 0 }
if(-not (Test-Path ..\ci_artifacts\gh_runs)){ New-Item -ItemType Directory -Path ..\ci_artifacts\gh_runs | Out-Null }
$summary = @()
foreach($r in $runs){
    $db = $r.databaseId
    $num = $r.number
    $wf = $r.workflowName
    $file = "..\ci_artifacts\gh_runs\run_${db}_${num}.log"
    Write-Host "Downloading run $db (number $num) -> $file"
    gh run view $db --log > $file 2>&1
    $summary += [PSCustomObject]@{ databaseId=$db; number=$num; workflow=$wf; file=$file }
}

# Patterns to look for (include English and Chinese variants)
$patterns = @('ParameterBindingValidationException','Start-Process','ArgumentList','鏃犳硶瀵瑰弬鏁?,'鍙傛暟涓?Null 鎴栫┖')

# Accumulator for matches (avoid using automatic $matches variable)
$scanMatches = @()
foreach($s in $summary){
    $m = Select-String -Path $s.file -Pattern $patterns -SimpleMatch -ErrorAction SilentlyContinue
    if($m){
        foreach($mm in $m){
            $scanMatches += [PSCustomObject]@{ databaseId=$s.databaseId; number=$s.number; workflow=$s.workflow; Path=$mm.Path; LineNumber=$mm.LineNumber; Line=$mm.Line }
        }
    }
}

$outSummary = '..\ci_artifacts\gh_run_errors_summary.txt'
$outIds = '..\ci_artifacts\gh_run_matched_ids.txt'
if($scanMatches.Count -gt 0){
    $scanMatches | Sort-Object databaseId,number,Path,LineNumber | Format-Table -AutoSize | Out-String | Out-File -FilePath $outSummary -Encoding utf8
    $scanMatches | Select-Object -ExpandProperty databaseId -Unique | ForEach-Object { $_ } | Out-File -FilePath $outIds -Encoding utf8
    Write-Host 'Matches found; saved summary to' $outSummary 'and ids to' $outIds
} else {
    'No matches found' | Out-File -FilePath $outSummary -Encoding utf8
    Write-Host 'No matches found; wrote empty summary to' $outSummary
}
Write-Host 'Done'

