use anyhow::Result;

fn main() -> Result<()> {
    // parse argument: amount
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: nova-runner <amount>");
        std::process::exit(2);
    }
    let amount: u64 = args[1].parse().map_err(|_| anyhow::anyhow!("invalid amount"))?;

    let sig = nova_core::bridge::lock(amount)?;
    // we expect cold_sign to print or return signature; print a marker
    if !sig.is_empty() {
        println!("cold_sign: success");
    } else {
        println!("cold_sign: empty signature");
    }
    Ok(())
}
