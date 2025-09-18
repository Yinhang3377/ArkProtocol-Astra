# Safe search helper: construct Chinese string from Unicode code points to avoid encoding/quoting issues
$ch = -join ([char[]] (0x53C2,0x6570,0x4E3A,0x20,0x4E75,0x20))
# Note: 0x4E75 is placeholder for ASCII 'N' not needed; we'll just assemble '参数为 ' + 'Null 或空' dynamically
$chinese = -join ([char[]](0x53C2,0x6570,0x4E3A)) # 参 数 为
$orEmpty = -join ([char[]](0x6216,0x7A7A)) # 或 空
# Build regex: '参数为.*Null' OR '参数为.*空' OR English patterns
$pattern = "(" + [regex]::Escape($chinese) + ".*Null" + ")|(" + [regex]::Escape($chinese) + ".*" + [regex]::Escape($orEmpty) + ")|(ArgumentList is null)|(ArgumentList is empty)"
Write-Host "Searching with regex: $pattern"
Get-ChildItem -Path .\ci_artifacts -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
    try {
        Select-String -Path $_.FullName -Pattern $pattern -AllMatches -ErrorAction SilentlyContinue | ForEach-Object {
            $line = $_.Line.Trim()
            Write-Host "$($_.Path):$($_.LineNumber): $line"
        }
    } catch {
        # ignore files that can't be read as text
    }
}
