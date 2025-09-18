# ASCII-safe search helper: build Chinese fragments from code points to avoid file encoding problems
# Builds patterns like '参数为 Null', '参数为.*或空', and common English exceptions.
$p1 = -join ([char[]](0x53C2,0x6570,0x4E3A))  # 参数为
$p2 = -join ([char[]](0x6216,0x7A7A))        # 或空
$patterns = @(
    $p1 + ' Null',
    $p1 + '.*' + $p2,
    $p1 + $p2,
    'ArgumentList is null',
    'ArgumentList is empty',
    'ParameterBindingValidationException',
    'ParameterBindingException'
)

foreach ($p in $patterns) {
    Write-Host "=== Pattern: $p ==="
    Get-ChildItem -Path .\ci_artifacts -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            Select-String -Path $_.FullName -Pattern $p -AllMatches -ErrorAction SilentlyContinue | ForEach-Object {
                $line = $_.Line.Trim()
                Write-Host ("{0}:{1}: {2}" -f $_.Path, $_.LineNumber, $line)
            }
        } catch {
            # ignore unreadable files
        }
    }
}
