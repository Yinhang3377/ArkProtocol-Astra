use ark_wallet_cli::cli::{ hot_prepare_envelope, hot_decrypt_envelope };
use zeroize::Zeroize;

fn main() {
    // NOTE: in real usage, `tx` and `mnemonic` must be provided. This demo uses a
    // small inline transaction and a test mnemonic for demonstration only.
    let tx = ark_wallet_cli::cli::Tx {
        nonce: 1,
        to: "demo-addr".to_string(),
        amount: 42,
    };

    let mnemonic =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let (env_json, key_b64) = hot_prepare_envelope(&tx, mnemonic).expect("prepare failed");
    println!("Envelope JSON: {}", env_json);
    println!("Ephemeral key (base64): {}", key_b64);

    // Decrypt (simulating the broadcaster)
    let signed = hot_decrypt_envelope(&env_json, &key_b64).expect("decrypt failed");
    println!("Decrypted SignedTx: {:?}", signed);

    // Zeroize ephemeral key after use
    let mut k = key_b64;
    k.zeroize();
}
