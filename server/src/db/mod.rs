pub mod alert_repo;
pub mod error;
pub mod models;
mod repo;
pub mod service_repo;
pub mod user_repo;

use error::DbResult;
use std::sync::Arc;

pub use alert_repo::{AlertRepository, SqliteAlertRepo};
pub use repo::SqliteRepo;
pub use service_repo::{ServiceRepository, SqliteServiceRepo};
pub use user_repo::{UserRepoError, UserRepository};

/// 数据库仓库类型（简化为 Arc<SqliteRepo>）
pub type DbRepo = Arc<SqliteRepo>;

/// 初始化数据库连接
///
/// 使用 SQLite 嵌入式数据库，零依赖部署
///
/// # 环境变量
/// - `DATABASE_URL`: 完整 SQLite 连接串（优先）
/// - `SQLITE_PATH`: SQLite 数据库文件路径（默认: monitor.db）
pub async fn init_db() -> DbResult<DbRepo> {
    let (sqlite_path, sqlite_url) = match std::env::var("DATABASE_URL") {
        Ok(url) => {
            let path = url
                .trim_start_matches("sqlite:///")
                .trim_start_matches("sqlite://")
                .trim_start_matches("sqlite:")
                .trim_end_matches("?mode=rwc")
                .to_string();
            (path, url)
        }
        Err(_) => {
            let path = std::env::var("SQLITE_PATH").unwrap_or_else(|_| "monitor.db".to_string());
            let url = format!("sqlite:{}?mode=rwc", path);
            (path, url)
        }
    };

    tracing::info!("Initializing SQLite database...");

    let repo = SqliteRepo::new(&sqlite_url).await?;

    tracing::info!("db connected: SQLite ({})", sqlite_path);

    Ok(Arc::new(repo))
}
