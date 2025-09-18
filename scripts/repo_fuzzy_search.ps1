# repo_fuzzy_search.ps1
# Safe repo-wide fuzzy search for '参数', 'ArgumentList', 'Null' and related tokens
$charParam = -join ([char[]](0x53C2,0x6570)) # 参数
$patterns = @(
    $charParam,
    'ArgumentList',
    '\bNull\b',
    'ParameterBindingValidationException',
    'ParameterBindingException'
)
$excludes = @('target', '.git', 'ci_artifacts', 'data')

Write-Host "Searching repository for patterns: $($patterns -join ', ') ...\n"

Get-ChildItem -Recurse -File -ErrorAction SilentlyContinue | Where-Object {
    $fn = $_.FullName.ToLower()
    foreach ($e in $excludes) {
        if ($fn -like "*\\$e\\*" -or $fn -like "*\\$e") { return $false }
    }
    return $true
} | ForEach-Object {
    try {
        Select-String -Path $_.FullName -Pattern $patterns -SimpleMatch -ErrorAction SilentlyContinue | ForEach-Object {
            $line = $_.Line.Trim()
            Write-Host ("{0}:{1}: {2}" -f $_.Path, $_.LineNumber, $line)
        }
    } catch {
        # ignore unreadable/binary files
    }
}

Write-Host "\nSearch complete."
