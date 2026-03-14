//! WebSocket 广播器
//!
//! 负责将 Server 事件广播到所有已连接的 WebSocket 客户端
//!
//! # 性能优化
//! - 使用 `tokio::sync::broadcast` 零拷贝广播
//! - 通道容量设置为 1000,避免过度内存占用
//! - 慢速消费者会丢失消息(而非阻塞整个系统)

use tokio::sync::broadcast;
use vespera_common::ServerMessage;

/// 广播通道容量
///
/// 保留最近 1000 条消息,防止内存膨胀
/// 如果前端消费速度慢,会丢失旧消息(这是可接受的)
const CHANNEL_CAPACITY: usize = 1000;

/// WebSocket 广播器
///
/// 用于在 AppState 中共享
#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<ServerMessage>,
}

impl Broadcaster {
    /// 创建新的广播器
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    /// 广播消息到所有订阅者
    ///
    /// # 返回值
    /// - Ok(n): 成功发送到 n 个接收者
    /// - Err: 无接收者(可忽略)
    ///
    /// # 性能
    /// - 零拷贝 (broadcast 内部使用 Arc)
    /// - 非阻塞 (send 立即返回)
    pub fn broadcast(
        &self,
        msg: ServerMessage,
    ) -> Result<usize, broadcast::error::SendError<ServerMessage>> {
        self.tx.send(msg)
    }

    /// 订阅广播通道
    ///
    /// 每个 WebSocket 连接调用一次
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.tx.subscribe()
    }

    /// 获取当前订阅者数量
    ///
    /// 用于监控和调试
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vespera_common::MetricsUpdate;

    #[tokio::test]
    async fn test_broadcaster_basic() {
        let broadcaster = Broadcaster::new();

        // 创建 2 个订阅者
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        assert_eq!(broadcaster.receiver_count(), 2);

        // 广播消息
        let msg = ServerMessage::Ping;
        let result = broadcaster.broadcast(msg.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // 2 个接收者

        // 验证接收
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        match (received1, received2) {
            (ServerMessage::Ping, ServerMessage::Ping) => {}
            _ => panic!("Expected Ping message"),
        }
    }

    #[tokio::test]
    async fn test_broadcaster_no_receivers() {
        let broadcaster = Broadcaster::new();

        // 没有订阅者时广播
        let msg = ServerMessage::Ping;
        let result = broadcaster.broadcast(msg);

        // 应该返回错误(但这是可接受的)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcaster_metrics_update() {
        let broadcaster = Broadcaster::new();
        let mut rx = broadcaster.subscribe();

        // 广播复杂消息
        let msg = ServerMessage::MetricsUpdate(MetricsUpdate {
            node_id: 1,
            node_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            node_name: "test-node".to_string(),
            timestamp: 1705449600,
            cpu_usage: 45.2,
            memory_usage: 78.5,
            memory_used: 8589934592,
            memory_total: 17179869184,
            disk_info: vec![],
            network_in: 1024000,
            network_out: 512000,
            load_1: Some(1.5),
            load_5: Some(1.2),
            load_15: Some(1.0),
        });

        broadcaster.broadcast(msg).unwrap();

        let received = rx.recv().await.unwrap();
        match received {
            ServerMessage::MetricsUpdate(update) => {
                assert_eq!(update.node_id, 1);
                assert_eq!(update.cpu_usage, 45.2);
            }
            _ => panic!("Expected MetricsUpdate"),
        }
    }

    #[tokio::test]
    async fn test_broadcaster_drop_slow_consumer() {
        let broadcaster = Broadcaster::new();
        let mut rx = broadcaster.subscribe();

        // 发送超过通道容量的消息
        for i in 0..CHANNEL_CAPACITY + 100 {
            let msg = ServerMessage::NodeOnline {
                node_id: i as i64,
                node_name: format!("node-{}", i),
            };
            let _ = broadcaster.broadcast(msg);
        }

        // 慢速消费者应该收到 RecvError::Lagged
        let result = rx.recv().await;
        // 第一次 recv 可能会返回 Lagged 错误
        // 这是预期行为,确认系统不会阻塞
        if result.is_err() {
            let err = result.unwrap_err();
            match err {
                broadcast::error::RecvError::Lagged(n) => {
                    // 确认丢失了消息
                    assert!(n > 0);
                }
                _ => panic!("Expected Lagged error"),
            }
        }
    }
}
