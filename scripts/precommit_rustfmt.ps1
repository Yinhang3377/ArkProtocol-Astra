param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Find staged Rust files
$staged = git diff --cached --name-only --diff-filter=ACM | Where-Object { $_ -match '\.rs$' }
if(-not $staged){ Write-Host 'No staged Rust files'; exit 0 }

Write-Host 'Running cargo fmt --all -- --check'
try{
    # Run cargo fmt --all -- --check to verify formatting
    & cargo fmt --all -- --check 2>&1 | Tee-Object -Variable fmtOut
    if($LASTEXITCODE -ne 0){
        Write-Host 'Formatting issues detected, attempting to auto-fix with cargo fmt --all' -ForegroundColor Yellow
        & cargo fmt --all
        # Stage formatted files
        git add -A
        # Re-run check
        & cargo fmt --all -- --check 2>&1 | Tee-Object -Variable fmtOut2
        if($LASTEXITCODE -ne 0){
            Write-Host 'cargo fmt left formatting issues. Please run cargo fmt locally and re-stage.' -ForegroundColor Red
            exit 1
        }
    }
} catch{
    Write-Host "cargo fmt failed: $_" -ForegroundColor Red
    exit 1
}
Write-Host 'Rust fmt pre-commit checks passed.' -ForegroundColor Green
exit 0