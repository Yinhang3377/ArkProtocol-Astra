//! Ark Wallet CLI 学习注释总览
//! - 使用 clap 解析命令：MnemonicNew、MnemonicImport、Keystore { create/import/export } 等
//! - 助记词与种子：bip39 生成/解析助记词；用 passphrase 生成种子；用 bip32 从种子派生私钥
//! - 地址与公钥：k256 生成公钥，address 模块从公钥派生 20 字节地址并 Base58 编码
//! - 密钥库：keystore 模块将私钥加密（PBKDF2/Scrypt + AES-GCM），导入/导出 JSON
//! - 密码输入：支持 --password、--password-stdin、--password-prompt（互斥），可二次确认
//! - JSON 输出：--json 时输出结构化 JSON；否则打印人类可读文本
//! - 路径与文件：使用 canonicalize 规范化路径，保证跨平台一致性
//!
//! 常用示例：
//! - 生成助记词：`ark-wallet mnemonic new --lang en --words 12`
//! - 导入助记词导出地址/私钥：`ark-wallet mnemonic import --mnemonic "...words..." --full`
//! - 创建 keystore：`ark-wallet keystore create --password-stdin`
//! - 导出私钥到文件（非 JSON 仅打印规范化路径）：`ark-wallet keystore export --file ks.json --password "pwd" --out-priv priv.hex`
//! - JSON 导出（包含 file 与 privkey_hex）：`ark-wallet --json keystore export --file ks.json --password "pwd" --out-priv priv.hex`

use bip39::Language;
use clap::{ArgAction, Parser, Subcommand};
use zeroize::{Zeroize, Zeroizing};

// 引入内部模块
mod security;
mod wallet;

/// 解析命令行传入的语言代码为 bip39::Language。
/// - 仅当启用 feature="zh" 才会真正用到简体中文词表；否则回退到英文，保证可编译。
fn parse_lang(code: &str) -> Language {
    match code {
        "zh" => {
            #[cfg(feature = "zh")]
            {
                Language::SimplifiedChinese
            }
            #[cfg(not(feature = "zh"))]
            {
                // 未启用中文词表时，回退为英文，避免编译/运行期因特性缺失而报错。
                Language::English
            }
        }
        // 默认英文
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

    // 子命令总入口（MnemonicNew / MnemonicImport / Keystore）
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    // 生成助记词（可选择词数、语言；passphrase 作为 BIP39 额外口令参与 seed 计算）
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
    // 从助记词导入：解析助记词 -> 生成种子 -> 解析/应用 BIP32 路径 -> 计算地址
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
    // Create：从助记词派生私钥后，用选定的 KDF（PBKDF2/Scrypt）与 AES-GCM 加密保存到 JSON
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

    // Import：读取 keystore JSON，解密得到私钥，重新计算地址并按需输出
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

    // Export：从 keystore 解密出私钥，打印 hex 或写入文件；--json 时输出结构化 JSON
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

// 返回当前 UTC 时间的 RFC3339 格式字符串（用于 keystore 元数据）
fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

// 从 STDIN 读取完整输入作为密码（用于 --password-stdin）。
// - 去除末尾的 \r\n（Windows）或 \n（类 Unix）
// - 使用 Zeroizing 包装，超出作用域时自动清零内存，降低泄露风险
fn read_password_from_stdin() -> anyhow::Result<Zeroizing<String>> {
    use std::io::{self, Read};
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    // 去除换行（支持 \r\n 和 \n）
    let s = s.trim_end_matches(&['\r', '\n'][..]).to_string();
    Ok(Zeroizing::new(s))
}

// 交互式读取密码（用于 --password-prompt）。
// - 当检测到非交互环境（非 TTY），不会阻塞等待隐藏输入，而是回退为读取一行 STDIN
// - 可通过设置 ARK_WALLET_WARN_NO_TTY=1 打印回退提示，默认静默
fn read_password_interactive(prompt: &str) -> anyhow::Result<Zeroizing<String>> {
    use dialoguer::Password;

    // 非 TTY（例如管道、CI）时，回退为从 STDIN 读取
    if !console::Term::stdout().is_term() {
        // 设置 ARK_WALLET_WARN_NO_TTY=1 时输出提示，否则默认静默
        if std::env::var_os("ARK_WALLET_WARN_NO_TTY").is_some() {
            eprintln!("检测到非交互环境：将从 STDIN 读取密码（建议改用 --password-stdin）");
        }
        use std::io::{self, BufRead};
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        let pw = line.trim_end_matches(&['\r', '\n'][..]).to_string();
        return Ok(Zeroizing::new(pw));
    }

    // 去掉提示结尾的冒号或空格，避免重复显示 "Password: :"
    let prompt_clean = prompt.trim_end_matches(&[':', ' '][..]).to_string();
    let input = Password::new().with_prompt(&prompt_clean).interact()?;
    Ok(Zeroizing::new(input))
}

// 统一解析口令来源：--password / --password-stdin / --password-prompt 三选一。
// - 未提供或同时提供多个来源都会报错，避免歧义。
// - 返回 Zeroizing<String>，调用方负责后续使用。
fn resolve_password(
    pw: Option<String>,
    from_stdin: bool,
    prompt: bool,
) -> anyhow::Result<Zeroizing<String>> {
    // 以位加法统计来源数量（true 视为 1），确保恰好一个来源被选择
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
    // 走到这里说明选择了 --password-prompt
    read_password_interactive("Password: ")
}

// 创建/加密 keystore 场景下的口令解析：在 resolve_password 基础上增加二次确认。
// - 只有当使用 --password-prompt 且传入 --password-confirm 时才进行确认，避免误用。
// - 口令不一致时报错。
fn resolve_password_create(
    pw: Option<String>,
    from_stdin: bool,
    prompt: bool,
    confirm: bool,
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

// 校验 KDF 是否受支持（当前仅 scrypt 与 pbkdf2）。
// - 统一在解析命令后尽早校验，避免进入昂贵流程后才失败。
fn validate_kdf(kdf: &str) -> anyhow::Result<()> {
    match kdf {
        "scrypt" | "pbkdf2" => Ok(()),
        other => anyhow::bail!("invalid kdf: {other}. allowed: scrypt, pbkdf2"),
    }
}

// 主流程：解析命令并执行相应分支。
// - 注意清理敏感数据（随机熵、seed、私钥等），避免留在内存中。
// - JSON 模式下尽量输出结构化信息，便于脚本/测试消费。
fn main() -> anyhow::Result<()> {
    use std::fs;

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::MnemonicNew {
            lang,
            words,
            passphrase,
        } => {
            use bip39::Mnemonic;
            let lang = parse_lang(&lang);
            // BIP39 词数 -> 熵长度映射（12/15/18/21/24 -> 128/160/192/224/256 bit）
            let entropy_bytes = match words {
                12 => 16,
                15 => 20,
                18 => 24,
                21 => 28,
                24 => 32,
                _ => 16,
            };
            // 生成随机熵并构造助记词；生产环境建议使用系统强随机
            let mut buf = [0u8; 32];
            getrandom::getrandom(&mut buf).map_err(|e| anyhow::anyhow!("getrandom failed: {e}"))?;
            let m = Mnemonic::from_entropy_in(lang, &buf[..entropy_bytes])?;
            // 由助记词 + 可选 passphrase 生成种子（用于后续 BIP32 派生）
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
            // 清理随机缓冲与 seed，降低敏感数据驻留风险
            let mut zb = buf;
            zb.zeroize();
            seed.zeroize();
        }

        Cmd::MnemonicImport {
            mnemonic,
            mnemonic_file,
            lang,
            passphrase,
            path,
            full,
        } => {
            use bip32::{DerivationPath, XPrv};
            use sha2::{Digest, Sha256};

            let lang = parse_lang(&lang);
            // 读取助记词来源：文件优先，否则使用命令行参数；均为空时报错
            let mut mn_text = if let Some(file) = mnemonic_file {
                fs::read_to_string(file)?.trim().to_string()
            } else if let Some(mn) = mnemonic {
                mn
            } else {
                anyhow::bail!("either --mnemonic or --mnemonic-file is required")
            };
            let m = bip39::Mnemonic::parse_in(lang, &mn_text)?;
            let mut seed = m.to_seed(passphrase.as_deref().unwrap_or(""));
            // 解析 BIP32 路径并派生扩展私钥；随后得到公钥与演示用地址
            let dp: DerivationPath = path.parse()?;

            let xprv = XPrv::derive_from_path(seed, &dp)?;
            let pk = xprv.public_key().to_bytes();
            // 这里用 Sha256 的前 20 字节做“演示地址”，实际项目可替换为正式地址规则
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
            // 清理敏感数据
            mn_text.zeroize();
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
                    // 优先从文件读取助记词；否则从命令行；缺失时报错
                    let mut mn_text = if let Some(file) = mnemonic_file {
                        fs::read_to_string(file)?.trim().to_string()
                    } else if let Some(mn) = mnemonic {
                        mn
                    } else {
                        anyhow::bail!("either --mnemonic or --mnemonic-file is required")
                    };

                    // 先校验助记词、路径与派生是否可行，失败则立刻报错（不触发密码输入）
                    let pass = Zeroizing::new(passphrase.unwrap_or_default());
                    let (priv32, pk33, _) = wallet::hd::derive_priv_from_mnemonic(
                        lang,
                        &mn_text,
                        pass.as_str(),
                        &path,
                    )?;
                    let address = wallet::address::from_pubkey(&pk33);
                    // 助记词文本不再需要，尽早清理
                    mn_text.zeroize();

                    // KDF 标准化与校验；随后解析/读取口令（可二次确认）
                    kdf = kdf.to_lowercase();
                    validate_kdf(&kdf)?;
                    let password = resolve_password_create(
                        password,
                        password_stdin,
                        password_prompt,
                        password_confirm,
                    )?;
                    // 最低长度约束（示例值：8），可根据安全要求调整
                    if password.len() < 8 {
                        anyhow::bail!("password too short (min 8 chars)");
                    }

                    // 加密并构造 keystore JSON
                    let (crypto, _nonce) = wallet::keystore::encrypt(
                        &priv32,
                        password.as_str(),
                        &kdf,
                        iterations,
                        n,
                        r,
                        p,
                    )?;
                    let path_str = path.clone(); // JSON 输出中保留原始派生路径
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
                        // JSON 模式：输出地址、派生路径与保存的文件名
                        let out_json = serde_json::json!({
                            "address": ks.address,
                            "path": path_str,
                            "file": p.to_string_lossy()
                        });
                        println!("{}", serde_json::to_string_pretty(&out_json)?);
                    } else {
                        // 人类可读模式：简洁提示
                        println!("keystore saved: {}", p.display());
                    }

                    // 擦除私钥（Zeroize）
                    {
                        let mut k = priv32;
                        k.zeroize();
                    }
                }

                KsCmd::Import {
                    file,
                    password,
                    password_stdin,
                    password_prompt,
                    full,
                } => {
                    // 读取并反序列化 keystore；版本不兼容直接拒绝
                    let raw = fs::read_to_string(&file)?;
                    let ks: wallet::keystore::Keystore = serde_json::from_str(&raw)?;
                    if ks.version != wallet::keystore::VERSION {
                        anyhow::bail!("unsupported keystore version: {}", ks.version);
                    }

                    // 解密得到私钥 -> 还原公钥与地址
                    let password = resolve_password(password, password_stdin, password_prompt)?;
                    let priv32 = wallet::keystore::decrypt(&ks.crypto, password.as_str())?;
                    let pk33 = wallet::hd::pubkey_from_privkey_secp256k1(&priv32)?;
                    let address = wallet::address::from_pubkey(&pk33);

                    if full || cli.json {
                        // 打印更多校验信息，包含 keystore 中记录的公钥 hex 与当前计算的是否一致
                        let out = serde_json::json!({
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

                KsCmd::Export {
                    file,
                    password,
                    password_stdin,
                    password_prompt,
                    out_priv,
                } => {
                    // 加载 keystore 并校验版本
                    let raw = fs::read_to_string(&file)?;
                    let ks: wallet::keystore::Keystore = serde_json::from_str(&raw)?;
                    if ks.version != wallet::keystore::VERSION {
                        anyhow::bail!("unsupported keystore version: {}", ks.version);
                    }

                    // 解密得到私钥并格式化为小写十六进制
                    let password = resolve_password(password, password_stdin, password_prompt)?;
                    let priv32 = wallet::keystore::decrypt(&ks.crypto, password.as_str())?;
                    let hex = wallet::keystore::hex_lower(&priv32);

                    if cli.json {
                        if let Some(outp) = out_priv {
                            // 写入文件，同时在 JSON 中返回规范化（绝对）路径与私钥 hex
                            std::fs::write(outp.as_str(), &hex)?;
                            let p = std::path::Path::new(&outp);
                            // 不调用 canonicalize（避免某些环境下失败），但确保为绝对路径
                            let abs = if p.is_absolute() {
                                p.to_path_buf()
                            } else {
                                std::env::current_dir()?.join(p)
                            };
                            let out = serde_json::json!({ "privkey_hex": hex, "file": abs.to_string_lossy() });
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        } else {
                            // 仅返回私钥 hex（不写文件）
                            let out = serde_json::json!({ "privkey_hex": hex });
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        }
                    } else if let Some(outp) = out_priv {
                        // 非 JSON 模式：写入文件，并在 stdout 打印规范化路径（用于管道/测试消费）
                        std::fs::write(outp.as_str(), &hex)?;
                        let p = std::path::Path::new(&outp);
                        // 优先尝试 canonicalize；失败则根据相对/绝对路径构造退路
                        let abs = std::fs::canonicalize(p).unwrap_or_else(|_| {
                            if p.is_absolute() {
                                p.to_path_buf()
                            } else {
                                std::env::current_dir().unwrap().join(p)
                            }
                        });
                        println!("{}", abs.display());
                    } else {
                        // 仅打印 hex 到 stdout
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
