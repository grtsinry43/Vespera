//! 告警管理 API 处理器
//!
//! 提供告警规则的 CRUD 操作和告警历史查询

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use vespera_common::{Response as ApiResponse, ServerError, UserRole};

use crate::{
    alert::models::{Alert, AlertRule},
    db::{
        alert_repo::{AlertRepository, AlertRuleCreate, AlertRuleUpdate},
        error::DbError,
    },
    middleware::auth::AuthUser,
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

/// 列出所有告警规则
///
/// GET /api/v1/alerts/rules
#[utoipa::path(
    get,
    path = "/api/v1/alerts/rules",
    params(
        ("limit" = i64, Query, description = "每页数量，默认 20"),
        ("offset" = i64, Query, description = "偏移量，默认 0")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<AlertRule>>)),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn list_rules(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<AlertRule>>>, ServerError> {
    let limit = query.limit.min(100).max(1); // 限制在 1-100 之间
    let offset = query.offset.max(0);

    let rules = state
        .db
        .alerts()
        .list_rules(limit, offset)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to list rules: {}", e)))?;

    Ok(Json(ApiResponse::success(rules)))
}

/// 获取单个告警规则
///
/// GET /api/v1/alerts/rules/:id
#[utoipa::path(
    get,
    path = "/api/v1/alerts/rules/{id}",
    params(
        ("id" = i64, Path, description = "告警规则 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<AlertRule>)),
        (status = 404, description = "规则不存在"),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn get_rule(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<AlertRule>>, ServerError> {
    let rule = state
        .db
        .alerts()
        .get_rule(id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get rule: {}", e)))?
        .ok_or_else(|| ServerError::NotFound(format!("Alert rule {} not found", id)))?;

    Ok(Json(ApiResponse::success(rule)))
}

/// 创建告警规则
///
/// POST /api/v1/alerts/rules
#[utoipa::path(
    post,
    path = "/api/v1/alerts/rules",
    request_body = AlertRuleCreate,
    responses(
        (status = 200, description = "创建成功", body = inline(vespera_common::Response<AlertRule>)),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未认证"),
        (status = 403, description = "需要管理员权限")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn create_rule(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AlertRuleCreate>,
) -> Result<Json<ApiResponse<AlertRule>>, ServerError> {
    // 只有管理员可以创建规则
    if auth.role != UserRole::Admin {
        return Err(ServerError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    let rule = state
        .db
        .alerts()
        .create_rule(&req)
        .await
        .map_err(|e| match e {
            DbError::ParseError(msg) => ServerError::BadRequest(msg),
            _ => ServerError::Internal(format!("Failed to create rule: {}", e)),
        })?;

    tracing::info!(
        "User {} created alert rule: {} (id={})",
        auth.username,
        rule.name,
        rule.id
    );

    Ok(Json(ApiResponse::success(rule)))
}

/// 更新告警规则
///
/// PUT /api/v1/alerts/rules/:id
#[utoipa::path(
    put,
    path = "/api/v1/alerts/rules/{id}",
    params(
        ("id" = i64, Path, description = "告警规则 ID")
    ),
    request_body = AlertRuleUpdate,
    responses(
        (status = 200, description = "更新成功", body = inline(vespera_common::Response<AlertRule>)),
        (status = 404, description = "规则不存在"),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未认证"),
        (status = 403, description = "需要管理员权限")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn update_rule(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<AlertRuleUpdate>,
) -> Result<Json<ApiResponse<AlertRule>>, ServerError> {
    // 只有管理员可以更新规则
    if auth.role != UserRole::Admin {
        return Err(ServerError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    let rule = state
        .db
        .alerts()
        .update_rule(id, &req)
        .await
        .map_err(|e| match e {
            DbError::NotFound => ServerError::NotFound(format!("Alert rule {} not found", id)),
            DbError::ParseError(msg) => ServerError::BadRequest(msg),
            _ => ServerError::Internal(format!("Failed to update rule: {}", e)),
        })?;

    tracing::info!(
        "User {} updated alert rule: {} (id={})",
        auth.username,
        rule.name,
        rule.id
    );

    Ok(Json(ApiResponse::success(rule)))
}

/// 删除告警规则
///
/// DELETE /api/v1/alerts/rules/:id
#[utoipa::path(
    delete,
    path = "/api/v1/alerts/rules/{id}",
    params(
        ("id" = i64, Path, description = "告警规则 ID")
    ),
    responses(
        (status = 200, description = "删除成功"),
        (status = 404, description = "规则不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "需要管理员权限")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn delete_rule(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    // 只有管理员可以删除规则
    if auth.role != UserRole::Admin {
        return Err(ServerError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    state
        .db
        .alerts()
        .delete_rule(id)
        .await
        .map_err(|e| match e {
            DbError::NotFound => ServerError::NotFound(format!("Alert rule {} not found", id)),
            _ => ServerError::Internal(format!("Failed to delete rule: {}", e)),
        })?;

    tracing::info!("User {} deleted alert rule id={}", auth.username, id);

    Ok(Json(ApiResponse::success(())))
}

/// 获取活跃告警列表
///
/// GET /api/v1/alerts
#[utoipa::path(
    get,
    path = "/api/v1/alerts",
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<Alert>>)),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn list_alerts(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Alert>>>, ServerError> {
    let alerts = state
        .db
        .alerts()
        .get_active_alerts()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to list alerts: {}", e)))?;

    Ok(Json(ApiResponse::success(alerts)))
}

/// 获取节点的活跃告警
///
/// GET /api/v1/alerts/node/:node_id
#[utoipa::path(
    get,
    path = "/api/v1/alerts/node/{node_id}",
    params(
        ("node_id" = i64, Path, description = "节点 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<Vec<Alert>>)),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn list_node_alerts(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<Alert>>>, ServerError> {
    let alerts = state
        .db
        .alerts()
        .get_active_alerts_for_node(node_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to list node alerts: {}", e)))?;

    Ok(Json(ApiResponse::success(alerts)))
}

/// 解决告警
///
/// POST /api/v1/alerts/:id/resolve
#[utoipa::path(
    post,
    path = "/api/v1/alerts/{id}/resolve",
    params(
        ("id" = i64, Path, description = "告警 ID")
    ),
    responses(
        (status = 200, description = "解决成功"),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "告警"
)]
pub async fn resolve_alert(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    state
        .db
        .alerts()
        .resolve_alert(id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to resolve alert: {}", e)))?;

    tracing::info!("User {} resolved alert id={}", auth.username, id);

    Ok(Json(ApiResponse::success(())))
}
