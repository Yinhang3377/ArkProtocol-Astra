use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use aes_gcm::{Aes256Gcm, Nonce}; // AES-GCM
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use secp256k1::{Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Mode for signing: cold (from shards) or hot (from mnemonic)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Cold,
    Hot,
}

impl Mode {
    pub fn from_str(s: &str) -> Result<Mode> {
        match s {
            "cold" => Ok(Mode::Cold),
            "hot" => Ok(Mode::Hot),
            _ => anyhow::bail!("unknown mode: {}", s),
        }
    }
}

/// Simple Tx representation used by CLI signing demo
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    pub nonce: u64,
    pub to: String,
    pub amount: u64,
}

/// Signature bytes wrapper
pub type Signature = Vec<u8>;

/// A signed transaction envelope produced by an offline hot-sign device.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SignedTx {
    pub tx: Tx,
    /// hex-encoded signature bytes
    pub signature: String,
}

/// AES-GCM envelope carrying the ciphertext and nonce, both base64 encoded.
#[derive(Serialize, Deserialize, Debug)]
pub struct Envelope {
    pub nonce: String,
    pub ciphertext: String,
}

/// Public unified sign interface used by CLI: sign(tx, mode, args...) -> Signature
pub fn sign(
    tx: &Tx,
    mode: Mode,
    shards: Option<(&[u8], &[u8])>,
    mnemonic: Option<&str>,
) -> Result<Signature> {
    let msg = serde_json::to_vec(tx)?;
    match mode {
        Mode::Cold => {
            let (s1, s2) =
                shards.ok_or_else(|| anyhow::anyhow!("shards required for cold mode"))?;
            // local demo of combining shards -> derive 32-byte key -> sign
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(s1);
            hasher.update(s2);
            hasher.update(&msg);
            let key32 = hasher.finalize();
            let sk = SecretKey::from_slice(&key32).map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let secp = Secp256k1::signing_only();
            let m = Message::from_slice(&key32[..32])?;
            let s = secp.sign_ecdsa(&m, &sk);
            let sig = s.serialize_der().to_vec();
            Ok(sig)
        }
        Mode::Hot => {
            let m = mnemonic.ok_or_else(|| anyhow::anyhow!("mnemonic required for hot mode"))?;
            // derive private key from mnemonic (demo uses wallet::hd convenience)
            let (priv32, _pk33, _path) = crate::wallet::hd::derive_priv_from_mnemonic(
                bip39::Language::English,
                m,
                "",
                "m/44'/7777'/0'/0/0",
            )?;
            let sig = {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&msg);
                let digest = hasher.finalize();
                let sk =
                    SecretKey::from_slice(&priv32).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let secp = Secp256k1::signing_only();
                let m = Message::from_slice(&digest[..32])?;
                let s = secp.sign_ecdsa(&m, &sk);
                s.serialize_der().to_vec()
            };
            // zeroize private key in memory
            let mut k = priv32;
            k.zeroize();
            Ok(sig)
        }
    }
}

/// Prepare an AES-256-GCM envelope containing the signed JSON for the given
/// transaction using the provided mnemonic (hot mode). Returns a tuple of
/// (envelope_json, ephemeral_key_b64). The ephemeral key is a 32-byte random
/// symmetric key which should be transferred securely (e.g. via QR) to the
/// online broadcaster so it can decrypt and broadcast the signed payload.
pub fn hot_prepare_envelope(tx: &Tx, mnemonic: &str) -> Result<(String, String)> {
    // Produce signature using existing hot flow (this zeroizes derived key inside)
    let sig_bytes = sign(tx, Mode::Hot, None, Some(mnemonic))?;

    // Build SignedTx
    let signed = SignedTx {
        tx: tx.clone(),
        signature: hex::encode(&sig_bytes),
    };
    let signed_json = serde_json::to_vec(&signed)?;

    // Generate ephemeral AES-256 key and nonce
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    // Encrypt with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), signed_json.as_ref())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Build envelope (base64 encode nonce and ciphertext)
    let env = Envelope {
        nonce: general_purpose::STANDARD.encode(nonce),
        ciphertext: general_purpose::STANDARD.encode(&ct),
    };
    let env_json = serde_json::to_string(&env)?;

    // Export ephemeral key as base64 for QR transfer
    let key_b64 = general_purpose::STANDARD.encode(key);

    // Zeroize sensitive buffers
    // signed_json is local (Vec<u8>) - zeroize in place
    let mut signed_json_mut = signed_json;
    signed_json_mut.zeroize();
    key.zeroize();
    nonce.zeroize();

    Ok((env_json, key_b64))
}

/// Decrypt an envelope JSON using the provided ephemeral key (base64). Returns
/// the embedded SignedTx structure.
///
/// Note: currently this helper is retained for the two-phase offline/online
/// hot-sign flow (prepare -> transfer key -> decrypt & broadcast). The
/// CLI's one-click relay mode posts the key to the relay and doesn't call
/// this locally; keep the function available for manual/QR flows and tests.
#[allow(dead_code)]
pub fn hot_decrypt_envelope(envelope_json: &str, key_b64: &str) -> Result<SignedTx> {
    let env: Envelope = serde_json::from_str(envelope_json)?;
    let nonce = general_purpose::STANDARD.decode(env.nonce)?;
    let ct = general_purpose::STANDARD.decode(env.ciphertext)?;
    let key = general_purpose::STANDARD.decode(key_b64)?;

    if key.len() != 32 {
        anyhow::bail!("ephemeral key must be 32 bytes");
    }
    if nonce.len() != 12 {
        anyhow::bail!("nonce must be 12 bytes for AES-GCM");
    }

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ct.as_ref())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    // parse SignedTx
    let signed: SignedTx = serde_json::from_slice(&plaintext)?;

    // zeroize key material
    // key and nonce local Vec<u8> will be dropped; attempt to zeroize if possible
    // convert to mutable and zeroize
    let mut key_mut = key;
    key_mut.zeroize();
    // nonce is local Vec<u8>
    // best effort to zeroize
    let mut nonce_mut = nonce;
    nonce_mut.zeroize();

    Ok(signed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cold_sign_from_shards() {
        // create two dummy shard files (in-memory)
        let s1 = b"shard-one-contents";
        let s2 = b"shard-two-contents";
        let tx = Tx {
            nonce: 1,
            to: "addr".to_string(),
            amount: 100,
        };
        let sig = sign(&tx, Mode::Cold, Some((s1.as_ref(), s2.as_ref())), None)
            .expect("cold sign failed");
        println!("cold sig len={}", sig.len());
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_hot_sign_from_mnemonic() {
        // use a fixed mnemonic (for test only)
        let mnemonic =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let tx = Tx {
            nonce: 2,
            to: "addr2".to_string(),
            amount: 200,
        };
        let sig = sign(&tx, Mode::Hot, None, Some(mnemonic)).expect("hot sign failed");
        println!("hot sig len={}", sig.len());
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_hot_envelope_roundtrip() {
        let mnemonic =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let tx = Tx {
            nonce: 42,
            to: "envelope-addr".to_string(),
            amount: 999,
        };
        let (env_json, key_b64) = hot_prepare_envelope(&tx, mnemonic).expect("prepare envelope");
        assert!(!env_json.is_empty());
        assert!(!key_b64.is_empty());

        let signed = hot_decrypt_envelope(&env_json, &key_b64).expect("decrypt envelope");
        assert_eq!(signed.tx, tx);
        // signature should be hex string decoding to non-empty bytes
        let sig_bytes = hex::decode(&signed.signature).expect("sig hex");
        assert!(!sig_bytes.is_empty());
    }
}
