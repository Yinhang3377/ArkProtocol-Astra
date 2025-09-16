use anyhow::Result;
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
#[derive(Deserialize, Serialize)]
pub struct Tx {
    pub nonce: u64,
    pub to: String,
    pub amount: u64,
}

/// Signature bytes wrapper
pub type Signature = Vec<u8>;

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
}
