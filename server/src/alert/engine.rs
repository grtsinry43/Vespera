//! 告警引擎核心
//!
//! 负责评估告警规则并触发通知

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::alert::{models::*, state::AlertStateStore};
use crate::db::{AlertRepository, DbRepo};
use crate::db::models::{Metric, Node};
use crate::ws::Broadcaster;

/// 告警引擎
pub struct AlertEngine {
    db: DbRepo,
    state_store: Arc<AlertStateStore>,
    webhook_client: reqwest::Client,
    broadcaster: Broadcaster,
}

impl AlertEngine {
    pub fn new(db: DbRepo, broadcaster: Broadcaster) -> Self {
        Self {
            db,
            state_store: Arc::new(AlertStateStore::new()),
            webhook_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            broadcaster,
        }
    }

    /// 评估指标并触发告警
    pub async fn evaluate_metrics(
        &self,
        node: &Node,
        metrics: &Metric,
    ) -> Result<Vec<Alert>, AlertError> {
        // 获取适用于该节点的规则
        let rules = self.db.alerts().get_rules_for_node(node.id).await
            .map_err(|e| AlertError::DatabaseError(e.to_string()))?;

        let mut triggered_alerts = Vec::new();

        for rule in rules {
            if let Some(alert) = self.evaluate_rule(node, metrics, &rule).await? {
                // 检查静默期
                if self.state_store.should_fire(
                    node.id,
                    rule.id,
                    rule.silence_duration_secs
                ).await {
                    // 存储到数据库
                    let alert_id = self.db.alerts().insert_alert(&alert).await
                        .map_err(|e| AlertError::DatabaseError(e.to_string()))?;

                    let mut alert_with_id = alert.clone();
                    alert_with_id.id = Some(alert_id);

                    // 标记已触发
                    self.state_store.mark_fired(node.id, rule.id).await;

                    // 发送通知 (异步,不阻塞)
                    let engine = self.clone();
                    let alert_clone = alert_with_id.clone();
                    let rule_clone = rule.clone();
                    tokio::spawn(async move {
                        engine.send_notifications(&alert_clone, &rule_clone).await;
                    });

                    triggered_alerts.push(alert_with_id);
                }
            }
        }

        Ok(triggered_alerts)
    }

    /// 评估单个规则
    async fn evaluate_rule(
        &self,
        node: &Node,
        metrics: &Metric,
        rule: &AlertRule,
    ) -> Result<Option<Alert>, AlertError> {
        match &rule.config {
            AlertRuleConfig::CpuHigh { threshold_percent, duration_secs } => {
                if metrics.cpu_usage > (*threshold_percent as f64) {
                    if self.state_store.check_duration_exceeded(
                        node.id,
                        &rule.rule_type,
                        *duration_secs
                    ).await {
                        return Ok(Some(Alert {
                            id: None,
                            rule_id: rule.id,
                            node_id: node.id,
                            node_name: node.name.clone(),
                            severity: rule.severity.clone(),
                            alert_type: rule.rule_type.clone(),
                            message: format!(
                                "CPU 使用率 {:.1}% 超过阈值 {:.1}%",
                                metrics.cpu_usage, threshold_percent
                            ),
                            triggered_at: chrono::Utc::now().timestamp(),
                            resolved_at: None,
                            metadata: Some(serde_json::json!({
                                "node_is_public": node.is_public,
                                "cpu_usage": metrics.cpu_usage,
                                "threshold": threshold_percent
                            })),
                        }));
                    }
                } else {
                    self.state_store.clear_duration_state(node.id, &rule.rule_type).await;
                }
            }
            AlertRuleConfig::MemoryHigh { threshold_percent, duration_secs } => {
                if metrics.memory_usage > (*threshold_percent as f64) {
                    if self.state_store.check_duration_exceeded(
                        node.id,
                        &rule.rule_type,
                        *duration_secs
                    ).await {
                        return Ok(Some(Alert {
                            id: None,
                            rule_id: rule.id,
                            node_id: node.id,
                            node_name: node.name.clone(),
                            severity: rule.severity.clone(),
                            alert_type: rule.rule_type.clone(),
                            message: format!(
                                "内存使用率 {:.1}% 超过阈值 {:.1}%",
                                metrics.memory_usage, threshold_percent
                            ),
                            triggered_at: chrono::Utc::now().timestamp(),
                            resolved_at: None,
                            metadata: Some(serde_json::json!({
                                "node_is_public": node.is_public,
                                "memory_usage": metrics.memory_usage,
                                "threshold": threshold_percent
                            })),
                        }));
                    }
                } else {
                    self.state_store.clear_duration_state(node.id, &rule.rule_type).await;
                }
            }
            AlertRuleConfig::DiskFull { threshold_percent, mount_point } => {
                for disk in &metrics.disk_info {
                    if let Some(mp) = mount_point {
                        if &disk.mount != mp {
                            continue;
                        }
                    }

                    if disk.usage > (*threshold_percent as f64) {
                        return Ok(Some(Alert {
                            id: None,
                            rule_id: rule.id,
                            node_id: node.id,
                            node_name: node.name.clone(),
                            severity: rule.severity.clone(),
                            alert_type: rule.rule_type.clone(),
                            message: format!(
                                "磁盘 {} 使用率 {:.1}% 超过阈值 {:.1}%",
                                disk.mount, disk.usage, threshold_percent
                            ),
                            triggered_at: chrono::Utc::now().timestamp(),
                            resolved_at: None,
                            metadata: Some(serde_json::json!({
                                "node_is_public": node.is_public,
                                "mount": disk.mount,
                                "usage": disk.usage,
                                "threshold": threshold_percent
                            })),
                        }));
                    }
                }
            }
            _ => {
                // 其他规则类型暂不实现
            }
        }

        Ok(None)
    }

    /// 发送通知
    async fn send_notifications(&self, alert: &Alert, rule: &AlertRule) {
        for channel in &rule.notification_channels {
            match channel {
                NotificationChannel::Webhook { url, headers, template } => {
                    if let Err(e) = self.send_webhook(alert, url, headers, template).await {
                        tracing::error!("Failed to send webhook: {}", e);
                    }
                }
                NotificationChannel::WebSocket => {
                    let _ = self.broadcaster.broadcast(
                        vespera_common::ServerMessage::Alert(alert.to_ws_message())
                    );
                }
                NotificationChannel::Email { .. } => {
                    // TODO: Email 实现
                    tracing::warn!("Email notification not implemented yet");
                }
            }
        }
    }

    /// 发送 Webhook
    async fn send_webhook(
        &self,
        alert: &Alert,
        url: &str,
        headers: &Option<std::collections::HashMap<String, String>>,
        _template: &Option<String>,
    ) -> Result<(), AlertError> {
        let payload = serde_json::to_value(alert)
            .map_err(|e| AlertError::SerializationError(e.to_string()))?;

        let mut request = self.webhook_client.post(url).json(&payload);

        if let Some(hdrs) = headers {
            for (key, value) in hdrs {
                request = request.header(key, value);
            }
        }

        request.send().await
            .map_err(|e| AlertError::WebhookError(e.to_string()))?;

        Ok(())
    }
}

impl Clone for AlertEngine {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            state_store: self.state_store.clone(),
            webhook_client: self.webhook_client.clone(),
            broadcaster: self.broadcaster.clone(),
        }
    }
}

/// 告警错误
#[derive(thiserror::Error, Debug)]
pub enum AlertError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Webhook error: {0}")]
    WebhookError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
