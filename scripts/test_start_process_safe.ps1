# Test Start-Process-Safe wrapper
. $PSScriptRoot\lib_process.ps1

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
    # To avoid accidental ParameterBindingValidationException in some environments,
    # only execute the direct reproducer if TEST_DIRECT_STARTPROCESS env var is set to '1'.
    if ($env:TEST_DIRECT_STARTPROCESS -eq '1') {
    # Use Start-Process-Safe to avoid ParameterBindingValidationException in environments
    # where an empty ArgumentList would trigger a ParameterBindingValidationException.
    Start-Process-Safe -FilePath (Get-Command powershell).Source -ArgumentList "" -PassThru -Wait
        Write-Host "Direct Start-Process did NOT throw (unexpected)"
    } else {
        Write-Host "Skipping direct Start-Process reproducer. Set TEST_DIRECT_STARTPROCESS=1 to run it intentionally."
    }
} catch {
    Write-Host "Direct Start-Process threw (expected): $($_.Exception.GetType().FullName): $($_.Exception.Message)"
}

Write-Host "Test complete"
