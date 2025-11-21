use axum::{extract::State, Json};
use std::sync::Arc;

use vespera_common::{ReportRequest, Response, ServerMessage, MetricsUpdate, DiskInfoWs};
use crate::{
    db::models::{DiskInfo, Metric, NodeCreate},
    error::AppError,
    state::AppState,
};

/// Agent 数据上报端点
///
/// POST /api/v1/report
///
/// # 功能
/// - 接收 Agent 上报的节点信息和监控指标
/// - 首次上报时自动注册节点
/// - 已存在节点更新 last_seen 和插入指标
/// - 实时广播指标更新到所有 WebSocket 连接
///
/// # 安全
/// - 需要通过 verify_agent_token 中间件验证
///
/// # 性能
/// - 目标处理时间: < 10ms
/// - 使用索引查询 (uuid)
/// - 单次事务处理
pub async fn report_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<Response<()>>, AppError> {
    let start = std::time::Instant::now();

    tracing::debug!(
        node_uuid = %req.node_uuid,
        node_name = %req.node_name,
        "Received metrics report"
    );

    // 转换 UUID 为字符串（用于数据库查询）
    let uuid_str = req.node_uuid.to_string();

    // 1. 检查节点是否存在
    let node = state.db.get_node_by_uuid(&uuid_str).await?;

    let (node_id, node_name, node_is_public, is_new_node) = match node {
        Some(existing_node) => {
            // 节点已存在：更新 last_seen
            tracing::debug!(
                node_id = existing_node.id,
                node_uuid = %req.node_uuid,
                "Updating existing node"
            );

            state
                .db
                .update_node_status(
                    existing_node.id,
                    "online",
                    req.metrics.timestamp,
                )
                .await?;

            (
                existing_node.id,
                existing_node.name,
                existing_node.is_public,
                false,
            )
        }
        None => {
            // 首次上报：创建新节点
            tracing::info!(
                node_uuid = %req.node_uuid,
                node_name = %req.node_name,
                ip_address = %req.ip_address,
                "Registering new node"
            );

            let node_create = NodeCreate {
                uuid: uuid_str.clone(),
                name: req.node_name.clone(),
                ip_address: req.ip_address.clone(),
                agent_version: req.agent_version.clone(),
                os_type: req.os_type.clone(),
                os_version: req.os_version.clone(),
                cpu_cores: req.cpu_cores,
                total_memory: req.total_memory,
                is_public: false,
                tags: req.tags.clone(),
            };

            let created_node = state.db.create_node(&node_create).await?;
            (
                created_node.id,
                created_node.name,
                created_node.is_public,
                true,
            )
        }
    };

    // 2. 插入监控指标
    let metric = Metric {
        id: None,
        node_id,
        timestamp: req.metrics.timestamp,
        cpu_usage: req.metrics.cpu_usage,
        cpu_cores: req.cpu_cores,
        memory_used: req.metrics.memory_used,
        memory_total: req.total_memory,
        memory_usage: req.metrics.memory_usage,
        disk_info: req
            .metrics
            .disk_info
            .iter()
            .map(|d| DiskInfo {
                mount: d.mount.clone(),
                used: d.used,
                total: d.total,
                usage: d.usage,
            })
            .collect(),
        net_in_bytes: req.metrics.net_in_bytes,
        net_out_bytes: req.metrics.net_out_bytes,
        load_1: req.metrics.load_1,
        load_5: req.metrics.load_5,
        load_15: req.metrics.load_15,
    };

    state.db.insert_metrics(&metric).await?;

    // 2.5 评估告警规则
    if let Some(alert_engine) = &state.alert_engine {
        let tags_json = match &req.tags {
            Some(tags) => serde_json::to_string(tags).ok(),
            None => None,
        };

        let node_for_alert = crate::db::models::Node {
            id: node_id,
            uuid: uuid_str.clone(),
            name: node_name.clone(),
            ip_address: req.ip_address.clone(),
            agent_version: req.agent_version.clone(),
            os_type: req.os_type.clone(),
            os_version: req.os_version.clone(),
            cpu_cores: req.cpu_cores,
            total_memory: req.total_memory,
            status: "online".to_string(),
            last_seen: req.metrics.timestamp,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            is_public: node_is_public,
            tags: tags_json,
        };

        if let Err(e) = alert_engine.evaluate_metrics(&node_for_alert, &metric).await {
            tracing::error!("Failed to evaluate alerts: {:?}", e);
        }
    }

    // 3. 广播到 WebSocket 连接
    let ws_update = MetricsUpdate {
        node_id,
        node_uuid: uuid_str.clone(),
        node_name: node_name.clone(),
        timestamp: req.metrics.timestamp,
        cpu_usage: req.metrics.cpu_usage as f32,
        memory_usage: req.metrics.memory_usage as f32,
        memory_used: req.metrics.memory_used,
        memory_total: req.total_memory,
        disk_info: req
            .metrics
            .disk_info
            .iter()
            .map(|d| DiskInfoWs {
                mount: d.mount.clone(),
                used: d.used,
                total: d.total,
                usage: d.usage as f32,
            })
            .collect(),
        network_in: req.metrics.net_in_bytes,
        network_out: req.metrics.net_out_bytes,
        load_1: req.metrics.load_1.map(|v| v as f32),
        load_5: req.metrics.load_5.map(|v| v as f32),
        load_15: req.metrics.load_15.map(|v| v as f32),
    };

    // 广播指标更新
    let broadcast_result = state.broadcaster.broadcast(ServerMessage::MetricsUpdate(ws_update));

    // 记录广播结果 (不影响响应)
    match broadcast_result {
        Ok(n) => {
            tracing::debug!(
                node_id = node_id,
                receiver_count = n,
                "Metrics update broadcasted to {} WebSocket clients",
                n
            );
        }
        Err(_) => {
            tracing::debug!(
                node_id = node_id,
                "No WebSocket clients connected, metrics not broadcasted"
            );
        }
    }

    // 如果是新节点,广播上线事件
    if is_new_node {
        let _ = state.broadcaster.broadcast(ServerMessage::NodeOnline {
            node_id,
            node_name,
        });
    }

    let elapsed = start.elapsed();
    tracing::debug!(
        node_id = node_id,
        elapsed_ms = elapsed.as_millis(),
        "Metrics report processed"
    );

    // 性能监控：如果处理时间 > 10ms，记录警告
    if elapsed.as_millis() > 10 {
        tracing::warn!(
            node_id = node_id,
            elapsed_ms = elapsed.as_millis(),
            "Report processing exceeded 10ms threshold"
        );
    }

    Ok(Json(Response {
        code: 0,
        data: Some(()),
        msg: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vespera_common::request::{MetricsData, DiskInfo as RequestDiskInfo};
    use crate::test_utils::create_test_db;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_first_report_creates_node() {
        // 使用内存数据库，避免并发测试时的文件锁冲突
        let db = create_test_db().await;
        let state = Arc::new(AppState::new(db));

        // 构造首次上报请求
        let request = ReportRequest {
            node_uuid: Uuid::new_v4(),
            node_name: "Test Node".to_string(),
            ip_address: "192.168.1.100".to_string(),
            agent_version: "0.1.0".to_string(),
            os_type: "linux".to_string(),
            os_version: Some("Ubuntu 22.04".to_string()),
            cpu_cores: 8,
            total_memory: 17179869184,
            tags: Some(vec!["test".to_string()]),
            metrics: MetricsData {
                timestamp: chrono::Utc::now().timestamp(),
                cpu_usage: 45.2,
                memory_used: 8589934592,
                memory_usage: 50.0,
                disk_info: vec![RequestDiskInfo {
                    mount: "/".to_string(),
                    used: 107374182400,
                    total: 214748364800,
                    usage: 50.0,
                }],
                net_in_bytes: 1073741824,
                net_out_bytes: 2147483648,
                load_1: Some(1.5),
                load_5: Some(1.2),
                load_15: Some(1.0),
            },
        };

        // 调用 Handler
        let result = report_handler(State(state.clone()), Json(request.clone())).await;
        assert!(result.is_ok());

        // 验证节点已创建
        let node = state
            .db
            .get_node_by_uuid(&request.node_uuid.to_string())
            .await
            .expect("Failed to query node")
            .expect("Node not created");

        assert_eq!(node.name, "Test Node");
        assert_eq!(node.ip_address, "192.168.1.100");
    }
}
