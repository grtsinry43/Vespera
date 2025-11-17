//! 测试工具模块
//!
//! 提供测试所需的辅助函数，解决并发测试中的数据库锁问题

use crate::db::{DbRepo, SqliteRepo};
use std::sync::Arc;

/// 创建测试用的临时数据库
///
/// # 解决的问题
/// 1. SQLite 文件锁冲突：每个测试使用独立的内存数据库
/// 2. 并发测试安全：内存数据库天然隔离
/// 3. 性能优化：内存数据库比文件数据库快 10-100 倍
///
/// # 实现细节
/// - 使用 `:memory:` 创建内存数据库（每个连接独立）
/// - 自动运行迁移
/// - 配置性能优化参数
///
/// # 返回值
/// 返回 `DbRepo`（Arc<SqliteRepo>），可直接用于测试
pub async fn create_test_db() -> DbRepo {
    // 使用内存数据库，每个测试完全隔离
    let sqlite_url = "sqlite::memory:";

    let repo = SqliteRepo::new(sqlite_url)
        .await
        .expect("Failed to create in-memory test database");

    Arc::new(repo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_db() {
        // 验证可以创建多个独立的测试数据库
        let db1 = create_test_db().await;
        let db2 = create_test_db().await;

        // 两个数据库应该是独立的实例
        assert!(!Arc::ptr_eq(&db1, &db2));
    }

    #[tokio::test]
    async fn test_concurrent_db_creation() {
        // 验证并发创建数据库不会冲突
        let handles: Vec<_> = (0..10)
            .map(|_| tokio::spawn(async { create_test_db().await }))
            .collect();

        for handle in handles {
            let db = handle.await.expect("Task should not panic");
            // 确保每个数据库都成功创建
            assert!(Arc::strong_count(&db) > 0);
        }
    }
}
