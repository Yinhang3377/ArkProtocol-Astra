param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Collect staged PowerShell files from git index
$staged = git diff --cached --name-only --diff-filter=ACM | Where-Object { $_ -match '\.ps1$' }
if(-not $staged){ Write-Host 'No staged PowerShell files'; exit 0 }

# Ensure PSScriptAnalyzer is available
if(-not (Get-Module -ListAvailable -Name PSScriptAnalyzer)){
    Write-Host 'PSScriptAnalyzer not found. Attempting to install module (requires admin privileges).'
    Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force -AllowClobber
}
Import-Module PSScriptAnalyzer -ErrorAction Stop

$failed = $false
foreach($file in $staged){
    Write-Host "Analyzing $file"
    $diag = Invoke-ScriptAnalyzer -Path $file -Recurse -Severity Warning,Error
    if($diag){
        Write-Host ('PSScriptAnalyzer reported issues in ' + $file + ':') -ForegroundColor Yellow
        $diag | Select-Object RuleName,Severity,Message | Format-Table | Out-String | Write-Host
        $failed = $true
    }
    # Search for risky Start-Process usages with empty ArgumentList or variable-built args
    $text = Get-Content -Raw -Encoding UTF8 $file
    if($text -match 'Start-Process\s+-ArgumentList\s+""' -or $text -match 'Start-Process\s+-ArgumentList\s+\$\w+'){
        Write-Host "Found potentially risky Start-Process usage in $file" -ForegroundColor Red
        $failed = $true
    }
}
if($failed){
    Write-Host 'Pre-commit PowerShell lint failed. Fix issues and try again.' -ForegroundColor Red
    exit 1
}
Write-Host 'PowerShell pre-commit checks passed.' -ForegroundColor Green
exit 0