use ark_wallet_cli::cli::{ hot_prepare_envelope, hot_decrypt_envelope, Tx };
use zeroize::Zeroize;

#[test]
fn test_integration_prepare_decrypt() {
    // Use a simple tx payload for the integration test
    let tx = Tx {
        nonce: 100,
        to: "integration-addr".to_string(),
        amount: 12345,
    };

    // Use a test mnemonic (do NOT use in production)
    let mnemonic =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let (env_json, key_b64) = hot_prepare_envelope(&tx, mnemonic).expect("prepare failed");
    assert!(!env_json.is_empty());
    assert!(!key_b64.is_empty());

    let signed = hot_decrypt_envelope(&env_json, &key_b64).expect("decrypt failed");
    assert_eq!(signed.tx, tx);

    // Zeroize ephemeral key after use
    let mut kb = key_b64;
    kb.zeroize();
}
