mod common;
mod config;
mod server;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Settings;
use server::{create_app, db};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vespera=info,tower_http=info,axum=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Vespera LightMonitor Server v{}", env!("CARGO_PKG_VERSION"));

    // 初始化数据库（PostgreSQL 优先，自动降级 SQLite）
    tracing::info!("📦 Initializing database...");
    let db_repo = db::init_db().await?;

    // 加载配置
    let settings = Settings::new();
    let bind_addr = settings.bind_address();

    tracing::info!("📡 Starting server on {}", bind_addr);

    // 创建 Axum 应用
    let app = create_app(db_repo);

    // 绑定监听器
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!("✅ Server started successfully");
    tracing::info!("🔗 Health check: http://{}/health", bind_addr);
    tracing::info!("📊 API endpoint: http://{}/api/v1", bind_addr);

    // 启动服务器
    axum::serve(listener, app).await?;

    Ok(())
}
