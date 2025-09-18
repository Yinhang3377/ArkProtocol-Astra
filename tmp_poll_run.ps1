$repo='Yinhang3377/ArkProtocol-Astra'
$run=17817898636
$timeout=600
$interval=5
$attempts=[int]([math]::Ceiling($timeout/$interval))
Write-Host "Polling run $run for $timeout seconds..."
for ($i=0; $i -lt $attempts; $i++) {
    $r=gh run view $run --repo $repo --json status,conclusion 2>$null | ConvertFrom-Json
    if ($null -eq $r) { Write-Host ("Poll ${i}: no data yet") } else { Write-Host ("Poll ${i}: status=$($r.status) conclusion=$($r.conclusion)") }
    if ($r -and $r.status -ne 'in_progress') {
        Write-Host 'Run completed, fetching logs...'
        gh run view $run --repo $repo --log --exit-status
        break
    }
    Start-Sleep -Seconds $interval
}
