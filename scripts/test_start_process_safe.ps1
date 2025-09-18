# Test Start-Process-Safe wrapper
function Start-Process-Safe {
    param(
        [Parameter(Mandatory=$true)] [string] $FilePath,
        [Parameter(Mandatory=$false)] [object] $ArgumentList,
        [switch] $NoNewWindow,
        [switch] $Wait,
        [switch] $PassThru
    )
    $params = @{ FilePath = $FilePath }
    if ($ArgumentList -ne $null) {
        if (-not ([string]::IsNullOrWhiteSpace([string]$ArgumentList))) {
            $params['ArgumentList'] = $ArgumentList
        }
    }
    if ($NoNewWindow.IsPresent) { $params['NoNewWindow'] = $true }
    if ($Wait.IsPresent) { $params['Wait'] = $true }
    if ($PassThru.IsPresent) { $params['PassThru'] = $true }
    return Start-Process @params
}

Write-Host "Running Start-Process-Safe with empty ArgumentList (should not throw)"
try {
    $p = Start-Process-Safe -FilePath (Get-Command powershell).Source -ArgumentList "" -PassThru -Wait
    Write-Host "Safe wrapper returned process id: $($p.Id)"
} catch {
    Write-Host "Safe wrapper threw: $($_.Exception.GetType().FullName): $($_.Exception.Message)"
}

Write-Host "Running direct Start-Process with empty ArgumentList to reproduce original error (should throw)"
try {
    Start-Process -FilePath (Get-Command powershell).Source -ArgumentList "" -PassThru -Wait
    Write-Host "Direct Start-Process did NOT throw (unexpected)"
} catch {
    Write-Host "Direct Start-Process threw (expected): $($_.Exception.GetType().FullName): $($_.Exception.Message)"
}

Write-Host "Test complete"
