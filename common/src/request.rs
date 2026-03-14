use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Agent 数据上报请求
///
/// Agent 每次上报时携带节点基本信息和当前指标数据
/// Server 根据 node_uuid 判断是首次注册还是后续上报
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReportRequest {
    // === 节点基本信息 (首次注册时使用) ===
    /// 节点唯一标识符 (UUID v4)
    ///
    /// 使用 uuid::Uuid 类型，serde 自动处理验证
    /// 无效 UUID 会自动返回 400 Bad Request
    pub node_uuid: Uuid,

    /// 节点名称
    pub node_name: String,

    /// IP 地址
    pub ip_address: String,

    /// Agent 版本
    pub agent_version: String,

    /// 操作系统类型 (linux/windows/macos)
    pub os_type: String,

    /// 操作系统版本 (例如: "Ubuntu 22.04")
    pub os_version: Option<String>,

    /// CPU 核心数
    pub cpu_cores: i64,

    /// 总内存 (bytes)
    pub total_memory: i64,

    /// 节点标签
    pub tags: Option<Vec<String>>,

    // === 当前监控指标 ===
    pub metrics: MetricsData,
}

/// 监控指标数据
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetricsData {
    /// Unix 时间戳 (秒)
    pub timestamp: i64,

    // === CPU ===
    /// CPU 使用率 (0-100)
    pub cpu_usage: f64,

    // === Memory ===
    /// 已用内存 (bytes)
    pub memory_used: i64,

    /// 内存使用率 (0-100)
    pub memory_usage: f64,

    // === Disk ===
    /// 磁盘信息列表
    pub disk_info: Vec<DiskInfo>,

    // === Network (累计值) ===
    /// 入站流量累计 (bytes)
    pub net_in_bytes: i64,

    /// 出站流量累计 (bytes)
    pub net_out_bytes: i64,

    // === Load Average ===
    /// 1分钟负载
    pub load_1: Option<f64>,

    /// 5分钟负载
    pub load_5: Option<f64>,

    /// 15分钟负载
    pub load_15: Option<f64>,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiskInfo {
    /// 挂载点
    pub mount: String,

    /// 已用空间 (bytes)
    pub used: i64,

    /// 总空间 (bytes)
    pub total: i64,

    /// 使用率 (0-100)
    pub usage: f64,
}
