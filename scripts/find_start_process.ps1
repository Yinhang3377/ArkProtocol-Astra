# Find Start-Process usages excluding noisy directories
Write-Host "Searching for Start-Process usages (excluding target, .git, ci_artifacts, data)..."
$excludesRegex = '\\(target|\.git|ci_artifacts|data)(\\|$)'
Get-ChildItem -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { -not ($_.FullName.ToLower() -match $excludesRegex) } |
    Select-String -Pattern 'Start-Process' -SimpleMatch |
    ForEach-Object { "$($_.Path):$($_.LineNumber): $($_.Line.Trim())" }
Write-Host "Search complete."