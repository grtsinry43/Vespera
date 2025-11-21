//! 节点数据模型
//!
//! 定义节点相关的数据结构，在 Server 和 Frontend 之间共享

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 节点公开信息（普通用户可见）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicNode {
    pub id: i64,
    pub name: String,
    pub status: String, // online/offline/error
    pub os_type: String,
    pub cpu_cores: i64,
    pub total_memory: i64,
    pub last_seen: i64,
    pub is_public: bool,
    pub tags: Option<Vec<String>>,
    // 最新指标（可选）
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub net_in: Option<f64>,  // MB/s
    pub net_out: Option<f64>, // MB/s
}

/// 节点完整信息（管理员可见）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminNode {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub ip_address: String,
    pub agent_version: String,
    pub os_type: String,
    pub os_version: Option<String>,
    pub cpu_cores: i64,
    pub total_memory: i64,
    pub status: String,
    pub last_seen: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_public: bool,
    pub tags: Option<Vec<String>>,
}

/// 节点详情（包含最新指标）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeDetail<T> {
    pub node: T,
    pub latest_metrics: Option<NodeMetrics>,
}

/// 节点指标摘要（用于详情页）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeMetrics {
    pub timestamp: i64,
    pub cpu_usage: f64,
    pub memory_used: i64,
    pub memory_total: i64,
    pub memory_usage: f64,
    pub disk_info: Vec<DiskMetric>,
    pub net_in_bytes: i64,
    pub net_out_bytes: i64,
    pub load_1: Option<f64>,
    pub load_5: Option<f64>,
    pub load_15: Option<f64>,
}

/// 磁盘指标
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiskMetric {
    pub mount: String,
    pub used: i64,
    pub total: i64,
    pub usage: f64,
}

/// 更新节点请求（管理员）
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

/// 更新节点可见性请求（管理员）
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNodeVisibilityRequest {
    pub is_public: bool,
}

/// 历史指标查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct MetricsRangeQuery {
    pub start: i64,
    pub end: i64,
    #[serde(default = "default_metrics_limit")]
    pub limit: i64,
}

fn default_metrics_limit() -> i64 {
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_node_serialization() {
        let node = PublicNode {
            id: 1,
            name: "test-node".to_string(),
            status: "online".to_string(),
            os_type: "linux".to_string(),
            cpu_cores: 8,
            total_memory: 17179869184,
            last_seen: 1234567890,
            is_public: true,
            tags: Some(vec!["prod".to_string()]),
            cpu_usage: Some(45.5),
            memory_usage: Some(62.3),
            net_in: Some(15.4),
            net_out: Some(42.1),
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"name\":\"test-node\""));
    }

    #[test]
    fn test_default_metrics_limit() {
        let query: MetricsRangeQuery = serde_json::from_str(
            r#"{"start": 1000, "end": 2000}"#
        ).unwrap();
        assert_eq!(query.limit, 100);
    }
}
