//! 数据清理模块
//!
//! 负责定期清理过期的历史数据，保持数据库轻量

use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::db::service_repo::ServiceRepository;
use crate::state::AppState;

/// Metrics 数据保留时间（24小时）
const METRICS_RETENTION_HOURS: i64 = 24;

/// ServiceStatus 数据保留时间（30小时）
const SERVICE_STATUS_RETENTION_HOURS: i64 = 30;

/// 数据清理间隔（1小时）
const CLEANUP_INTERVAL_HOURS: u64 = 1;

/// 启动数据清理后台任务
///
/// 定期清理过期的 metrics 和 service_status 数据
///
/// # 性能
/// - 使用批量删除操作
/// - 删除操作使用时间戳索引
/// - 异步执行不阻塞主服务
pub fn spawn_cleanup_task(state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut cleanup_interval = interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));

        tracing::info!(
            "Data cleanup task started (metrics: {}h, service_status: {}h, interval: {}h)",
            METRICS_RETENTION_HOURS,
            SERVICE_STATUS_RETENTION_HOURS,
            CLEANUP_INTERVAL_HOURS
        );

        loop {
            cleanup_interval.tick().await;

            if let Err(e) = cleanup_old_data(&state).await {
                tracing::error!("Data cleanup failed: {:?}", e);
            }
        }
    })
}

/// 清理过期数据
async fn cleanup_old_data(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().timestamp();

    // 1. 清理过期的 Metrics 数据（24小时前）
    let metrics_cutoff = now - (METRICS_RETENTION_HOURS * 3600);
    match state.db.cleanup_old_metrics(metrics_cutoff).await {
        Ok(deleted) => {
            if deleted > 0 {
                tracing::info!(
                    "Cleaned up {} old metrics records (older than {}h)",
                    deleted,
                    METRICS_RETENTION_HOURS
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to cleanup old metrics: {:?}", e);
        }
    }

    // 2. 清理过期的 ServiceStatus 数据（30小时前）
    match state
        .db
        .services()
        .cleanup_old_status(SERVICE_STATUS_RETENTION_HOURS)
        .await
    {
        Ok(deleted) => {
            if deleted > 0 {
                tracing::info!(
                    "Cleaned up {} old service_status records (older than {}h)",
                    deleted,
                    SERVICE_STATUS_RETENTION_HOURS
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to cleanup old service_status: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_periods() {
        // 验证保留时间配置正确
        assert_eq!(METRICS_RETENTION_HOURS, 24);
        assert_eq!(SERVICE_STATUS_RETENTION_HOURS, 30);
    }

    #[test]
    fn test_cleanup_interval() {
        let duration = Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600);
        assert_eq!(duration.as_secs(), 3600); // 1小时
    }

    #[test]
    fn test_cutoff_calculation() {
        let now = chrono::Utc::now().timestamp();
        let metrics_cutoff = now - (METRICS_RETENTION_HOURS * 3600);
        let service_cutoff = now - (SERVICE_STATUS_RETENTION_HOURS * 3600);

        // 验证计算正确
        assert_eq!(now - metrics_cutoff, METRICS_RETENTION_HOURS * 3600);
        assert_eq!(now - service_cutoff, SERVICE_STATUS_RETENTION_HOURS * 3600);
    }
}
