# ArkProtocol-Astra

## AES-256-GCM Envelope Helpers

- Added `hot_prepare_envelope` and `hot_decrypt_envelope` for two-phase hot signing.
- Sensitive buffers (keys, nonces) are zeroized using the `zeroize` crate for memory safety.
- Usage: Prepare an envelope with `hot_prepare_envelope`, decrypt with `hot_decrypt_envelope`.
- Next: CLI subcommands for prepare/broadcast (planned).
 - Next: CLI subcommands for prepare/decrypt (implemented in this branch).


Notes

This repository contains a CLI wallet (`crates/ark-wallet-cli`) used for demonstration and tooling. The AES-GCM helpers provide a secure envelope format for transferring signed payloads between offline (hot-sign) and online (broadcaster) devices.

### Usage example

Rust example (prepare -> decrypt):

```rust
use ark_wallet_cli::cli::{hot_prepare_envelope, hot_decrypt_envelope};
use zeroize::Zeroize;

// Prepare envelope on offline hot-sign device
let (envelope_json, ephemeral_key_b64) = hot_prepare_envelope(&tx, mnemonic)
	.expect("prepare failed");

// Transfer `envelope_json` to online broadcaster and deliver `ephemeral_key_b64` via a
// secure channel (QR, or one-click relay when configured).

// Decrypt on broadcaster (or for test)
let signed_tx = hot_decrypt_envelope(&envelope_json, &ephemeral_key_b64)
	.expect("decrypt failed");

// Zeroize ephemeral key after use
let mut ephemeral = ephemeral_key_b64;
ephemeral.zeroize();
```

### Tests & Security

- Unit test `test_hot_envelope_roundtrip` (in `crates/ark-wallet-cli`) verifies AES-GCM envelope round-trip.
- Sensitive data (derived private keys, ephemeral symmetric key and nonces, signed JSON) are zeroized in memory using the `zeroize` crate.

Security and side-channel notes
------------------------------

- Zeroization is a best-effort mitigation. It reduces the window in which sensitive bytes remain in process memory, but it does not protect against all classes of attacks (for example, if the OS swaps memory to disk, a kernel compromise, or if hardware side-channels are present).
- Avoid running this CLI on untrusted or instrumented hosts when handling production keys. Prefer hardware-backed key stores (HSMs or secure elements) when available.
- Be careful with constant-time and side-channel resistant code paths: ECDSA signing libraries used here are considered safe for typical use, but if your threat model includes local attackers with physical access or precise timing capabilities, consider blinding and constant-time primitives / audited cryptography crates.
- Do not rely solely on zeroize to prevent leaks; also harden the execution environment: disable swap, use secure OS configurations, limit process privileges, and prefer ephemeral isolated environments for signing operations.

If you need a higher-assurance setup, consider using specialized memory-safe tooling or dedicated hardware modules and review this code with a security auditor.

See implementation in `crates/ark-wallet-cli/src/cli.rs` for `hot_prepare_envelope`, `hot_decrypt_envelope` and associated tests.

### Example: envelope_demo

An example program is included at `examples/envelope_demo.rs`. It demonstrates preparing an AES-GCM envelope from a hot-sign mnemonic and decrypting it again (test/demo only).

Run the example with:

```bash
cargo run --example envelope_demo --package ark-wallet-cli
```

Note: the example uses a fixed test mnemonic and is intended for local testing only. Do not use this mnemonic in production.

### CLI Usage

- Prepare: `ark-wallet prepare --mnemonic "your mnemonic" --file tx.json --json`
	- Outputs JSON with `envelope` and `key_b64`. Transfer `key_b64` out-of-band (QR) when possible.
- Decrypt: `ark-wallet decrypt --envelope "$(cat envelope.json)" --key-b64 "your-key" --json`
	- Outputs the signed JSON. Always zeroize or securely erase the ephemeral key after use.

Example (run the demo example in this repo):

```bash
cargo run --example envelope_demo --package ark-wallet-cli
```

Security notes:

- The ephemeral symmetric key (base64 `key_b64`) must be treated as highly sensitive. Prefer out-of-band transport (QR) from signer->broadcaster. Including the key in a relay or public transport weakens the threat model.
- The implementation zeroizes derived private keys, the ephemeral symmetric key, nonces, and signed JSON buffers where practical. However, zeroization is a best-effort mitigation — avoid swapping/paging and consider platform-specific secure memory when operating at high threat levels.
- Never paste or transmit `key_b64` over untrusted channels.

#### Quick start (concrete example)

Create a minimal `tx.json` in the current directory:

```json
{
	"to": "addr",
	"amount": 100
}
```

Linux / macOS (bash) example using `jq`:

```bash
# Run prepare once and capture the JSON output
OUT=$(ark-wallet prepare --mnemonic "your mnemonic" --file tx.json --json)
# Extract envelope and key into files
echo "$OUT" | jq -r '.envelope' > envelope.json
echo "$OUT" | jq -r '.key_b64' > key.b64

# Decrypt using the extracted envelope and key
ark-wallet decrypt --envelope "$(cat envelope.json)" --key-b64 "$(cat key.b64)" --json
```

PowerShell example (Windows):

```powershell
# Prepare and save envelope + key to files
$out = ark-wallet prepare --mnemonic "your mnemonic" --file tx.json --json | ConvertFrom-Json
$out.envelope | Out-File -FilePath .\envelope.json -Encoding utf8
$out.key_b64 | Out-File -FilePath .\key.b64 -Encoding utf8

# Decrypt
$env = Get-Content .\envelope.json -Raw
$key = Get-Content .\key.b64 -Raw
ark-wallet decrypt --envelope $env --key-b64 $key --json
```

Notes:

- Prefer transferring `key_b64` by QR or other secure OOB channel rather than storing it on disk.
- Capture the `prepare` JSON once and extract both `envelope` and `key_b64` from that single output (avoid calling `prepare` twice).
