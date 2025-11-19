//! 告警规则数据库查询
//!
//! 提供告警规则、告警历史的数据库操作接口

use crate::alert::models::*;
use crate::db::error::DbError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// 告警数据库查询接口
#[async_trait::async_trait]
pub trait AlertRepository {
    /// 获取所有启用的告警规则
    async fn get_active_rules(&self) -> Result<Vec<AlertRule>, DbError>;

    /// 获取适用于特定节点的告警规则
    async fn get_rules_for_node(&self, node_id: i64) -> Result<Vec<AlertRule>, DbError>;

    /// 根据 ID 获取单个规则
    async fn get_rule(&self, rule_id: i64) -> Result<Option<AlertRule>, DbError>;

    /// 列出所有告警规则（包括未启用的）
    async fn list_rules(&self, limit: i64, offset: i64) -> Result<Vec<AlertRule>, DbError>;

    /// 创建告警规则
    async fn create_rule(&self, rule: &AlertRuleCreate) -> Result<AlertRule, DbError>;

    /// 更新告警规则
    async fn update_rule(&self, rule_id: i64, rule: &AlertRuleUpdate)
        -> Result<AlertRule, DbError>;

    /// 删除告警规则
    async fn delete_rule(&self, rule_id: i64) -> Result<(), DbError>;

    /// 插入告警记录
    async fn insert_alert(&self, alert: &Alert) -> Result<i64, DbError>;

    /// 获取活跃告警 (未解决)
    async fn get_active_alerts(&self) -> Result<Vec<Alert>, DbError>;

    /// 获取节点的活跃告警
    async fn get_active_alerts_for_node(&self, node_id: i64) -> Result<Vec<Alert>, DbError>;

    /// 解决告警
    async fn resolve_alert(&self, alert_id: i64) -> Result<(), DbError>;

    /// 获取通知配置
    async fn get_notification_settings(&self) -> Result<NotificationSettings, DbError>;
}

/// 创建告警规则请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertRuleCreate {
    pub name: String,
    pub rule_type: AlertRuleType,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub config: AlertRuleConfig,
    pub notification_channels: Vec<NotificationChannel>,
    pub silence_duration_secs: i64,
    pub repeat_interval_secs: Option<i64>,
    pub node_filter: NodeFilter,
}

/// 更新告警规则请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertRuleUpdate {
    pub name: Option<String>,
    pub severity: Option<AlertSeverity>,
    pub enabled: Option<bool>,
    pub config: Option<AlertRuleConfig>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub silence_duration_secs: Option<i64>,
    pub repeat_interval_secs: Option<i64>,
    pub node_filter: Option<NodeFilter>,
}

/// AlertRepository 的 SQLite 实现
pub struct SqliteAlertRepo {
    pool: SqlitePool,
}

impl SqliteAlertRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 解析规则行为 AlertRule
    fn parse_rule(row: &sqlx::sqlite::SqliteRow) -> Result<AlertRule, DbError> {
        use sqlx::Row;

        let id: i64 = row.get("id");
        let name: String = row.get("name");
        let node_id: Option<i64> = row.get("node_id");
        let rule_type_str: String = row.get("rule_type");
        let severity_str: String = row.get("severity");
        let enabled: i64 = row.get("enabled");
        let config_json: String = row.get("config");
        let channels_json: String = row.get("notification_channels");
        let silence_duration_secs: i64 = row.get("silence_duration_secs");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");

        let rule_type = AlertRuleType::from_str(&rule_type_str)
            .ok_or_else(|| DbError::ParseError(format!("Invalid rule_type: {}", rule_type_str)))?;

        let severity = AlertSeverity::from_str(&severity_str)
            .ok_or_else(|| DbError::ParseError(format!("Invalid severity: {}", severity_str)))?;

        let config: AlertRuleConfig = serde_json::from_str(&config_json)
            .map_err(|e| DbError::ParseError(format!("Invalid config JSON: {}", e)))?;

        let notification_channels: Vec<NotificationChannel> = serde_json::from_str(&channels_json)
            .map_err(|e| {
                DbError::ParseError(format!("Invalid notification_channels JSON: {}", e))
            })?;

        // 根据 node_id 构建 NodeFilter
        let node_filter = if let Some(nid) = node_id {
            NodeFilter::Specific {
                node_ids: vec![nid],
            }
        } else {
            NodeFilter::All
        };

        Ok(AlertRule {
            id,
            name,
            rule_type,
            severity,
            enabled: enabled == 1,
            config,
            notification_channels,
            silence_duration_secs,
            repeat_interval_secs: None,
            node_filter,
            created_at,
            updated_at,
        })
    }

    /// 解析告警行为 Alert
    fn parse_alert(row: &sqlx::sqlite::SqliteRow) -> Result<Alert, DbError> {
        use sqlx::Row;

        let id: i64 = row.get("id");
        let rule_id: i64 = row.get("rule_id");
        let node_id: i64 = row.get("node_id");
        let node_name: String = row.get("node_name");
        let severity_str: String = row.get("severity");
        let alert_type_str: String = row.get("alert_type");
        let message: String = row.get("message");
        let triggered_at: i64 = row.get("triggered_at");
        let resolved_at: Option<i64> = row.get("resolved_at");
        let metadata_json: Option<String> = row.get("metadata");

        let severity = AlertSeverity::from_str(&severity_str)
            .ok_or_else(|| DbError::ParseError(format!("Invalid severity: {}", severity_str)))?;

        let alert_type = AlertRuleType::from_str(&alert_type_str).ok_or_else(|| {
            DbError::ParseError(format!("Invalid alert_type: {}", alert_type_str))
        })?;

        let metadata = if let Some(json) = metadata_json {
            Some(
                serde_json::from_str(&json)
                    .map_err(|e| DbError::ParseError(format!("Invalid metadata JSON: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Alert {
            id: Some(id),
            rule_id,
            node_id,
            node_name,
            severity,
            alert_type,
            message,
            triggered_at,
            resolved_at,
            metadata,
        })
    }
}

#[async_trait::async_trait]
impl AlertRepository for SqliteAlertRepo {
    async fn get_active_rules(&self) -> Result<Vec<AlertRule>, DbError> {
        let rows = sqlx::query("SELECT * FROM alert_rules WHERE enabled = 1 ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        rows.iter().map(Self::parse_rule).collect()
    }

    async fn get_rules_for_node(&self, node_id: i64) -> Result<Vec<AlertRule>, DbError> {
        // 获取所有启用的规则
        let all_rules = self.get_active_rules().await?;

        // 过滤适用于该节点的规则
        let filtered_rules: Vec<AlertRule> = all_rules
            .into_iter()
            .filter(|rule| {
                match &rule.node_filter {
                    NodeFilter::All => true,
                    NodeFilter::Specific { node_ids } => node_ids.contains(&node_id),
                    NodeFilter::Tags { .. } => {
                        // TODO: 实现标签过滤 (需要查询节点标签)
                        true
                    }
                }
            })
            .collect();

        Ok(filtered_rules)
    }

    async fn get_rule(&self, rule_id: i64) -> Result<Option<AlertRule>, DbError> {
        let row = sqlx::query("SELECT * FROM alert_rules WHERE id = ?")
            .bind(rule_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Self::parse_rule(&r)?)),
            None => Ok(None),
        }
    }

    async fn insert_alert(&self, alert: &Alert) -> Result<i64, DbError> {
        let metadata_json = alert
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).ok())
            .flatten();

        let result = sqlx::query(
            "INSERT INTO alerts (
                rule_id, node_id, node_name, severity, alert_type,
                message, triggered_at, resolved_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(alert.rule_id)
        .bind(alert.node_id)
        .bind(&alert.node_name)
        .bind(alert.severity.as_str())
        .bind(alert.alert_type.as_str())
        .bind(&alert.message)
        .bind(alert.triggered_at)
        .bind(alert.resolved_at)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    async fn get_active_alerts(&self) -> Result<Vec<Alert>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM alerts WHERE resolved_at IS NULL ORDER BY triggered_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        rows.iter().map(Self::parse_alert).collect()
    }

    async fn get_active_alerts_for_node(&self, node_id: i64) -> Result<Vec<Alert>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM alerts WHERE node_id = ? AND resolved_at IS NULL ORDER BY triggered_at DESC"
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        rows.iter().map(Self::parse_alert).collect()
    }

    async fn resolve_alert(&self, alert_id: i64) -> Result<(), DbError> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE alerts SET resolved_at = ? WHERE id = ?")
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn list_rules(&self, limit: i64, offset: i64) -> Result<Vec<AlertRule>, DbError> {
        let rows = sqlx::query("SELECT * FROM alert_rules ORDER BY id DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        rows.iter().map(Self::parse_rule).collect()
    }

    async fn create_rule(&self, rule: &AlertRuleCreate) -> Result<AlertRule, DbError> {
        let now = chrono::Utc::now().timestamp();

        // 序列化 config 和 notification_channels
        let config_json = serde_json::to_string(&rule.config)
            .map_err(|e| DbError::ParseError(format!("Failed to serialize config: {}", e)))?;

        let channels_json = serde_json::to_string(&rule.notification_channels).map_err(|e| {
            DbError::ParseError(format!("Failed to serialize notification_channels: {}", e))
        })?;

        // 根据 NodeFilter 确定 node_id
        let node_id = match &rule.node_filter {
            NodeFilter::Specific { node_ids } if !node_ids.is_empty() => Some(node_ids[0]),
            _ => None,
        };

        let result = sqlx::query(
            "INSERT INTO alert_rules (
                name, node_id, rule_type, severity, config,
                notification_channels, silence_duration_secs, enabled,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&rule.name)
        .bind(node_id)
        .bind(rule.rule_type.as_str())
        .bind(rule.severity.as_str())
        .bind(&config_json)
        .bind(&channels_json)
        .bind(rule.silence_duration_secs)
        .bind(if rule.enabled { 1 } else { 0 })
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        let rule_id = result.last_insert_rowid();

        // 返回创建的规则
        Ok(AlertRule {
            id: rule_id,
            name: rule.name.clone(),
            rule_type: rule.rule_type.clone(),
            severity: rule.severity.clone(),
            enabled: rule.enabled,
            config: rule.config.clone(),
            notification_channels: rule.notification_channels.clone(),
            silence_duration_secs: rule.silence_duration_secs,
            repeat_interval_secs: rule.repeat_interval_secs,
            node_filter: rule.node_filter.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    async fn update_rule(
        &self,
        rule_id: i64,
        rule: &AlertRuleUpdate,
    ) -> Result<AlertRule, DbError> {
        let now = chrono::Utc::now().timestamp();

        // 获取现有规则
        let existing = self
            .get_rule(rule_id)
            .await?
            .ok_or_else(|| DbError::NotFound)?;

        // 构建更新字段
        let name = rule.name.as_ref().unwrap_or(&existing.name);
        let severity = rule.severity.as_ref().unwrap_or(&existing.severity);
        let enabled = rule.enabled.unwrap_or(existing.enabled);
        let config = rule.config.as_ref().unwrap_or(&existing.config);
        let notification_channels = rule
            .notification_channels
            .as_ref()
            .unwrap_or(&existing.notification_channels);
        let silence_duration_secs = rule
            .silence_duration_secs
            .unwrap_or(existing.silence_duration_secs);
        let repeat_interval_secs = rule.repeat_interval_secs.or(existing.repeat_interval_secs);
        let node_filter = rule.node_filter.as_ref().unwrap_or(&existing.node_filter);

        // 序列化 JSON 字段
        let config_json = serde_json::to_string(config)
            .map_err(|e| DbError::ParseError(format!("Failed to serialize config: {}", e)))?;

        let channels_json = serde_json::to_string(notification_channels).map_err(|e| {
            DbError::ParseError(format!("Failed to serialize notification_channels: {}", e))
        })?;

        // 根据 NodeFilter 确定 node_id
        let node_id = match node_filter {
            NodeFilter::Specific { node_ids } if !node_ids.is_empty() => Some(node_ids[0]),
            _ => None,
        };

        sqlx::query(
            "UPDATE alert_rules SET
                name = ?,
                node_id = ?,
                severity = ?,
                config = ?,
                notification_channels = ?,
                silence_duration_secs = ?,
                enabled = ?,
                updated_at = ?
            WHERE id = ?",
        )
        .bind(name)
        .bind(node_id)
        .bind(severity.as_str())
        .bind(&config_json)
        .bind(&channels_json)
        .bind(silence_duration_secs)
        .bind(if enabled { 1 } else { 0 })
        .bind(now)
        .bind(rule_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        // 返回更新后的规则
        Ok(AlertRule {
            id: rule_id,
            name: name.clone(),
            rule_type: existing.rule_type,
            severity: severity.clone(),
            enabled,
            config: config.clone(),
            notification_channels: notification_channels.clone(),
            silence_duration_secs,
            repeat_interval_secs,
            node_filter: node_filter.clone(),
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    async fn delete_rule(&self, rule_id: i64) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM alert_rules WHERE id = ?")
            .bind(rule_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound)?;
        }

        Ok(())
    }

    async fn get_notification_settings(&self) -> Result<NotificationSettings, DbError> {
        let row = sqlx::query("SELECT * FROM notification_settings WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        use sqlx::Row;

        Ok(NotificationSettings {
            id: row.get("id"),
            smtp_server: row.get("smtp_server"),
            smtp_username: row.get("smtp_username"),
            smtp_password: row.get("smtp_password"),
            smtp_from_address: row.get("smtp_from_address"),
            smtp_use_tls: row.get::<i64, _>("smtp_use_tls") == 1,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_filter_all() {
        let filter = NodeFilter::All;
        // All 应该匹配任何节点
        assert!(matches!(filter, NodeFilter::All));
    }

    #[test]
    fn test_node_filter_specific() {
        let filter = NodeFilter::Specific {
            node_ids: vec![1, 2, 3],
        };

        match filter {
            NodeFilter::Specific { node_ids } => {
                assert!(node_ids.contains(&1));
                assert!(node_ids.contains(&2));
                assert!(!node_ids.contains(&5));
            }
            _ => panic!("Expected Specific filter"),
        }
    }
}
