$ErrorActionPreference = 'Stop'
if (-not (Test-Path .\tmp_logs)) { New-Item -ItemType Directory -Path .\tmp_logs | Out-Null }

Write-Host "Fetching runs for branch v0.4.8"
$runsJson = gh run list --branch v0.4.8 --limit 50 --json databaseId,workflowName,status,conclusion,createdAt
if (-not $runsJson) { Write-Host 'No runs found for v0.4.8'; exit 0 }
$runs = $runsJson | ConvertFrom-Json

foreach ($r in $runs) {
    $id = $r.databaseId
    $wf = $r.workflowName
    Write-Host "--- Monitoring run $id ($wf) ---"
    $attempt = 0
    while ($true) {
        $infoJson = gh run view $id --json databaseId,status,conclusion --jq '. | {databaseId: .databaseId, status: .status, conclusion: .conclusion}'
        if (-not $infoJson) { Write-Host "Unable to fetch status for run $id"; break }
        $info = $infoJson | ConvertFrom-Json
        Write-Host "Status: $($info.status)  Conclusion: $($info.conclusion)"
        if ($info.status -ne 'in_progress') { break }
        $attempt++
        if ($attempt -gt 180) { Write-Host "Timeout waiting for run $id"; break }
        Start-Sleep -Seconds 10
    }

    $outfile = Join-Path .\tmp_logs ("$id.log")
    Write-Host "Downloading run $id logs to $outfile"
    gh run view $id --log > $outfile

    Write-Host "Scanning log for common failure patterns"
    $found = Select-String -Path $outfile -Pattern 'mkdir -p','\\.sccache','error','failed','Exception','exit code' -SimpleMatch -ErrorAction SilentlyContinue
    if ($found) {
        Write-Host "Potential issues found in $outfile (showing up to 20 matches):"
        Select-String -Path $outfile -Pattern 'mkdir -p','\\.sccache','error','failed','Exception','exit code' -SimpleMatch -List | Select-Object -First 20 | ForEach-Object { Write-Host $_.Line }
    } else {
        Write-Host "No obvious failure patterns found in $outfile"
    }
}

Write-Host "Monitoring complete."