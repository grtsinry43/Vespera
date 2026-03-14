//! WebSocket 消息类型定义
//!
//! 定义 Server 和 Client 之间的 WebSocket 通信协议

use serde::{Deserialize, Serialize};

/// Server -> Client 消息
///
/// 使用 serde's tag/content 枚举表示,序列化为:
/// ```json
/// {
///   "type": "metrics_update",
///   "data": { ... }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// 指标更新
    #[serde(rename = "metrics_update")]
    MetricsUpdate(MetricsUpdate),

    /// 节点上线
    #[serde(rename = "node_online")]
    NodeOnline { node_id: i64, node_name: String },

    /// 节点下线
    #[serde(rename = "node_offline")]
    NodeOffline { node_id: i64, node_name: String },

    /// 告警事件
    #[serde(rename = "alert")]
    Alert(AlertData),

    /// 心跳 ping
    #[serde(rename = "ping")]
    Ping,

    /// 错误消息
    #[serde(rename = "error")]
    Error { message: String },

    /// 认证成功
    #[serde(rename = "auth_success")]
    AuthSuccess,

    /// 认证失败
    #[serde(rename = "auth_failed")]
    AuthFailed { message: String },
}

/// 指标更新数据
///
/// 从 Agent 上报的数据转换而来
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    /// 节点 ID (数据库主键)
    pub node_id: i64,

    /// 节点 UUID
    pub node_uuid: String,

    /// 节点名称
    pub node_name: String,

    /// 时间戳 (Unix timestamp)
    pub timestamp: i64,

    /// CPU 使用率 (0-100)
    pub cpu_usage: f32,

    /// 内存使用率 (0-100)
    pub memory_usage: f32,

    /// 已使用内存 (bytes)
    pub memory_used: i64,

    /// 总内存 (bytes)
    pub memory_total: i64,

    /// 磁盘信息
    pub disk_info: Vec<DiskInfoWs>,

    /// 网络入流量 (bytes)
    pub network_in: i64,

    /// 网络出流量 (bytes)
    pub network_out: i64,

    /// 1 分钟负载
    pub load_1: Option<f32>,

    /// 5 分钟负载
    pub load_5: Option<f32>,

    /// 15 分钟负载
    pub load_15: Option<f32>,
}

/// 磁盘信息 (WebSocket 专用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfoWs {
    /// 挂载点
    pub mount: String,

    /// 已使用 (bytes)
    pub used: i64,

    /// 总容量 (bytes)
    pub total: i64,

    /// 使用率 (0-100)
    pub usage: f32,
}

/// 告警数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    /// 告警 ID
    pub alert_id: i64,

    /// 节点 ID
    pub node_id: i64,

    /// 节点名称
    pub node_name: String,

    /// 告警级别 (info/warning/critical)
    pub level: String,

    /// 告警类型 (cpu_high/memory_high/disk_full/offline)
    pub alert_type: String,

    /// 告警消息
    pub message: String,

    /// 所属节点是否公开
    pub is_public: bool,

    /// 触发时间
    pub triggered_at: i64,
}

/// Client -> Server 消息
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 认证消息 (首次连接必须发送)
    #[serde(rename = "auth")]
    Auth { token: String },

    /// 心跳响应
    #[serde(rename = "pong")]
    Pong,

    /// 订阅特定节点
    #[serde(rename = "subscribe")]
    Subscribe { node_ids: Vec<i64> },

    /// 取消订阅节点
    #[serde(rename = "unsubscribe")]
    Unsubscribe { node_ids: Vec<i64> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_serialization() {
        // 测试 MetricsUpdate 序列化
        let msg = ServerMessage::MetricsUpdate(MetricsUpdate {
            node_id: 1,
            node_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            node_name: "test-node".to_string(),
            timestamp: 1705449600,
            cpu_usage: 45.2,
            memory_usage: 78.5,
            memory_used: 8589934592,
            memory_total: 17179869184,
            disk_info: vec![DiskInfoWs {
                mount: "/".to_string(),
                used: 107374182400,
                total: 214748364800,
                usage: 50.0,
            }],
            network_in: 1024000,
            network_out: 512000,
            load_1: Some(1.5),
            load_5: Some(1.2),
            load_15: Some(1.0),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"metrics_update""#));
        assert!(json.contains(r#""node_id":1"#));

        // 反序列化验证
        let _deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_client_message_deserialization() {
        // 测试 Auth 消息
        let json = r#"{"type":"auth","token":"eyJhbGciOiJIUzI1NiJ9..."}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Auth { token } => {
                assert_eq!(token, "eyJhbGciOiJIUzI1NiJ9...");
            }
            _ => panic!("Expected Auth message"),
        }

        // 测试 Subscribe 消息
        let json = r#"{"type":"subscribe","node_ids":[1,2,3]}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Subscribe { node_ids } => {
                assert_eq!(node_ids, vec![1, 2, 3]);
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn test_error_messages() {
        // 测试错误消息
        let msg = ServerMessage::Error {
            message: "Connection lost".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains(r#""message":"Connection lost""#));
    }
}
