<#
Start-Process-Safe
Provides a single exported helper that mirrors Start-Process but only passes -ArgumentList
when the value is non-null and non-empty. Intended to be dot-sourced by other scripts:

. .\lib_process.ps1
Start-Process-Safe -FilePath '...' -ArgumentList $arg -PassThru -Wait
#>
function Start-Process-Safe {
    param(
        [Parameter(Mandatory=$true)] [string] $FilePath,
        [Parameter(Mandatory=$false)] [object] $ArgumentList,
        [switch] $NoNewWindow,
        [switch] $Wait,
        [switch] $PassThru,
        [string[]] $OtherArgs
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
Export-ModuleMember -Function Start-Process-Safe
