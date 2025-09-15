$manifest = Resolve-Path .\crates\ark-wallet-cli\Cargo.toml
Write-Host "manifest: $($manifest.Path)"
$manifestPath = $manifest.Path
cargo geiger --manifest-path $manifestPath --features "" > .\tmp_logs\geiger_ark_wallet_cli.txt 2>&1
Write-Host "geiger output written to .\\tmp_logs\\geiger_ark_wallet_cli.txt"