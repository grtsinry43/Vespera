use anyhow::Result;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;
use tracing::{error, info};
use vespera_agent::{Config, Reporter, SystemCollector};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Vespera Agent starting...");

    // 加载配置
    let config = load_config()?;
    info!(
        "Loaded configuration: node_id={}, server_url={}, report_interval={}s",
        config.agent.node_id, config.agent.server_url, config.agent.report_interval
    );

    // 创建采集器和上报器
    let mut collector = SystemCollector::new(config.agent.node_id.clone());
    let reporter = Reporter::new(
        config.agent.server_url.clone(),
        config.auth.secret.clone(),
        Duration::from_secs(config.agent.timeout),
        config.agent.retry_attempts,
    );

    // 设置定时器
    let mut ticker = interval(Duration::from_secs(config.agent.report_interval));

    info!("Agent started successfully, beginning metric collection...");

    // 主循环
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // 采集数据
                let metrics = collector.collect();

                // 验证数据
                if let Err(e) = metrics.validate() {
                    error!("Metrics validation failed: {}", e);
                    continue;
                }

                // 上报数据
                match reporter.report(&metrics).await {
                    Ok(_) => {
                        info!(
                            "Metrics reported: CPU={:.1}%, Mem={:.1}%, Disk={:.1}%",
                            metrics.cpu_usage,
                            metrics.memory_usage_percent(),
                            metrics.disk_usage_percent()
                        );
                    }
                    Err(e) => {
                        error!("Failed to report metrics: {}", e);
                    }
                }
            }
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal, exiting...");
                break;
            }
        }
    }

    info!("Agent stopped");
    Ok(())
}

/// 加载配置文件
fn load_config() -> Result<Config> {
    // 优先从环境变量加载（适用于 Docker）
    if std::env::var("VESPERA_SERVER_URL").is_ok() {
        info!("Loading configuration from environment variables");
        return Ok(Config::from_env()?);
    }

    // 尝试从配置文件加载
    let config_paths = ["agent.toml", "/etc/vespera/agent.toml", "config/agent.toml"];

    for path in &config_paths {
        if std::path::Path::new(path).exists() {
            info!("Loading configuration from {}", path);
            return Ok(Config::from_file(path)?);
        }
    }

    // 如果都没有找到，返回错误
    Err(anyhow::anyhow!(
        "No configuration found. Please provide either:\n\
         1. Environment variables (VESPERA_SERVER_URL, VESPERA_SECRET, etc.)\n\
         2. Configuration file (agent.toml, /etc/vespera/agent.toml, or config/agent.toml)"
    ))
}
