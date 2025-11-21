use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// 服务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Http,
    Tcp,
}

impl ServiceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceType::Http => "http",
            ServiceType::Tcp => "tcp",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" => Some(ServiceType::Http),
            "tcp" => Some(ServiceType::Tcp),
            _ => None,
        }
    }
}

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Up,
    Down,
    Timeout,
    Error,
    Unknown, // 用于前端展示缺失数据
}

impl ServiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceStatus::Up => "up",
            ServiceStatus::Down => "down",
            ServiceStatus::Timeout => "timeout",
            ServiceStatus::Error => "error",
            ServiceStatus::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "up" => Some(ServiceStatus::Up),
            "down" => Some(ServiceStatus::Down),
            "timeout" => Some(ServiceStatus::Timeout),
            "error" => Some(ServiceStatus::Error),
            "unknown" => Some(ServiceStatus::Unknown),
            _ => None,
        }
    }
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Service {
    pub id: i64,
    pub node_id: Option<i64>,
    pub name: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub target: String,
    pub check_interval: i64,      // 秒
    pub timeout: i64,             // 秒
    pub method: String,           // GET/POST/HEAD 等
    pub expected_code: i64,
    pub expected_body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub enabled: bool,
    pub is_public: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建服务请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceCreate {
    pub node_id: Option<i64>,
    pub name: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub target: String,
    #[serde(default = "default_check_interval")]
    pub check_interval: i64,
    #[serde(default = "default_timeout")]
    pub timeout: i64,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_expected_code")]
    pub expected_code: i64,
    pub expected_body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_is_public")]
    pub is_public: bool,
}

/// 更新服务请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceUpdate {
    pub name: Option<String>,
    pub target: Option<String>,
    pub check_interval: Option<i64>,
    pub timeout: Option<i64>,
    pub method: Option<String>,
    pub expected_code: Option<i64>,
    pub expected_body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
    pub is_public: Option<bool>,
}

/// 更新服务可见性请求（管理员）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateServiceVisibilityRequest {
    pub is_public: bool,
}

/// 服务检查结果
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceCheckResult {
    pub service_id: i64,
    pub agent_id: Option<i64>,
    pub status: ServiceStatus,
    pub response_time: Option<i64>,   // 毫秒
    pub status_code: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: i64,              // unix timestamp
}

/// 服务状态历史记录
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatusRecord {
    pub id: i64,
    pub service_id: i64,
    pub agent_id: Option<i64>,
    pub status: ServiceStatus,
    pub response_time: Option<i64>,
    pub status_code: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: i64,
}

/// 服务状态概览（包含最近30个数据点）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatusOverview {
    pub service: Service,
    pub current_status: ServiceStatus,
    pub history: Vec<ServiceStatusPoint>, // 最近30个数据点
}

/// 服务状态数据点（用于前端图表）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatusPoint {
    pub timestamp: i64,
    pub status: ServiceStatus,
    pub response_time: Option<i64>,
}

// 默认值函数
fn default_check_interval() -> i64 {
    3600 // 1小时
}

fn default_timeout() -> i64 {
    10 // 10秒
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_expected_code() -> i64 {
    200
}

fn default_enabled() -> bool {
    true
}

fn default_is_public() -> bool {
    false
}
