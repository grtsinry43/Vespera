// 服务监控 API 处理器
//!
//! 提供服务配置和状态查询的 API

use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use vespera_common::{
    Response as ApiResponse, ServerError, Service, ServiceCheckResult, ServiceCreate,
    ServiceStatusOverview, ServiceStatusPoint, ServiceUpdate, UpdateServiceVisibilityRequest,
};

use crate::{
    db::{error::DbError, service_repo::ServiceRepository},
    middleware::auth::{AdminUser, OptionalAuthUser},
    state::AppState,
};

// ============================================
// 服务管理接口（管理员权限）
// ============================================

/// 创建服务
///
/// POST /api/v1/services
///
/// **权限**: 需要管理员权限
pub async fn create_service(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ServiceCreate>,
) -> Result<Json<ApiResponse<Service>>, ServerError> {
    let service = state
        .db
        .services()
        .create_service(&payload)
        .await
        .map_err(|e| match e {
            DbError::Conflict(_) => ServerError::Conflict("Service already exists".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(service)))
}

/// 获取所有服务
///
/// GET /api/v1/services
///
/// **权限**: 未登录仅返回公开服务，登录后返回全部
pub async fn list_services(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Service>>>, ServerError> {
    let repo = state.db.services();
    let services = if auth_user.is_some() {
        repo.list_services()
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?
    } else {
        repo.list_public_services()
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?
    };

    Ok(Json(ApiResponse::success(services)))
}

/// 获取服务详情
///
/// GET /api/v1/services/:id
///
/// **权限**: 未登录仅可访问公开服务
pub async fn get_service(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Service>>, ServerError> {
    let service = state
        .db
        .services()
        .get_service(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .ok_or_else(|| ServerError::NotFound("Service not found".to_string()))?;

    if auth_user.is_none() && !service.is_public {
        return Err(ServerError::NotFound("Service not found".to_string()));
    }

    Ok(Json(ApiResponse::success(service)))
}

/// 更新服务
///
/// PUT /api/v1/services/:id
///
/// **权限**: 需要管理员权限
pub async fn update_service(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<ServiceUpdate>,
) -> Result<Json<ApiResponse<Service>>, ServerError> {
    let service = state
        .db
        .services()
        .update_service(id, &payload)
        .await
        .map_err(|e| match e {
            DbError::NotFound => ServerError::NotFound("Service not found".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(service)))
}

/// 更新服务公开状态
///
/// PUT /api/v1/services/:id/visibility
#[utoipa::path(
    put,
    path = "/api/v1/services/{id}/visibility",
    params(
        ("id" = i64, Path, description = "服务 ID")
    ),
    request_body = UpdateServiceVisibilityRequest,
    responses(
        (status = 200, description = "更新成功", body = inline(vespera_common::Response<vespera_common::Service>)),
        (status = 404, description = "服务不存在"),
        (status = 401, description = "未认证"),
        (status = 403, description = "权限不足")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "服务"
)]
pub async fn update_service_visibility(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateServiceVisibilityRequest>,
) -> Result<Json<ApiResponse<Service>>, ServerError> {
    let update = ServiceUpdate {
        name: None,
        target: None,
        check_interval: None,
        timeout: None,
        method: None,
        expected_code: None,
        expected_body: None,
        headers: None,
        enabled: None,
        is_public: Some(req.is_public),
    };

    let service = state
        .db
        .services()
        .update_service(id, &update)
        .await
        .map_err(|e| match e {
            DbError::NotFound => ServerError::NotFound("Service not found".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(service)))
}

/// 删除服务
///
/// DELETE /api/v1/services/:id
///
/// **权限**: 需要管理员权限
pub async fn delete_service(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    state
        .db
        .services()
        .delete_service(id)
        .await
        .map_err(|e| match e {
            DbError::NotFound => ServerError::NotFound("Service not found".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(())))
}

// ============================================
// 服务状态查询接口
// ============================================

/// 获取服务状态历史（最近30小时）
///
/// GET /api/v1/services/:id/status
///
/// **权限**: 未登录仅可访问公开服务
pub async fn get_service_status(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<ServiceStatusPoint>>>, ServerError> {
    // 先检查服务是否存在
    let service = state
        .db
        .services()
        .get_service(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .ok_or_else(|| ServerError::NotFound("Service not found".to_string()))?;

    if auth_user.is_none() && !service.is_public {
        return Err(ServerError::NotFound("Service not found".to_string()));
    }

    let history = state
        .db
        .services()
        .get_status_history(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(history)))
}

/// 获取服务状态概览（服务信息 + 状态历史）
///
/// GET /api/v1/services/:id/overview
///
/// **权限**: 未登录仅可访问公开服务
pub async fn get_service_overview(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ServiceStatusOverview>>, ServerError> {
    let service = state
        .db
        .services()
        .get_service(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .ok_or_else(|| ServerError::NotFound("Service not found".to_string()))?;

    if auth_user.is_none() && !service.is_public {
        return Err(ServerError::NotFound("Service not found".to_string()));
    }

    let history = state
        .db
        .services()
        .get_status_history(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 获取最新状态
    let current_status = state
        .db
        .services()
        .get_latest_status(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .map(|s| s.status)
        .unwrap_or(vespera_common::ServiceStatus::Unknown);

    let overview = ServiceStatusOverview {
        service,
        current_status,
        history,
    };

    Ok(Json(ApiResponse::success(overview)))
}

/// 获取所有服务状态概览（前端监控面板用）
///
/// GET /api/v1/services/all/overview
///
/// **权限**: 未登录仅返回公开服务
pub async fn get_all_services_overview(
    OptionalAuthUser(auth_user): OptionalAuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<ServiceStatusOverview>>>, ServerError> {
    let repo = state.db.services();
    // 根据认证状态选择公开或全部服务
    let services = if auth_user.is_some() {
        repo.list_services()
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?
    } else {
        repo.list_public_services()
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?
    };

    let mut overviews = Vec::new();

    // 为每个服务获取状态概览
    for service in services {
        let history = state
            .db
            .services()
            .get_status_history(service.id)
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        // 获取最新状态
        let current_status = state
            .db
            .services()
            .get_latest_status(service.id)
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?
            .map(|s| s.status)
            .unwrap_or(vespera_common::ServiceStatus::Unknown);

        overviews.push(ServiceStatusOverview {
            service,
            current_status,
            history,
        });
    }

    Ok(Json(ApiResponse::success(overviews)))
}

// ============================================
// Agent 接口（用于上报服务状态）
// ============================================

/// Agent 获取需要检查的服务列表
///
/// GET /api/v1/agent/services
///
/// **权限**: Agent 认证（后续实现）
pub async fn agent_get_services(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Service>>>, ServerError> {
    let services = state
        .db
        .services()
        .list_enabled_services()
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(services)))
}

/// Agent 上报服务检查结果
///
/// POST /api/v1/agent/service-status
///
/// **权限**: Agent 认证（后续实现）
pub async fn agent_report_status(
    State(state): State<Arc<AppState>>,
    Json(results): Json<Vec<ServiceCheckResult>>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    state
        .db
        .services()
        .record_check_results(&results)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    tracing::debug!("Received {} service check results", results.len());

    Ok(Json(ApiResponse::success(())))
}
