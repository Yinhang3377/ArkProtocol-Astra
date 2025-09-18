Write-Host "Listing lines that contain '-ArgumentList' in scripts/*.ps1"
Get-ChildItem -Path .\scripts -Filter *.ps1 -File |
    ForEach-Object {
        $path = $_.FullName
        $lines = Get-Content -Path $path
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -match '-ArgumentList') {
                $lineNumber = $i + 1
                Write-Host ($path + ':' + $lineNumber + ': ' + ($lines[$i].Trim()))
            }
        }
    }
Write-Host "Done."