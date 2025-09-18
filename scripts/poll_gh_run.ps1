param(
    [string]$RunId = '17817484494',
    [int]$TimeoutSec = 300
)
$repo = 'Yinhang3377/ArkProtocol-Astra'
$interval = 5
$attempts = [int]([math]::Ceiling($TimeoutSec / $interval))
Write-Host "Polling GH run $RunId in $repo (timeout ${TimeoutSec}s) ..."
for($i=0; $i -lt $attempts; $i++){
    $r = gh run view $RunId --repo $repo --json status,conclusion 2>$null | ConvertFrom-Json
    if($null -eq $r){ Write-Host ('Poll {0}: no data yet' -f $i) } else { Write-Host ('Poll {0}: status={1} conclusion={2}' -f $i, $r.status, $r.conclusion) }
    if($r -and $r.status -ne 'in_progress'){
        Write-Host "Run completed; dumping logs and exit status..."
        gh run view $RunId --repo $repo --log --exit-status
        if($r.conclusion -eq 'success'){
            Write-Host "All files clean, workflows re-triggered"
            exit 0
        } else {
            Write-Host "Workflow conclusion: $($r.conclusion)"
            exit 3
        }
    }
    Start-Sleep -Seconds $interval
}
Write-Host "Timeout waiting for run $RunId"
exit 2
