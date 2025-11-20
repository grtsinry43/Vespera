mod alert;
mod app;
mod cleanup;
mod db;
mod error;
mod handlers;
mod health_check;
mod middleware;
mod openapi;
mod routes;
mod state;
mod utils;
mod ws;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use app::create_app;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db::init_db;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载 .env 文件 (开发环境使用，生产环境应使用系统环境变量)
    // 尝试从多个位置加载 .env 文件
    if let Err(_) = dotenvy::dotenv() {
        // 如果当前目录没有 .env，尝试从 server 子目录加载
        let server_env = std::path::Path::new("server/.env");
        if server_env.exists() {
            dotenvy::from_path(server_env).ok();
        }
    }

    // 初始化日志系统
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vespera=info,tower_http=info,axum=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("================================================");
    tracing::info!(
        "  Vespera LightMonitor Server v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("================================================");

    // 初始化数据库
    let db_repo = init_db().await?;

    // ============================================
    // 首次启动检查:创建管理员
    // ============================================
    let user_count = db_repo.users().count_users().await?;

    if user_count == 0 {
        if let (Ok(username), Ok(password)) = (
            std::env::var("INITIAL_ADMIN_USERNAME"),
            std::env::var("INITIAL_ADMIN_PASSWORD"),
        ) {
            tracing::info!("No users found. Creating initial admin...");

            let password_hash = crate::utils::hash_password(&password)
                .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

            db_repo
                .users()
                .create_user(&username, None, Some(&password_hash), "admin")
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create admin user: {}", e))?;

            tracing::warn!("Initial admin '{}' created successfully!", username);
            tracing::warn!(
                "  Please remove INITIAL_ADMIN_* environment variables and change the password!"
            );
        } else {
            tracing::error!(" No users found in database!");
            tracing::error!(
                "   Set INITIAL_ADMIN_USERNAME and INITIAL_ADMIN_PASSWORD environment variables"
            );
            tracing::error!(
                "   Example: INITIAL_ADMIN_USERNAME=admin INITIAL_ADMIN_PASSWORD=change-me"
            );
            return Err(anyhow::anyhow!(
                "No users exist. Please set INITIAL_ADMIN_USERNAME and INITIAL_ADMIN_PASSWORD"
            ));
        }
    } else {
        tracing::info!("Found {} existing users", user_count);
    }

    // 获取绑定地址（从环境变量或默认值）
    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    tracing::info!("Starting server on {}", bind_addr);

    // 创建 Axum 应用
    let app = create_app(db_repo);

    // 绑定监听器
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server started successfully");
    tracing::info!("> Health check: http://{}/health", bind_addr);
    tracing::info!("> API endpoint: http://{}/api/v1", bind_addr);
    tracing::info!("> Scalar UI: http://{}/scalar", bind_addr);
    tracing::info!("> Swagger UI: http://{}/swagger-ui", bind_addr);

    // 启动服务器
    axum::serve(listener, app).await?;

    Ok(())
}
