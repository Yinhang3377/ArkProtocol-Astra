use bip39::Language;
use clap::{ ArgAction, Parser, Subcommand };
use zeroize::{ Zeroize, Zeroizing };

// 引入内部模块
mod wallet;

fn parse_lang(code: &str) -> Language {
    match code {
        "zh" => {
            #[cfg(feature = "zh")]
            {
                Language::SimplifiedChinese
            }
            #[cfg(not(feature = "zh"))]
            {
                Language::English
            }
        }
        _ => Language::English,
    }
}

#[derive(Parser)]
#[command(
    name = "ark-wallet",
    version,
    about = "Ark Wallet CLI",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    /// 以 JSON 输出
    #[arg(global = true, long)]
    json: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 生成助记词（支持英文/中文简体）
    MnemonicNew {
        /// 语言：en 或 zh
        #[arg(long, default_value = "en")]
        lang: String,
        /// 词数：12|15|18|21|24
        #[arg(long, default_value_t = 12)]
        words: usize,
        /// 可选助记词密码
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// 从助记词导入并派生首个地址（占位示例）
    MnemonicImport {
        /// 直接传入助记词（与 --mnemonic-file 二选一）
        #[arg(long)]
        mnemonic: Option<String>,
        /// 从文件读取助记词（与 --mnemonic 二选一）
        #[arg(long)]
        mnemonic_file: Option<String>,
        #[arg(long, default_value = "en")]
        lang: String,
        #[arg(long)]
        passphrase: Option<String>,
        /// BIP32 路径，如 m/44'/7777'/0'/0/0
        #[arg(long, default_value = "m/44'/7777'/0'/0/0")]
        path: String,
        /// 输出完整信息（地址、xpub、xprv、公钥/私钥十六进制）
        #[arg(long, action = ArgAction::SetTrue)]
        full: bool,
    },

    /// Keystore 管理(create/import/export)
    Keystore {
        #[command(subcommand)]
        cmd: KsCmd,
    },
}

#[derive(Subcommand)]
enum KsCmd {
    /// 创建加密 keystore 文件（从助记词派生私钥后加密）
    Create {
        /// 直接传入助记词（与 --mnemonic-file 二选一）
        #[arg(long)]
        mnemonic: Option<String>,
        /// 从文件读取助记词（与 --mnemonic 二选一）
        #[arg(long)]
        mnemonic_file: Option<String>,
        /// 助记词语言：en/zh
        #[arg(long, default_value = "en")]
        lang: String,
        /// 助记词密码（可选）
        #[arg(long)]
        passphrase: Option<String>,
        /// BIP32 路径
        #[arg(long, default_value = "m/44'/7777'/0'/0/0")]
        path: String,

        /// 加密密码（从 --password 或 STDIN/Prompt 三选一）
        #[arg(long, conflicts_with_all = ["password_stdin", "password_prompt"])]
        password: Option<String>,
        /// 从标准输入读取口令：echo "pwd" | ... --password-stdin
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Read password from STDIN",
            conflicts_with_all = ["password", "password_prompt"]
        )]
        password_stdin: bool,
        /// 交互式隐藏输入口令
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Prompt for password interactively",
            conflicts_with_all = ["password", "password_stdin"]
        )]
        password_prompt: bool,
        /// 二次确认口令（需配合 --password-prompt）
        #[arg(
            long,
            action = ArgAction::SetTrue,
            help = "Confirm password (only with --password-prompt)",
            requires = "password_prompt"
        )]
        password_confirm: bool,
        /// 选择 KDF：scrypt|pbkdf2
        #[arg(long, default_value = "scrypt")]
        kdf: String,

        /// PBKDF2 迭代次数（仅 kdf=pbkdf2）
        #[arg(long, default_value_t = 600_000)]
        iterations: u32,

        /// scrypt 参数（仅 kdf=scrypt）：N、r、p
        #[arg(long, default_value_t = 32_768)]
        n: u32,
        #[arg(long, default_value_t = 8)]
        r: u32,
        #[arg(long, default_value_t = 1)]
        p: u32,

        /// 输出文件
        #[arg(long, default_value = "keystore.json")]
        out: String,
        /// 若存在则覆盖
        #[arg(long, action = ArgAction::SetTrue)]
        overwrite: bool,
    },

    /// 导入 keystore（解密校验并打印地址）
    Import {
        #[arg(long)]
        file: String,
        #[arg(long, conflicts_with_all = ["password_stdin", "password_prompt"])]
        password: Option<String>,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = ["password", "password_prompt"]
        )]
        password_stdin: bool,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = ["password", "password_stdin"]
        )]
        password_prompt: bool,
        /// 输出完整信息（含公钥十六进制）
        #[arg(long, action = ArgAction::SetTrue)]
        full: bool,
    },

    /// 导出私钥（解密后输出 hex 或写入文件）
    Export {
        #[arg(long)]
        file: String,
        #[arg(long, conflicts_with_all = ["password_stdin", "password_prompt"])]
        password: Option<String>,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = ["password", "password_prompt"]
        )]
        password_stdin: bool,
        #[arg(
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = ["password", "password_stdin"]
        )]
        password_prompt: bool,
        /// 输出到文件（可选），默认打印 hex
        #[arg(long, num_args = 0..=1, default_missing_value = "privkey.hex")]
        out_priv: Option<String>,
    },
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn read_password_from_stdin() -> anyhow::Result<Zeroizing<String>> {
    use std::io::{ self, Read };
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    let s = s.trim_end_matches(&['\r', '\n'][..]).to_string();
    Ok(Zeroizing::new(s))
}

fn read_password_interactive(prompt: &str) -> anyhow::Result<Zeroizing<String>> {
    use dialoguer::Password;

    // 非 TTY（例如测试/管道）时，回退为从 STDIN 读取一行作为密码
    if !console::Term::stdout().is_term() {
        // 设置 ARK_WALLET_WARN_NO_TTY=1 时输出提示，否则默认静默
        if std::env::var_os("ARK_WALLET_WARN_NO_TTY").is_some() {
            eprintln!("检测到非交互环境：将从 STDIN 读取密码（建议改用 --password-stdin）");
        }
        use std::io::{ self, BufRead };
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        let pw = line.trim_end_matches(&['\r', '\n'][..]).to_string();
        return Ok(Zeroizing::new(pw));
    }

    let prompt_clean = prompt.trim_end_matches(&[':', ' '][..]).to_string();
    let input = Password::new().with_prompt(&prompt_clean).interact()?;
    Ok(Zeroizing::new(input))
}

fn resolve_password(
    pw: Option<String>,
    from_stdin: bool,
    prompt: bool
) -> anyhow::Result<Zeroizing<String>> {
    let sources = (pw.is_some() as u8) + (from_stdin as u8) + (prompt as u8);
    if sources == 0 {
        anyhow::bail!(
            "password is required: provide --password or --password-stdin or --password-prompt"
        );
    }
    if sources > 1 {
        anyhow::bail!(
            "conflicting password sources: use only one of --password/--password-stdin/--password-prompt"
        );
    }
    if let Some(p) = pw {
        return Ok(Zeroizing::new(p));
    }
    if from_stdin {
        return read_password_from_stdin();
    }
    read_password_interactive("Password: ")
}

// 新增：create 专用，支持确认
fn resolve_password_create(
    pw: Option<String>,
    from_stdin: bool,
    prompt: bool,
    confirm: bool
) -> anyhow::Result<Zeroizing<String>> {
    let pwd = resolve_password(pw, from_stdin, prompt)?;
    if confirm {
        if !prompt {
            anyhow::bail!("--password-confirm requires --password-prompt");
        }
        let pwd2 = read_password_interactive("Confirm password: ")?;
        if pwd.as_str() != pwd2.as_str() {
            anyhow::bail!("passwords do not match");
        }
    }
    Ok(pwd)
}

// 新增：校验 KDF 参数
fn validate_kdf(kdf: &str) -> anyhow::Result<()> {
    match kdf {
        "scrypt" | "pbkdf2" => Ok(()),
        other => anyhow::bail!("invalid kdf: {other}. allowed: scrypt, pbkdf2"),
    }
}

fn main() -> anyhow::Result<()> {
    use std::fs;

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::MnemonicNew { lang, words, passphrase } => {
            use bip39::Mnemonic;
            let lang = parse_lang(&lang);
            let entropy_bytes = match words {
                12 => 16,
                15 => 20,
                18 => 24,
                21 => 28,
                24 => 32,
                _ => 16,
            };
            let mut buf = [0u8; 32];
            getrandom::getrandom(&mut buf).map_err(|e| anyhow::anyhow!("getrandom failed: {e}"))?;
            let m = Mnemonic::from_entropy_in(lang, &buf[..entropy_bytes])?;
            let mut seed = m.to_seed(passphrase.as_deref().unwrap_or(""));

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({ "mnemonic": m.to_string(), "lang": format!("{:?}", lang), "seed_len": seed.len() })
                    )?
                );
            } else {
                println!("Mnemonic: {m}");
            }
            // 清理随机缓冲
            let mut zb = buf;
            zb.zeroize();
            // 清理 seed
            seed.zeroize();
        }

        Cmd::MnemonicImport { mnemonic, mnemonic_file, lang, passphrase, path, full } => {
            use bip32::{ DerivationPath, XPrv };
            use sha2::{ Digest, Sha256 };

            let lang = parse_lang(&lang);
            let mut mn_text = if let Some(file) = mnemonic_file {
                fs::read_to_string(file)?.trim().to_string()
            } else if let Some(mn) = mnemonic {
                mn
            } else {
                anyhow::bail!("either --mnemonic or --mnemonic-file is required")
            };
            let m = bip39::Mnemonic::parse_in(lang, &mn_text)?;
            let mut seed = m.to_seed(passphrase.as_deref().unwrap_or(""));
            let dp: DerivationPath = path.parse()?;

            let xprv = XPrv::derive_from_path(seed, &dp)?;
            let pk = xprv.public_key().to_bytes();
            let addr_hash = Sha256::digest(pk);
            let address = bs58::encode(&addr_hash[..20]).into_string();

            if cli.json || full {
                let xpub_str = xprv.public_key().to_string(bip32::Prefix::XPUB);
                let xprv_str = xprv.to_string(bip32::Prefix::XPRV);
                let priv_hex = wallet::keystore::hex_lower(&xprv.private_key().to_bytes());
                let pub_hex = wallet::keystore::hex_lower(&pk);
                let out = if full {
                    serde_json::json!({
                        "address": address, "path": path,
                        "xpub": xpub_str,
                        "xprv": xprv_str.as_str(),
                        "pubkey_hex": pub_hex, "privkey_hex": priv_hex
                    })
                } else {
                    serde_json::json!({ "address": address, "path": path })
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Address: {address}");
            }
            // 清理助记词文本
            mn_text.zeroize();
            // 清理 seed
            seed.zeroize();
        }

        Cmd::Keystore { cmd } => {
            match cmd {
                KsCmd::Create {
                    mnemonic,
                    mnemonic_file,
                    lang,
                    passphrase,
                    path,
                    password,
                    password_stdin,
                    password_prompt,
                    password_confirm,
                    mut kdf,
                    iterations,
                    n,
                    r,
                    p,
                    out,
                    overwrite,
                } => {
                    use std::path::Path;
                    let lang = parse_lang(&lang);
                    let mut mn_text = if let Some(file) = mnemonic_file {
                        fs::read_to_string(file)?.trim().to_string()
                    } else if let Some(mn) = mnemonic {
                        mn
                    } else {
                        anyhow::bail!("either --mnemonic or --mnemonic-file is required")
                    };

                    // 先校验助记词/路径，失败则直接报错，不提示密码
                    let pass = Zeroizing::new(passphrase.unwrap_or_default());
                    let (priv32, pk33, _) = wallet::hd::derive_priv_from_mnemonic(
                        lang,
                        &mn_text,
                        pass.as_str(),
                        &path
                    )?;
                    let address = wallet::address::from_pubkey(&pk33);
                    // 清理助记词文本
                    mn_text.zeroize();

                    // 再校验 KDF与读取口令
                    kdf = kdf.to_lowercase();
                    validate_kdf(&kdf)?;
                    let password = resolve_password_create(
                        password,
                        password_stdin,
                        password_prompt,
                        password_confirm
                    )?;
                    if password.len() < 8 {
                        anyhow::bail!("password too short (min 8 chars)");
                    }

                    let (crypto, _nonce) = wallet::keystore::encrypt(
                        &priv32,
                        password.as_str(),
                        &kdf,
                        iterations,
                        n,
                        r,
                        p
                    )?;
                    // 为 JSON 输出保留派生路径
                    let path_str = path.clone();
                    let ks = wallet::keystore::Keystore {
                        version: wallet::keystore::VERSION,
                        created_at: now_rfc3339(),
                        address,
                        path: Some(path),
                        pubkey_hex: wallet::keystore::hex_lower(&pk33),
                        crypto,
                    };
                    let json = serde_json::to_string_pretty(&ks)?;
                    let p = Path::new(&out);
                    if p.exists() && !overwrite {
                        anyhow::bail!("file exists: {out}. Use --overwrite to replace");
                    }
                    fs::write(p, json)?;
                    if cli.json {
                        let out_json =
                            serde_json::json!({
                            "address": ks.address,
                            "path": path_str,
                            "file": p.to_string_lossy()
                        });
                        println!("{}", serde_json::to_string_pretty(&out_json)?);
                    } else {
                        println!("keystore saved: {}", p.display());
                    }

                    // 擦除私钥
                    {
                        let mut k = priv32;
                        k.zeroize();
                    }
                }

                KsCmd::Import { file, password, password_stdin, password_prompt, full } => {
                    let raw = fs::read_to_string(&file)?;
                    let ks: wallet::keystore::Keystore = serde_json::from_str(&raw)?;
                    if ks.version != wallet::keystore::VERSION {
                        anyhow::bail!("unsupported keystore version: {}", ks.version);
                    }

                    let password = resolve_password(password, password_stdin, password_prompt)?;
                    let priv32 = wallet::keystore::decrypt(&ks.crypto, password.as_str())?;
                    let pk33 = wallet::hd::pubkey_from_privkey_secp256k1(&priv32)?;
                    let address = wallet::address::from_pubkey(&pk33);

                    if full || cli.json {
                        let out =
                            serde_json::json!({
                            "address": address,
                            "path": ks.path,
                            "pubkey_hex": wallet::keystore::hex_lower(&pk33),
                            "keystore_pubkey_hex": ks.pubkey_hex,
                            "match": ks.address == address && ks.pubkey_hex == wallet::keystore::hex_lower(&pk33),
                        });
                        let s = serde_json::to_string_pretty(&out)?;
                        println!("{s}");
                    } else {
                        println!("Address: {address}");
                    }

                    // 擦除私钥
                    {
                        let mut k = priv32;
                        k.zeroize();
                    }
                }

                KsCmd::Export { file, password, password_stdin, password_prompt, out_priv } => {
                    let raw = fs::read_to_string(&file)?;
                    let ks: wallet::keystore::Keystore = serde_json::from_str(&raw)?;
                    if ks.version != wallet::keystore::VERSION {
                        anyhow::bail!("unsupported keystore version: {}", ks.version);
                    }

                    let password = resolve_password(password, password_stdin, password_prompt)?;
                    let priv32 = wallet::keystore::decrypt(&ks.crypto, password.as_str())?;
                    let hex = wallet::keystore::hex_lower(&priv32);
                    if cli.json {
                        if let Some(outp) = out_priv {
                            std::fs::write(outp.as_str(), &hex)?;
                            // 统一输出绝对路径
                            let p = std::path::Path::new(&outp);
                            let abs = if p.is_absolute() {
                                p.to_path_buf()
                            } else {
                                std::env::current_dir()?.join(p)
                            };
                            let out =
                                serde_json::json!({ "privkey_hex": hex, "file": abs.to_string_lossy() });
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        } else {
                            let out = serde_json::json!({ "privkey_hex": hex });
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        }
                    } else if let Some(outp) = out_priv {
                        std::fs::write(outp.as_str(), &hex)?;
                        let p = std::path::Path::new(&outp);
                        // 规范化，避免 macOS /var 与 /private/var 差异
                        let abs = std::fs::canonicalize(&p).unwrap_or_else(|_| {
                            if p.is_absolute() {
                                p.to_path_buf()
                            } else {
                                std::env::current_dir().unwrap().join(p)
                            }
                        });
                        println!("private key hex saved: {}", abs.display());
                    } else {
                        println!("{hex}");
                    }

                    // 擦除私钥
                    {
                        let mut k = priv32;
                        k.zeroize();
                    }
                }
            } // end match cmd
        } // end Cmd::Keystore arm
    } // end match cli.cmd
    Ok(())
}
