# start_detached_auto_fix.ps1
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$script = Join-Path $scriptDir 'auto_fix_pr_ci.ps1'
if(-not (Test-Path $script)){
    Write-Host "Missing script: $script"
    exit 1
}
$argString = "-NoProfile -ExecutionPolicy Bypass -File `"$script`" -PrNumber 27 -IntervalSeconds 60 -MaxIterations 0"
Write-Host "Starting detached PowerShell with args: $argString"
$psExe = (Get-Command powershell).Source
# Safely call Start-Process: if $argString is empty or whitespace, do not pass -ArgumentList (PowerShell validates it must be non-empty)
if ([string]::IsNullOrWhiteSpace($argString)) {
    Write-Host "Argument list is empty; starting PowerShell without -ArgumentList"
    $proc = Start-Process-Safe -FilePath $psExe -WindowStyle Hidden -PassThru
} else {
    $proc = Start-Process-Safe -FilePath $psExe -ArgumentList $argString -WindowStyle Hidden -PassThru
}
Write-Host "Started process Id=$($proc.Id)"
# Wait a moment and show whether log exists
Start-Sleep -Seconds 1
$log = Join-Path $scriptDir '..\ci_artifacts\auto_fix_pr_ci.log'
if(Test-Path $log){ Write-Host 'log exists, tail:'; Get-Content $log -Tail 40 } else { Write-Host 'no log yet: ' $log }
