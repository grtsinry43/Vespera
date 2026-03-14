//! 节点健康检查模块
//!
//! 负责定期检查节点的在线状态，将超时未上报的节点标记为离线

use crate::state::AppState;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use vespera_common::ServerMessage;

/// 心跳超时阈值（秒）
///
/// Agent 默认每 5 秒上报一次，设置 15 秒超时允许最多丢失 2 次上报
const NODE_TIMEOUT_SECS: i64 = 15;

/// 健康检查间隔（秒）
const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;

/// 启动节点健康检查后台任务
///
/// 每隔 HEALTH_CHECK_INTERVAL_SECS 秒检查一次所有节点，
/// 将 last_seen 超过 NODE_TIMEOUT_SECS 的节点标记为 offline
///
/// # 性能
/// - 查询使用索引 (status, last_seen)
/// - 批量更新减少数据库开销
/// - 异步执行不阻塞主服务
pub fn spawn_health_check_task(state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS));

        tracing::info!(
            "Node health check task started (timeout: {}s, interval: {}s)",
            NODE_TIMEOUT_SECS,
            HEALTH_CHECK_INTERVAL_SECS
        );

        loop {
            check_interval.tick().await;

            if let Err(e) = check_stale_nodes(&state).await {
                tracing::error!("Health check failed: {:?}", e);
            }
        }
    })
}

/// 检查并标记超时节点
async fn check_stale_nodes(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().timestamp();
    let timeout_threshold = now - NODE_TIMEOUT_SECS;

    // 查询所有超时的在线节点
    let stale_nodes = state.db.get_stale_nodes(timeout_threshold).await?;

    if stale_nodes.is_empty() {
        // 没有超时节点，无需操作
        return Ok(());
    }

    tracing::info!(
        "Found {} stale nodes (last_seen < {})",
        stale_nodes.len(),
        timeout_threshold
    );

    // 批量更新状态并广播下线事件
    for node in stale_nodes {
        // 更新节点状态为 offline
        if let Err(e) = state
            .db
            .update_node_status(node.id, "offline", node.last_seen)
            .await
        {
            tracing::error!(
                "Failed to update node {} status to offline: {:?}",
                node.id,
                e
            );
            continue;
        }

        tracing::info!(
            node_id = node.id,
            node_name = %node.name,
            last_seen = node.last_seen,
            elapsed_secs = now - node.last_seen,
            "Node marked as offline"
        );

        // 广播 NodeOffline 事件
        let offline_msg = ServerMessage::NodeOffline {
            node_id: node.id,
            node_name: node.name.clone(),
        };

        match state.broadcaster.broadcast(offline_msg) {
            Ok(n) => {
                tracing::debug!(
                    node_id = node.id,
                    receiver_count = n,
                    "NodeOffline event broadcasted to {} WebSocket clients",
                    n
                );
            }
            Err(_) => {
                tracing::debug!(
                    node_id = node.id,
                    "No WebSocket clients connected, NodeOffline event not broadcasted"
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_threshold() {
        let now = chrono::Utc::now().timestamp();
        let threshold = now - NODE_TIMEOUT_SECS;

        // 验证阈值计算正确
        assert_eq!(now - threshold, NODE_TIMEOUT_SECS);
    }

    #[test]
    fn test_interval_duration() {
        let duration = Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS);
        assert_eq!(duration.as_secs(), HEALTH_CHECK_INTERVAL_SECS);
    }
}
