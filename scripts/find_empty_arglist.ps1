# Find Start-Process calls that pass empty string or variable ArgumentList
Write-Host "Scanning scripts for Start-Process with empty ArgumentList literal or variable ArgumentList..."
Get-ChildItem -Path .\scripts -Filter *.ps1 -File |
    Select-String -Pattern "Start-Process\s+-ArgumentList\s+\"\"|Start-Process\s+-ArgumentList\s+'\'\'|Start-Process\s+-ArgumentList\s+\$[a-zA-Z_]\w*" -AllMatches |
    ForEach-Object { "$($_.Path):$($_.LineNumber): $($_.Line.Trim())" }
Write-Host "Scan complete."