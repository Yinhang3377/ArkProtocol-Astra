pub mod mpc_backup;
pub mod mpc_recover;

#[cfg(test)]
mod tests {
    use super::mpc_backup::generate_shares;
    use super::mpc_recover::recover_secret;

    #[test]
    fn backup_and_recover_demo() {
        let secret = [1u8; 32];
        let shares = generate_shares(&secret).expect("generate shares");
        assert_eq!(shares.len(), 3);
        let rec = recover_secret(
            &[
                (1, shares[0].clone()),
                (2, shares[1].clone()),
            ]
        ).expect("recover");
        assert!(!rec.is_empty());
    }
}
