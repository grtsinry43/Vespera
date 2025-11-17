mod app;
mod db;
mod error;
mod handlers;
mod middleware;
mod routes;
mod state;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use app::create_app;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db::init_db;

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

    // 初始化数据库
    tracing::info!("📦 Initializing database...");
    let db_repo = init_db().await?;

    // 获取绑定地址（从环境变量或默认值）
    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

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
