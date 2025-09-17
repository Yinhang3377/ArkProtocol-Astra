param(
    [long]$RunId = 17801698782,
    [int]$MaxMinutes = 20,
    [int]$MinIntervalSec = 60,
    [int]$MaxIntervalSec = 90
)

# Ensure artifact dir exists
$artDir = Join-Path -Path $PSScriptRoot -ChildPath "..\ci_artifacts" | Resolve-Path -ErrorAction SilentlyContinue
if (-not $artDir) {
    $artDir = Join-Path -Path $PSScriptRoot -ChildPath "..\ci_artifacts"
    New-Item -ItemType Directory -Path $artDir -Force | Out-Null
}
$artDir = (Get-Item $artDir).FullName

$start = Get-Date
$deadline = $start.AddMinutes($MaxMinutes)
$snapshotCount = 0

Write-Host "Polling run $RunId until $deadline (max $MaxMinutes minutes)"

while ((Get-Date) -le $deadline) {
    $now = Get-Date
    $snapshotFile = Join-Path $artDir "main_run_${RunId}_snapshot_${($now).ToString('yyyyMMdd_HHmmss')}.json"
    $mainJson = Join-Path $artDir "main_run_${RunId}.json"

    # Query run metadata (non-interactive)
    try {
        gh api repos/Yinhang3377/ArkProtocol-Astra/actions/runs/$RunId --jq '.' > $snapshotFile 2>$null
        if (Test-Path $snapshotFile) {
            Copy-Item -Path $snapshotFile -Destination $mainJson -Force
            $snapshotCount++
        } else {
            Write-Host "gh api returned no JSON this iteration"
        }
    } catch {
        Write-Host "gh api call failed: $_"
    }

    # Read status from the saved JSON if present
    if (Test-Path $mainJson) {
        $json = Get-Content $mainJson -Raw | ConvertFrom-Json
        $run = $json.workflow_runs | Select-Object -First 1
        if (-not $run) { $run = $json }
        $status = $run.status
        $conclusion = $run.conclusion
        Write-Host "Run $RunId status=$status conclusion=$conclusion (snapshot #$snapshotCount)"

        if ($status -eq 'completed') {
            $logFile = Join-Path $artDir "main_run_${RunId}.log"
            try {
                Write-Host "Run completed; downloading logs to $logFile"
                gh run view $RunId --log | Out-File -FilePath $logFile -Encoding utf8
            } catch {
                Write-Host "Failed to download logs: $_"
            }

            # Exit loop after completion
            break
        }
    }

    # Sleep random interval between MinIntervalSec and MaxIntervalSec
    $rand = Get-Random -Minimum $MinIntervalSec -Maximum ($MaxIntervalSec + 1)
    Write-Host "Sleeping $rand seconds..."
    Start-Sleep -Seconds $rand
}

if ((Get-Date) -gt $deadline) {
    Write-Host "Deadline reached after $MaxMinutes minutes; exiting."
} else {
    Write-Host "Polling finished; snapshots: $snapshotCount"
}
