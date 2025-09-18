# Start AutoFixPRWatcher job (safe starter)
## Resolve script path dynamically to avoid Unicode/hardcoded path issues
$jobName = 'AutoFixPRWatcher'
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$scriptPathItem = Get-ChildItem -Path $scriptDir -Filter 'auto_fix_pr_ci.ps1' -File -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $scriptPathItem) {
    Write-Host "Script 'auto_fix_pr_ci.ps1' not found in $scriptDir"
    exit 1
}
$scriptPath = $scriptPathItem.FullName

# Stop and remove existing jobs with the same name
$existing = Get-Job -Name $jobName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Stopping and removing existing job(s) for $jobName"
    foreach ($j in $existing) {
        if ($j.State -eq 'Running') { Stop-Job -Id $j.Id -Force -ErrorAction SilentlyContinue }
        Remove-Job -Id $j.Id -Force -ErrorAction SilentlyContinue
    }
}

Start-Sleep -Milliseconds 200
if (-not (Test-Path $scriptPath)) { Write-Host "Script not found: $scriptPath"; exit 1 }

# Start the persistent watcher (PrNumber 27, interval 60s, infinite iterations)
$j = Start-Job -Name $jobName -FilePath $scriptPath -ArgumentList '-PrNumber','27','-IntervalSeconds','60','-MaxIterations','0'
Start-Sleep -Seconds 1
Get-Job -Name $jobName | Select-Object Id,Name,State | Format-List
Write-Host "Logs: ci_artifacts/auto_fix_pr_ci.log and ci_artifacts/gh_runs/"
