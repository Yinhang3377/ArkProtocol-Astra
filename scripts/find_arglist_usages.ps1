# Find Start-Process with -ArgumentList inside scripts
Write-Host "Scanning scripts for 'Start-Process -ArgumentList'..."
Get-ChildItem -Path .\scripts -Filter *.ps1 -File |
    Select-String -Pattern 'Start-Process\s+-ArgumentList' -AllMatches |
    ForEach-Object { "$($_.Path):$($_.LineNumber): $($_.Line.Trim())" }
Write-Host "Scan complete."