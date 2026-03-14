//! 告警数据模型
//!
//! 定义告警规则、告警记录、通知渠道等数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// 告警规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertRuleType {
    CpuHigh,
    MemoryHigh,
    DiskFull,
    NodeOffline,
    LoadHigh,
}

impl AlertRuleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertRuleType::CpuHigh => "cpu_high",
            AlertRuleType::MemoryHigh => "memory_high",
            AlertRuleType::DiskFull => "disk_full",
            AlertRuleType::NodeOffline => "node_offline",
            AlertRuleType::LoadHigh => "load_high",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cpu_high" => Some(AlertRuleType::CpuHigh),
            "memory_high" => Some(AlertRuleType::MemoryHigh),
            "disk_full" => Some(AlertRuleType::DiskFull),
            "node_offline" => Some(AlertRuleType::NodeOffline),
            "load_high" => Some(AlertRuleType::LoadHigh),
            _ => None,
        }
    }
}

/// 告警严重级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info" => Some(AlertSeverity::Info),
            "warning" => Some(AlertSeverity::Warning),
            "critical" => Some(AlertSeverity::Critical),
            _ => None,
        }
    }
}

/// 告警规则配置
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum AlertRuleConfig {
    CpuHigh {
        threshold_percent: f32,
        duration_secs: i64,
    },
    MemoryHigh {
        threshold_percent: f32,
        duration_secs: i64,
    },
    DiskFull {
        threshold_percent: f32,
        mount_point: Option<String>,
    },
    NodeOffline {
        timeout_secs: i64,
    },
    LoadHigh {
        threshold: f32,
        duration_secs: i64,
    },
}

/// 节点过滤器
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum NodeFilter {
    All,
    Specific { node_ids: Vec<i64> },
    Tags { tags: Vec<String> },
}

/// 通知渠道
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum NotificationChannel {
    Webhook {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
    },
    Email {
        recipients: Vec<String>,
        subject_template: String,
        body_template: String,
    },
    WebSocket,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AlertRule {
    pub id: i64,
    pub name: String,
    pub rule_type: AlertRuleType,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub config: AlertRuleConfig,
    pub notification_channels: Vec<NotificationChannel>,
    pub silence_duration_secs: i64,
    pub repeat_interval_secs: Option<i64>,
    pub node_filter: NodeFilter,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 告警记录
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Alert {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub rule_id: i64,
    pub node_id: i64,
    pub node_name: String,
    pub severity: AlertSeverity,
    pub alert_type: AlertRuleType,
    pub message: String,
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Alert {
    /// 转换为 WebSocket 消息格式
    pub fn to_ws_message(&self) -> vespera_common::AlertData {
        let is_public = self
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("node_is_public"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        vespera_common::AlertData {
            alert_id: self.id.unwrap_or(0),
            node_id: self.node_id,
            node_name: self.node_name.clone(),
            level: self.severity.as_str().to_string(),
            alert_type: self.alert_type.as_str().to_string(),
            message: self.message.clone(),
            is_public,
            triggered_at: self.triggered_at,
        }
    }
}

/// 通知配置 (全局)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationSettings {
    pub id: i64,
    pub smtp_server: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_address: Option<String>,
    pub smtp_use_tls: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_type_serialization() {
        let rule_type = AlertRuleType::CpuHigh;
        let json = serde_json::to_string(&rule_type).unwrap();
        assert_eq!(json, r#""cpu_high""#);

        let deserialized: AlertRuleType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AlertRuleType::CpuHigh);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = AlertSeverity::Warning;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, r#""warning""#);
    }

    #[test]
    fn test_rule_config_serialization() {
        let config = AlertRuleConfig::CpuHigh {
            threshold_percent: 90.0,
            duration_secs: 300,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"CpuHigh""#));
        assert!(json.contains(r#""threshold_percent":90.0"#));

        let deserialized: AlertRuleConfig = serde_json::from_str(&json).unwrap();
        match deserialized {
            AlertRuleConfig::CpuHigh {
                threshold_percent,
                duration_secs,
            } => {
                assert_eq!(threshold_percent, 90.0);
                assert_eq!(duration_secs, 300);
            }
            _ => panic!("Expected CpuHigh"),
        }
    }

    #[test]
    fn test_notification_channel_webhook() {
        let channel = NotificationChannel::Webhook {
            url: "https://hooks.slack.com/test".to_string(),
            headers: None,
            template: None,
        };

        let json = serde_json::to_string(&channel).unwrap();
        assert!(json.contains(r#""type":"Webhook""#));
        assert!(json.contains(r#""url":"https://hooks.slack.com/test""#));
    }

    #[test]
    fn test_node_filter() {
        // All
        let filter = NodeFilter::All;
        let json = serde_json::to_string(&filter).unwrap();
        assert_eq!(json, r#"{"type":"All"}"#);

        // Specific
        let filter = NodeFilter::Specific {
            node_ids: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains(r#""type":"Specific""#));
        assert!(json.contains(r#"[1,2,3]"#));
    }
}
