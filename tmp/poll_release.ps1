$ErrorActionPreference = 'Stop'
# Poll GitHub Actions for branch v0.4.8 until no runs are in_progress (timeout after ~4 minutes)
$max = 40
$i = 0
if (-not (Test-Path .\tmp)) { New-Item -ItemType Directory -Path .\tmp | Out-Null }
if (-not (Test-Path .\tmp_logs)) { New-Item -ItemType Directory -Path .\tmp_logs | Out-Null }

while ($i -lt $max) {
    $i++
    Write-Host "poll $i"
    $json = gh run list --branch v0.4.8 --limit 50 --json databaseId,workflowName,status,conclusion,createdAt
    if (-not $json) { Write-Host 'no runs found'; break }
    $runs = $json | ConvertFrom-Json
    $inprogress = $runs | Where-Object { $_.status -eq 'in_progress' }
    if (-not $inprogress -or $inprogress.Count -eq 0) {
        Write-Host 'no in_progress runs; breaking'
        break
    }
    Start-Sleep -Seconds 6
}

Write-Host 'done polling; fetching final run list'
$finalJson = gh run list --branch v0.4.8 --limit 50 --json databaseId,workflowName,status,conclusion,createdAt
if (-not $finalJson) { Write-Host 'no final runs found'; exit 0 }
$final = $finalJson | ConvertFrom-Json

foreach ($r in $final) {
    $id = $r.databaseId
    $wf = $r.workflowName
    $status = $r.status
    $conclusion = $r.conclusion
    Write-Host "Run: $id  workflow: $wf  status: $status  conclusion: $conclusion"
    $outfile = Join-Path .\tmp_logs ("$id.log")
    Write-Host "Downloading run $id -> $outfile"
    gh run view $id --log > $outfile

    # Quick scan for errors/failures
    $matches = Select-String -Path $outfile -Pattern 'error','failed','Exception','exit code' -SimpleMatch -List -Quiet:$false -ErrorAction SilentlyContinue
    if ($matches) {
        Write-Host "Found potential issues in $outfile (showing up to 20 matches):"
        Select-String -Path $outfile -Pattern 'error','failed','Exception','exit code' -SimpleMatch -List | Select-Object -First 20 | ForEach-Object { Write-Host $_.Line }
    } else {
        Write-Host "No obvious error lines found in $outfile"
    }
}

Write-Host 'All done.'
