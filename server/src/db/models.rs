use serde::{Deserialize, Serialize};
use vespera_common::UserRole;

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
    pub is_public: bool,
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
    pub is_public: bool,
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

// ============================================
// 用户认证相关模型
// ============================================

/// 数据库用户模型 (包含敏感信息)
#[derive(Debug, Clone)]
pub struct DbUser {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>, // OAuth 用户可能为 NULL
    pub role: String,                  // "admin" | "user"
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_login_at: Option<i64>,
}

impl DbUser {
    /// 转换为公开的 User 结构 (移除敏感信息)
    pub fn to_public_user(&self) -> vespera_common::User {
        vespera_common::User {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            role: UserRole::from_str(&self.role).unwrap_or(UserRole::User),
            avatar_url: self.avatar_url.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_login_at: self.last_login_at,
        }
    }
}

/// OAuth 关联账户
#[derive(Debug, Clone)]
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,          // "google" | "github"
    pub provider_user_id: String,  // OAuth 提供商的 user ID
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Refresh Token 数据库模型
#[derive(Debug, Clone)]
pub struct DbRefreshToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,        // SHA-256 哈希
    pub expires_at: i64,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub device_info: Option<String>,
}

/// 用户-节点权限
#[derive(Debug, Clone)]
pub struct UserNodePermission {
    pub id: i64,
    pub user_id: i64,
    pub node_id: i64,
    pub can_view: bool,
    pub can_manage: bool,
    pub created_at: i64,
}
