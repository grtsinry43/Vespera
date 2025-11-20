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
    ServiceStatusOverview, ServiceStatusPoint, ServiceUpdate,
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
/// **权限**: 需要认证（普通用户可查看）
pub async fn list_services(
    _auth: OptionalAuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Service>>>, ServerError> {
    let services = state
        .db
        .services()
        .list_services()
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(services)))
}

/// 获取服务详情
///
/// GET /api/v1/services/:id
///
/// **权限**: 需要认证
pub async fn get_service(
    _auth: OptionalAuthUser,
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
/// **权限**: 需要认证
pub async fn get_service_status(
    _auth: OptionalAuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<ServiceStatusPoint>>>, ServerError> {
    // 先检查服务是否存在
    let _service = state
        .db
        .services()
        .get_service(id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .ok_or_else(|| ServerError::NotFound("Service not found".to_string()))?;

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
/// **权限**: 公开接口
pub async fn get_service_overview(
    _auth: OptionalAuthUser,
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
/// **权限**: 公开接口，适合前端监控面板展示
pub async fn get_all_services_overview(
    _auth: OptionalAuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<ServiceStatusOverview>>>, ServerError> {
    // 获取所有服务
    let services = state
        .db
        .services()
        .list_services()
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

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
