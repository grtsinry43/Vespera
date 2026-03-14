//! 节点管理 API 处理器
//!
//! 提供节点查询和管理操作，区分普通用户和管理员权限

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use vespera_common::{
    AdminNode, DiskMetric, MetricsRangeQuery, NodeDetail, NodeMetrics, PublicNode,
    Response as ApiResponse, ServerError, UpdateNodeRequest, UpdateNodeVisibilityRequest,
};

use crate::{
    db::{
        error::DbError,
        models::{Metric, Node},
    },
    middleware::auth::{AdminUser, OptionalAuthUser},
    state::AppState,
};

/// 分页参数
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

// ============================================
// 普通用户接口 (公开信息)
// ============================================

/// 列出所有节点（公开信息）
///
/// GET /api/v1/nodes
///
/// **公开接口**：无需认证即可访问
#[utoipa::path(
    get,
    path = "/api/v1/nodes",
    params(
        ("limit" = i64, Query, description = "每页数量，默认 20"),
        ("offset" = i64, Query, description = "偏移量，默认 0")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<PublicNode>>))
    ),
    tag = "节点"
)]
pub async fn list_nodes(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<PublicNode>>>, ServerError> {
    let limit = query.limit.min(100).max(1); // 限制在 1-100 之间
    let offset = query.offset.max(0);

    let nodes = if auth_user.is_some() {
        state
            .db
            .list_nodes(limit, offset)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to list nodes: {}", e)))?
    } else {
        state
            .db
            .list_public_nodes(limit, offset)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to list nodes: {}", e)))?
    };

    // 转换为公开信息，并附带最新指标
    let mut public_nodes = Vec::with_capacity(nodes.len());
    for node in nodes {
        let mut public_node = node_to_public(node.clone());

        // 查询最新指标
        if let Ok(metrics) = state.db.get_latest_metrics(node.id, 1).await {
            if let Some(latest) = metrics.into_iter().next() {
                public_node.cpu_usage = Some(latest.cpu_usage);
                public_node.memory_usage = Some(latest.memory_usage);
                // 转换为 MB/s
                public_node.net_in = Some(latest.net_in_bytes as f64 / (1024.0 * 1024.0));
                public_node.net_out = Some(latest.net_out_bytes as f64 / (1024.0 * 1024.0));
            }
        }

        public_nodes.push(public_node);
    }

    Ok(Json(ApiResponse::success(public_nodes)))
}

/// 获取节点详情（公开信息）
///
/// GET /api/v1/nodes/:id
///
/// **公开接口**：无需认证即可访问
#[utoipa::path(
    get,
    path = "/api/v1/nodes/{id}",
    params(
        ("id" = i64, Path, description = "节点 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<NodeDetail<PublicNode>>)),
        (status = 404, description = "节点不存在")
    ),
    tag = "节点"
)]
pub async fn get_node(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
) -> Result<Json<ApiResponse<NodeDetail<PublicNode>>>, ServerError> {
    // 获取节点信息
    let node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    if auth_user.is_none() && !node.is_public {
        return Err(ServerError::NotFound("Node not found".to_string()));
    }

    // 获取最新指标
    let latest_metrics = state
        .db
        .get_latest_metrics(node_id, 1)
        .await
        .ok()
        .and_then(|metrics| metrics.into_iter().next())
        .map(metric_to_node_metrics);

    let detail = NodeDetail {
        node: node_to_public(node),
        latest_metrics,
    };

    Ok(Json(ApiResponse::success(detail)))
}

/// 获取节点历史指标
///
/// GET /api/v1/nodes/:id/metrics
///
/// **公开接口**：无需认证即可访问
#[utoipa::path(
    get,
    path = "/api/v1/nodes/{id}/metrics",
    params(
        ("id" = i64, Path, description = "节点 ID"),
        ("start" = i64, Query, description = "开始时间（Unix 时间戳）"),
        ("end" = i64, Query, description = "结束时间（Unix 时间戳）"),
        ("limit" = i64, Query, description = "返回条数，默认 100，最大 1000")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<NodeMetrics>>)),
        (status = 404, description = "节点不存在")
    ),
    tag = "节点"
)]
pub async fn get_node_metrics(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
    Query(query): Query<MetricsRangeQuery>,
) -> Result<Json<ApiResponse<Vec<NodeMetrics>>>, ServerError> {
    // 验证节点是否存在
    let node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    if auth_user.is_none() && !node.is_public {
        return Err(ServerError::NotFound("Node not found".to_string()));
    }

    // 限制查询条数
    let limit = query.limit.min(1000).max(1);

    // 查询指标
    let mut metrics = state
        .db
        .get_metrics_range(node_id, query.start, query.end, limit)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get metrics: {}", e)))?;

    // 如果 limit 设置为特殊值 24，则进行采样（用于 24h 图表）
    if query.limit == 24 && metrics.len() > 24 {
        metrics = sample_metrics(metrics, 24);
    }

    let node_metrics = metrics.into_iter().map(metric_to_node_metrics).collect();

    Ok(Json(ApiResponse::success(node_metrics)))
}

// ============================================
// 管理员接口 (完整信息)
// ============================================

/// 列出所有节点（完整信息）
///
/// GET /api/v1/admin/nodes
#[utoipa::path(
    get,
    path = "/api/v1/admin/nodes",
    params(
        ("limit" = i64, Query, description = "每页数量，默认 20"),
        ("offset" = i64, Query, description = "偏移量，默认 0")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<AdminNode>>)),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "节点管理"
)]
pub async fn admin_list_nodes(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<AdminNode>>>, ServerError> {
    let limit = query.limit.min(100).max(1);
    let offset = query.offset.max(0);

    let nodes = state
        .db
        .list_nodes(limit, offset)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to list nodes: {}", e)))?;

    // 转换为管理员视图
    let admin_nodes = nodes.into_iter().map(node_to_admin).collect();

    Ok(Json(ApiResponse::success(admin_nodes)))
}

/// 获取节点详情（完整信息）
///
/// GET /api/v1/admin/nodes/:id
#[utoipa::path(
    get,
    path = "/api/v1/admin/nodes/{id}",
    params(
        ("id" = i64, Path, description = "节点 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<NodeDetail<AdminNode>>)),
        (status = 404, description = "节点不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "节点管理"
)]
pub async fn admin_get_node(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
) -> Result<Json<ApiResponse<NodeDetail<AdminNode>>>, ServerError> {
    let node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    let latest_metrics = state
        .db
        .get_latest_metrics(node_id, 1)
        .await
        .ok()
        .and_then(|metrics| metrics.into_iter().next())
        .map(metric_to_node_metrics);

    let detail = NodeDetail {
        node: node_to_admin(node),
        latest_metrics,
    };

    Ok(Json(ApiResponse::success(detail)))
}

/// 更新节点信息
///
/// PUT /api/v1/admin/nodes/:id
#[utoipa::path(
    put,
    path = "/api/v1/admin/nodes/{id}",
    params(
        ("id" = i64, Path, description = "节点 ID")
    ),
    request_body = UpdateNodeRequest,
    responses(
        (status = 200, description = "更新成功", body = inline(vespera_common::Response<AdminNode>)),
        (status = 404, description = "节点不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "节点管理"
)]
pub async fn admin_update_node(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<ApiResponse<AdminNode>>, ServerError> {
    // 验证节点是否存在
    let _node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    // 准备更新数据
    let tags_json = req.tags.as_ref().and_then(|tags| {
        serde_json::to_string(tags).ok()
    });

    // 更新节点
    state
        .db
        .update_node(
            node_id,
            req.name.as_deref(),
            tags_json.as_deref(),
            req.is_public,
        )
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update node: {}", e)))?;

    // 返回更新后的节点
    let updated_node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get updated node: {}", e)))?
        .ok_or_else(|| ServerError::Internal("Node disappeared after update".to_string()))?;

    Ok(Json(ApiResponse::success(node_to_admin(updated_node))))
}

/// 更新节点公开状态
///
/// PUT /api/v1/admin/nodes/:id/visibility
#[utoipa::path(
    put,
    path = "/api/v1/admin/nodes/{id}/visibility",
    params(
        ("id" = i64, Path, description = "节点 ID")
    ),
    request_body = UpdateNodeVisibilityRequest,
    responses(
        (status = 200, description = "更新成功", body = inline(vespera_common::Response<AdminNode>)),
        (status = 404, description = "节点不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "节点管理"
)]
pub async fn admin_update_node_visibility(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
    Json(req): Json<UpdateNodeVisibilityRequest>,
) -> Result<Json<ApiResponse<AdminNode>>, ServerError> {
    // 更新节点可见性
    state
        .db
        .update_node(node_id, None, None, Some(req.is_public))
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update node visibility: {}", e)))?;

    // 返回更新后的节点
    let updated_node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to load node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    Ok(Json(ApiResponse::success(node_to_admin(updated_node))))
}

/// 删除节点
///
/// DELETE /api/v1/admin/nodes/:id
#[utoipa::path(
    delete,
    path = "/api/v1/admin/nodes/{id}",
    params(
        ("id" = i64, Path, description = "节点 ID")
    ),
    responses(
        (status = 200, description = "删除成功"),
        (status = 404, description = "节点不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "节点管理"
)]
pub async fn admin_delete_node(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    // 验证节点是否存在
    let _node = state
        .db
        .get_node_by_id(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get node: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Node not found".to_string()))?;

    // 删除节点（级联删除指标）
    state
        .db
        .delete_node(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to delete node: {}", e)))?;

    tracing::info!(node_id = node_id, "Node deleted by admin");

    Ok(Json(ApiResponse::success(())))
}

// ============================================
// 辅助函数
// ============================================

/// 将数据库 Node 转换为公开信息
fn node_to_public(node: Node) -> PublicNode {
    PublicNode {
        id: node.id,
        name: node.name,
        status: node.status,
        os_type: node.os_type,
        cpu_cores: node.cpu_cores,
        total_memory: node.total_memory,
        last_seen: node.last_seen,
        is_public: node.is_public,
        tags: node.tags.and_then(|json| serde_json::from_str(&json).ok()),
        // 这些字段在后面会被填充
        cpu_usage: None,
        memory_usage: None,
        net_in: None,
        net_out: None,
    }
}

/// 将数据库 Node 转换为管理员信息
fn node_to_admin(node: Node) -> AdminNode {
    AdminNode {
        id: node.id,
        uuid: node.uuid,
        name: node.name,
        ip_address: node.ip_address,
        agent_version: node.agent_version,
        os_type: node.os_type,
        os_version: node.os_version,
        cpu_cores: node.cpu_cores,
        total_memory: node.total_memory,
        status: node.status,
        last_seen: node.last_seen,
        created_at: node.created_at,
        updated_at: node.updated_at,
        is_public: node.is_public,
        tags: node.tags.and_then(|json| serde_json::from_str(&json).ok()),
    }
}

/// 将数据库 Metric 转换为 NodeMetrics
fn metric_to_node_metrics(metric: Metric) -> NodeMetrics {
    NodeMetrics {
        timestamp: metric.timestamp,
        cpu_usage: metric.cpu_usage,
        memory_used: metric.memory_used,
        memory_total: metric.memory_total,
        memory_usage: metric.memory_usage,
        disk_info: metric
            .disk_info
            .into_iter()
            .map(|d| DiskMetric {
                mount: d.mount,
                used: d.used,
                total: d.total,
                usage: d.usage,
            })
            .collect(),
        net_in_bytes: metric.net_in_bytes,
        net_out_bytes: metric.net_out_bytes,
        load_1: metric.load_1,
        load_5: metric.load_5,
        load_15: metric.load_15,
    }
}

/// 采样指标数据
///
/// 将大量指标数据均匀采样为指定数量的点
/// 算法：将时间区间分为 `sample_count` 段，每段取最近的一个点
fn sample_metrics(metrics: Vec<Metric>, sample_count: usize) -> Vec<Metric> {
    if metrics.len() <= sample_count {
        return metrics;
    }

    // 按时间排序（确保有序）
    let mut sorted_metrics = metrics;
    sorted_metrics.sort_by_key(|m| m.timestamp);

    if sorted_metrics.is_empty() {
        return sorted_metrics;
    }

    let start_time = sorted_metrics.first().unwrap().timestamp;
    let end_time = sorted_metrics.last().unwrap().timestamp;
    let time_range = end_time - start_time;

    if time_range == 0 {
        // 所有数据时间戳相同，直接返回前 sample_count 个
        return sorted_metrics.into_iter().take(sample_count).collect();
    }

    // 计算每个采样段的时间间隔
    let interval = time_range as f64 / sample_count as f64;

    let mut sampled = Vec::with_capacity(sample_count);
    let mut current_index = 0;

    // 对每个采样段，取该段内最后一个点（最接近段结束时间的点）
    for i in 0..sample_count {
        let bucket_end_time = start_time + ((i + 1) as f64 * interval) as i64;

        // 找到该段内的最后一个点
        while current_index < sorted_metrics.len()
            && sorted_metrics[current_index].timestamp <= bucket_end_time {
            current_index += 1;
        }

        // 回退一个，这是该段的最后一个点
        if current_index > 0 && current_index - 1 < sorted_metrics.len() {
            sampled.push(sorted_metrics[current_index - 1].clone());
        }
    }

    // 确保包含最后一个点
    if let Some(last) = sorted_metrics.last() {
        if sampled.last().map(|m| m.timestamp) != Some(last.timestamp) {
            if sampled.len() < sample_count {
                sampled.push(last.clone());
            } else {
                sampled[sample_count - 1] = last.clone();
            }
        }
    }

    sampled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::DiskInfo;

    #[test]
    fn test_node_to_public() {
        let node = Node {
            id: 1,
            uuid: "test-uuid".to_string(),
            name: "test-node".to_string(),
            ip_address: "192.168.1.1".to_string(),
            agent_version: "0.1.0".to_string(),
            os_type: "linux".to_string(),
            os_version: Some("Ubuntu 22.04".to_string()),
            cpu_cores: 8,
            total_memory: 17179869184,
            status: "online".to_string(),
            last_seen: 1234567890,
            created_at: 1234567890,
            updated_at: 1234567890,
            is_public: true,
            tags: Some(r#"["prod"]"#.to_string()),
        };

        let public = node_to_public(node);
        assert_eq!(public.name, "test-node");
        assert_eq!(public.tags, Some(vec!["prod".to_string()]));
        // 验证敏感信息不存在（编译时检查，公开结构体没有这些字段）
    }

    #[test]
    fn test_metric_to_node_metrics() {
        let metric = Metric {
            id: Some(1),
            node_id: 1,
            timestamp: 1234567890,
            cpu_usage: 45.2,
            cpu_cores: 8,
            memory_used: 8589934592,
            memory_total: 17179869184,
            memory_usage: 50.0,
            disk_info: vec![DiskInfo {
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
        };

        let node_metrics = metric_to_node_metrics(metric);
        assert_eq!(node_metrics.cpu_usage, 45.2);
        assert_eq!(node_metrics.disk_info.len(), 1);
    }
}
