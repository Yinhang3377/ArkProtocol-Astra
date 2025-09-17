param(
  [int]$maxRuns = 50,
  [string]$repo = "Yinhang3377/ArkProtocol-Astra"
)

$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) {
  Write-Error "gh CLI not found in PATH. Install and `gh auth login` first."
  exit 2
}

$ghRunsJson = "ci_artifacts/gh_run_list_full.json"
$ghRunsLogDir = "ci_artifacts/gh_runs"
if (-not (Test-Path $ghRunsLogDir)) { New-Item -ItemType Directory -Path $ghRunsLogDir | Out-Null }

Write-Host "Listing last $maxRuns runs for $repo"
# Use gh to list runs as JSON
gh run list --repo $repo --limit $maxRuns --json databaseId,headSha,headBranch,conclusion,status,name,createdAt,workflowName > $ghRunsJson
if ($LASTEXITCODE -ne 0) { Write-Error "gh run list failed"; exit 3 }

$summary = @()
$runs = Get-Content $ghRunsJson | Out-String | ConvertFrom-Json
foreach ($r in $runs) {
  $id = $r.databaseId
  $branch = $r.headBranch
  $name = $r.workflowName
  $conclusion = $r.conclusion
  $status = $r.status
  if ($conclusion -ne "success") {
    $outFile = Join-Path $ghRunsLogDir "run_$id.log"
    Write-Host "Downloading run $id ($name) branch=$branch conclusion=$conclusion status=$status -> $outFile"
    gh run view $id --repo $repo --log > $outFile 2>&1
    if ($LASTEXITCODE -ne 0) { Write-Warning "failed to download logs for run $id" }
    $summary += @{ id = $id; branch=$branch; name=$name; conclusion=$conclusion; file=$outFile }
  }
}

$errorsFile = Join-Path $ghRunsLogDir "errors_extracted.txt"
"" | Out-File $errorsFile -Encoding utf8
foreach ($s in $summary) {
  Add-Content $errorsFile "==== $($s.file) ===="
  # extract lines that look like errors or rustfmt diffs
  Select-String -Path $s.file -Pattern "error:|panic!|could not compile|stream did not contain valid UTF-8|##\[error\]|FAILED|failed" -SimpleMatch | ForEach-Object {
    Add-Content $errorsFile $_.Line
  }
}

Write-Host "Downloaded $($summary.Count) non-success runs. Errors written to $errorsFile"
exit 0
