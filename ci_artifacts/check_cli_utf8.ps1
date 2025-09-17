$path = 'C:\Users\plant\Desktop\Rust区块链\ArkProtocol-Astra\crates\ark-wallet-cli\src\cli.rs'
$enc = New-Object System.Text.UTF8Encoding($false,$true)
try {
  $bytes = [System.IO.File]::ReadAllBytes($path)
  $enc.GetString($bytes) > $null
  Write-Output "OK: utf8"
  exit 0
} catch {
  Write-Output "DECODE ERROR: $($_.Exception.Message)"
  if ($bytes -ne $null) {
    $len = [Math]::Min($bytes.Length, 200)
    $hex = -join ($bytes[0..($len-1)] | ForEach-Object { ('{0:X2}' -f $_) })
    Write-Output $hex
  }
  exit 2
}