//! 用户管理 API (管理员专用)

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use vespera_common::{
    CreateUserRequest, ResetPasswordRequest, Response as ApiResponse, ServerError,
    UpdateUserRequest, User,
};

use crate::{
    db::{DbRepo, UserRepoError},
    middleware::auth::AdminUser,
    utils::hash_password,
};
use crate::state::AppState;

/// 列出所有用户
///
/// GET /api/v1/users
pub async fn list_users(
    _admin: AdminUser, // 需要管理员权限
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<User>>>, ServerError> {
    let db = &state.db;
    let db_users = db
        .users()
        .list_users(100, 0)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let users = db_users.into_iter().map(|u| u.to_public_user()).collect();

    Ok(Json(ApiResponse::success(users)))
}

/// 获取用户详情
///
/// GET /api/v1/users/:id
pub async fn get_user(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<ApiResponse<User>>, ServerError> {
    let db = &state.db;
    let db_user = db.users().find_by_id(user_id).await.map_err(|e| match e {
        UserRepoError::UserNotFound => ServerError::NotFound("User not found".to_string()),
        _ => ServerError::Internal(e.to_string()),
    })?;

    Ok(Json(ApiResponse::success(db_user.to_public_user())))
}

/// 创建用户
///
/// POST /api/v1/users
pub async fn create_user(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<User>>, ServerError> {
    let db = &state.db;
    // 1. 哈希密码
    let password_hash = hash_password(&req.password)
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))?;

    // 2. 创建用户
    let db_user = db
        .users()
        .create_user(
            &req.username,
            req.email.as_deref(),
            Some(&password_hash),
            req.role.as_str(),
        )
        .await
        .map_err(|e| match e {
            UserRepoError::UsernameExists => {
                ServerError::BadRequest("Username already exists".to_string())
            }
            UserRepoError::EmailExists => {
                ServerError::BadRequest("Email already exists".to_string())
            }
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(db_user.to_public_user())))
}

/// 更新用户
///
/// PUT /api/v1/users/:id
pub async fn update_user(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<User>>, ServerError> {
    let db = &state.db;
    let db_user = db
        .users()
        .update_user(
            user_id,
            req.email.as_deref(),
            req.avatar_url.as_deref(),
            req.is_active,
            req.role.as_ref().map(|r| r.as_str()),
        )
        .await
        .map_err(|e| match e {
            UserRepoError::UserNotFound => ServerError::NotFound("User not found".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    Ok(Json(ApiResponse::success(db_user.to_public_user())))
}

/// 删除用户
///
/// DELETE /api/v1/users/:id
pub async fn delete_user(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    let db = &state.db;
    // 防止删除自己
    if admin.0.id == user_id {
        return Err(ServerError::BadRequest(
            "Cannot delete your own account".to_string(),
        ));
    }

    db.users()
        .delete_user(user_id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(())))
}

/// 重置用户密码 (管理员)
///
/// POST /api/v1/users/:id/reset-password
pub async fn reset_password(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    let db = &state.db;
    // 1. 哈希新密码
    let new_password_hash = hash_password(&req.new_password)
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))?;

    // 2. 更新密码
    db.users()
        .update_password(user_id, &new_password_hash)
        .await
        .map_err(|e| match e {
            UserRepoError::UserNotFound => ServerError::NotFound("User not found".to_string()),
            _ => ServerError::Internal(e.to_string()),
        })?;

    // 3. 删除该用户的所有 Refresh Tokens (强制重新登录)
    db.users()
        .delete_user_refresh_tokens(user_id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(())))
}
