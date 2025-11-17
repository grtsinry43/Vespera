pub mod error;
pub mod models;
mod repo;

use error::DbResult;
use std::sync::Arc;

pub use repo::SqliteRepo;

/// 数据库仓库类型（简化为 Arc<SqliteRepo>）
pub type DbRepo = Arc<SqliteRepo>;

/// 初始化数据库连接
///
/// 使用 SQLite 嵌入式数据库，零依赖部署
///
/// # 环境变量
/// - `SQLITE_PATH`: SQLite 数据库文件路径（默认: monitor.db）
pub async fn init_db() -> DbResult<DbRepo> {
    let sqlite_path = std::env::var("SQLITE_PATH").unwrap_or_else(|_| "monitor.db".to_string());
    let sqlite_url = format!("sqlite:{}?mode=rwc", sqlite_path);

    tracing::info!("Initializing SQLite database...");

    let repo = SqliteRepo::new(&sqlite_url).await?;

    tracing::info!("db connected: SQLite ({})", sqlite_path);

    Ok(Arc::new(repo))
}
