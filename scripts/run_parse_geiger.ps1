#!/usr/bin/env pwsh
# Clean minimal PowerShell-only geiger extractor
Set-StrictMode -Version Latest
$gfile = Join-Path $PSScriptRoot '..\target\geiger\geiger-full.json'
$out = Join-Path $PSScriptRoot '..\target\geiger\extract_manual.txt'
if (-Not (Test-Path $gfile)) { Write-Error "geiger-full.json not found"; exit 4 }
$json = Get-Content $gfile -Raw | ConvertFrom-Json
$packages = $json.packages
$targets = @('getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec')
$lines = @('Geiger manual extract for selected crates:')
foreach ($t in $targets) {
    $entries = $packages | Where-Object { $_.package.id.name -eq $t }
    if (-not $entries) { $lines += ("- {0}: NOT FOUND" -f $t); continue }
    foreach ($ent in $entries) {
        $v = $ent.package.id.version
        $forbids = $ent.unsafety.forbids_unsafe
        $uf = $ent.unsafety.used.functions
        $ue = $ent.unsafety.used.exprs
        $lines += ("- {0} {1} | forbids_unsafe={2} | used.funcs.safe={3} unsafe={4} | used.exprs.safe={5} unsafe={6}" -f $t, $v, $forbids, ($uf.safe -as [int]), ($uf.unsafe_ -as [int]), ($ue.safe -as [int]), ($ue.unsafe_ -as [int]))
    }
}
$lines += ''
$lines += 'Automated recommendations:'
foreach ($t in $targets) {
    $entries = $packages | Where-Object { $_.package.id.name -eq $t }
    if (-not $entries) { $lines += ("- {0}: MISSING" -f $t); continue }
    $max = 0; $vers = @()
    foreach ($ent in $entries) {
        $vers += $ent.package.id.version
        $u_exprs = ($ent.unsafety.used.exprs.unsafe_ -as [int])
        $u_funcs = ($ent.unsafety.used.functions.unsafe_ -as [int])
        $max = [math]::Max($max, $u_exprs)
        $max = [math]::Max($max, $u_funcs)
    }
    if ($max -eq 0) { $lines += ("- {0} {1}: NO used unsafe" -f $t, ($vers -join ',')) } else { $lines += ("- {0} {1}: FOUND used unsafe count={2}" -f $t, ($vers -join ','), $max) }
}
# ensure output dir exists
$outDir = Split-Path $out -Parent
if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }
[System.IO.File]::WriteAllLines($out, $lines, [System.Text.Encoding]::UTF8)
Write-Output ("Wrote: {0}" -f $out)
