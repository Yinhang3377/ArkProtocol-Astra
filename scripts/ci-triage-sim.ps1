param([switch]$DryRun, [int]$Limit = 3)

# Simple simulation: emits one fake error with a Chinese message to test DryRun output
$errors = @(
    @{ RunId = 12345; Message = '参数为 Null 或空'; File = 'start.ps1' }
)

if ($DryRun) {
    Write-Host "DryRun 模拟发现 $($errors.Count) 个问题"
    foreach ($e in $errors) {
        Write-Host ("Run {0} → {1} 在 {2}" -f $e.RunId, $e.Message, $e.File)
    }
} else {
    Write-Host '正式扫描中...'
}
