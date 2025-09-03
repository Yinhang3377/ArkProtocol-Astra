use clap::{ArgAction, Parser};
use std::{fs, path::Path, time::Instant};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "ark-node", version, about = "ArkProtocol-Astra Node")]
struct Cli {
    /// 配置文件路径
    #[arg(long, default_value = "config/node.toml")]
    config: String,
    /// 日志级别
    #[arg(long, default_value = "info")]
    log: String,
    /// 观察者模式
    #[arg(long, action = ArgAction::SetTrue)]
    observer: bool,
}

#[derive(Debug, serde::Deserialize)]
struct NodeConfig {
    p2p: P2p,
    rpc: Rpc,
    db: Db,
    genesis: Genesis,
}

#[derive(Debug, serde::Deserialize)]
struct P2p {
    listen_addr: String,
    bootnodes: Vec<String>,
}
#[derive(Debug, serde::Deserialize)]
struct Rpc {
    http: String,
    ws: String,
    grpc: String,
    metrics: String,
    health: String,
}
#[derive(Debug, serde::Deserialize)]
struct Db {
    path: String,
}
#[derive(Debug, serde::Deserialize)]
struct Genesis {
    file: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(cli.log.clone()))
        .with_target(false)
        .compact()
        .init();

    tracing::info!("ArkProtocol-Astra node starting...");
    tracing::info!(config = %cli.config, observer = cli.observer, "loading config");

    let cfg = load_config(&cli.config)?;
    tracing::info!(
        "config loaded: db.path={}, health={}",
        cfg.db.path,
        cfg.rpc.health
    );
    // 使用 p2p/rpc/genesis 字段，避免未使用告警
    tracing::info!(
        p2p_listen = %cfg.p2p.listen_addr,
        bootnodes = cfg.p2p.bootnodes.len(),
        http = %cfg.rpc.http,
        ws = %cfg.rpc.ws,
        grpc = %cfg.rpc.grpc,
        metrics = %cfg.rpc.metrics,
        genesis = %cfg.genesis.file,
        "runtime config"
    );

    // 确保数据库目录存在
    if !Path::new(&cfg.db.path).exists() {
        fs::create_dir_all(&cfg.db.path)?;
        tracing::info!(db=%cfg.db.path, "created db directory");
    }

    // 读取创世文件（占位使用）
    match fs::read_to_string(&cfg.genesis.file) {
        Ok(_) => tracing::info!(genesis=%cfg.genesis.file, "genesis file found"),
        Err(e) => {
            tracing::warn!(genesis=%cfg.genesis.file, error=%e, "genesis file missing or unreadable")
        }
    }

    // 启动健康检查 HTTP（极简实现）
    let health_addr = cfg.rpc.health.clone();
    let health_task = tokio::spawn(async move {
        if let Err(e) = serve_health(&health_addr).await {
            tracing::error!(%health_addr, error=%e, "health server failed");
        }
    });

    // 启动 /metrics（Prometheus 文本协议）
    let metrics_addr = cfg.rpc.metrics.clone();
    let start = Instant::now();
    let version = env!("CARGO_PKG_VERSION");
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };
    let metrics_task = tokio::spawn(async move {
        if let Err(e) = serve_metrics(&metrics_addr, start, version, profile).await {
            tracing::error!(%metrics_addr, error=%e, "metrics server failed");
        }
    });

    tracing::info!("node initialized (stub). Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutdown signal received, cleaning up...");

    // 停止后台任务
    health_task.abort();
    metrics_task.abort();
    let _ = health_task.await;
    let _ = metrics_task.await;

    Ok(())
}

fn load_config(path: &str) -> anyhow::Result<NodeConfig> {
    let raw = fs::read_to_string(path)?;
    let cfg: NodeConfig = toml::from_str(&raw)?;
    Ok(cfg)
}

async fn serve_health(addr: &str) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "health server listening");
    loop {
        let (mut sock, _) = listener.accept().await?;
        tokio::spawn(async move {
            let resp =
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 2\r\n\r\nOK";
            let _ = sock.write_all(resp).await;
            let _ = sock.shutdown().await;
        });
    }
}

async fn serve_metrics(
    addr: &str,
    start: Instant,
    version: &'static str,
    profile: &'static str,
) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "metrics server listening");
    loop {
        let (mut sock, _) = listener.accept().await?;
        let uptime = start.elapsed().as_secs_f64();
        let body = format!(
            "# HELP ark_node_uptime_seconds Node uptime in seconds\n\
             # TYPE ark_node_uptime_seconds gauge\n\
             ark_node_uptime_seconds {}\n\
             # HELP ark_node_build_info Build info\n\
             # TYPE ark_node_build_info gauge\n\
             ark_node_build_info{{version=\"{}\",profile=\"{}\"}} 1\n",
            uptime, version, profile
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        tokio::spawn(async move {
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        });
    }
}
