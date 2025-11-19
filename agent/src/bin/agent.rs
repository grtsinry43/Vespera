use anyhow::Result;
use std::time::Duration;
use sysinfo::System;
use tokio::signal;
use tokio::time::interval;
use tracing::{error, info};
use vespera_agent::{collector::{get_local_ip, NodeInfo}, Config, Reporter, SystemCollector};

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
    let node_uuid = config.get_node_uuid();

    info!(
        "Loaded configuration: uuid={}, name={}, server_url={}, report_interval={}s",
        node_uuid, config.agent.node_name, config.agent.server_url, config.agent.report_interval
    );

    // 构建节点基础信息
    let mut system = System::new_all();
    system.refresh_all();

    let node_info = NodeInfo {
        uuid: node_uuid,
        name: config.agent.node_name.clone(),
        ip_address: get_local_ip(),
        agent_version: env!("CARGO_PKG_VERSION").to_string(),
        os_type: std::env::consts::OS.to_string(),
        os_version: System::long_os_version(),
        cpu_cores: system.cpus().len() as i64,
        total_memory: system.total_memory() as i64,
        tags: config.agent.tags.clone(),
    };

    info!(
        "Node info: OS={} {}, CPU={}cores, Memory={}GB",
        node_info.os_type,
        node_info.os_version.as_deref().unwrap_or("unknown"),
        node_info.cpu_cores,
        node_info.total_memory / 1024 / 1024 / 1024
    );

    // 创建采集器和上报器
    let mut collector = SystemCollector::new(node_info);
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
                let request = collector.collect();

                // 上报数据
                match reporter.report(&request).await {
                    Ok(_) => {
                        info!(
                            "Metrics reported: CPU={:.1}%, Mem={:.1}%, Load={:.2}",
                            request.metrics.cpu_usage,
                            request.metrics.memory_usage,
                            request.metrics.load_1.unwrap_or(0.0)
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
    // 优先从���境变量加载（适用于 Docker）
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
