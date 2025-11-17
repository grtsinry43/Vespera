use serde::{Deserialize, Serialize};

/// 节点（被监控的服务器）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub ip_address: String,
    pub agent_version: String,
    pub os_type: String,        // linux/windows/macos
    pub os_version: Option<String>,
    pub cpu_cores: i64,         // SQLite INTEGER = i64
    pub total_memory: i64,      // bytes
    pub status: String,         // online/offline/error
    pub last_seen: i64,         // unix timestamp
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Option<String>,   // JSON array
}

/// 创建节点的输入结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCreate {
    pub uuid: String,
    pub name: String,
    pub ip_address: String,
    pub agent_version: String,
    pub os_type: String,
    pub os_version: Option<String>,
    pub cpu_cores: i64,
    pub total_memory: i64,
    pub tags: Option<Vec<String>>,
}

impl NodeCreate {
    /// 将 tags Vec 序列化为 JSON 字符串
    pub fn tags_json(&self) -> Result<Option<String>, serde_json::Error> {
        match &self.tags {
            Some(tags) => Ok(Some(serde_json::to_string(tags)?)),
            None => Ok(None),
        }
    }
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount: String,
    pub used: i64,      // bytes
    pub total: i64,     // bytes
    pub usage: f64,     // 0-100 (SQLite REAL = f64)
}

/// 监控指标（时序数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: Option<i64>,
    pub node_id: i64,
    pub timestamp: i64,

    // CPU
    pub cpu_usage: f64,     // SQLite REAL = f64
    pub cpu_cores: i64,

    // Memory
    pub memory_used: i64,
    pub memory_total: i64,
    pub memory_usage: f64,

    // Disk (JSON array)
    pub disk_info: Vec<DiskInfo>,

    // Network (累计值)
    pub net_in_bytes: i64,
    pub net_out_bytes: i64,

    // Load Average
    pub load_1: Option<f64>,
    pub load_5: Option<f64>,
    pub load_15: Option<f64>,
}

impl Metric {
    /// 将 disk_info 序列化为 JSON 字符串
    pub fn disk_info_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.disk_info)
    }

    /// 从 JSON 字符串反序列化 disk_info
    pub fn from_disk_info_json(json: &str) -> Result<Vec<DiskInfo>, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: i64,
    pub name: String,
    pub node_id: Option<i64>,   // NULL 表示全局规则
    pub metric_type: String,    // cpu/memory/disk/network
    pub condition: String,      // gt/lt/eq
    pub threshold: f64,         // SQLite REAL = f64
    pub duration: i64,          // 秒
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建告警规则的输入结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleCreate {
    pub name: String,
    pub node_id: Option<i64>,
    pub metric_type: String,
    pub condition: String,
    pub threshold: f64,
    pub duration: i64,
}

/// 告警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub rule_id: i64,
    pub node_id: i64,
    pub level: String,          // warning/critical
    pub message: String,
    pub value: f64,             // SQLite REAL = f64
    pub status: String,         // active/resolved
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
}

/// 创建告警的输入结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCreate {
    pub rule_id: i64,
    pub node_id: i64,
    pub level: String,
    pub message: String,
    pub value: f64,
}
