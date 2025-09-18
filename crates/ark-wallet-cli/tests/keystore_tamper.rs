use ark_wallet_cli::wallet::keystore;
use base64::Engine;

#[test]
fn tampered_ciphertext_should_fail_decrypt() {
    let privkey = [7u8; 32];
    let pwd = "TestPwd#1";
    let (mut crypto, _nonce) = keystore::encrypt(&privkey, pwd, "pbkdf2", 1000, 0, 0, 0).unwrap();
    // Tamper ciphertext by altering a byte in base64 string (flip a char)
    let mut ct_bytes = base64::engine::general_purpose::STANDARD
        .decode(&crypto.ciphertext)
        .unwrap();
    ct_bytes[0] ^= 0xff;
    crypto.ciphertext = base64::engine::general_purpose::STANDARD.encode(ct_bytes);
    let err = keystore::decrypt(&crypto, pwd).unwrap_err();
    assert!(
        format!("{}", err).to_lowercase().contains("aead")
            || format!("{}", err).to_lowercase().contains("decrypt")
    );
}
