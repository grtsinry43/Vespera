//! 认证 API 处理器
//!
//! 包含用户注册、登录、刷新 token、登出等功能

use axum::{extract::State, Json};
use base64::Engine;
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use vespera_common::{
    ChangePasswordRequest, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest, Response as ApiResponse, User,
};

use crate::{
    db::{DbRepo, UserRepoError},
    middleware::auth::AuthUser,
    utils::{create_jwt, hash_password, verify_password},
};

use crate::state::AppState;
use vespera_common::ServerError;

/// 生成随机 Refresh Token
fn generate_refresh_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
}

/// 用户注册
///
/// POST /api/v1/auth/register
///
/// # 限制
/// - 首次注册时可以创建管理员
/// - 后续注册只能创建普通用户
/// - 管理员可以通过用户管理 API 创建其他用户
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ServerError> {
    let db = &state.db;
    // 1. 检查是否为首次注册
    let user_count = db
        .users()
        .count_users()
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 2. 确定角色 (首次注册可以是 admin,否则只能是 user)
    let role = if user_count == 0 && req.is_admin {
        "admin"
    } else {
        "user"
    };

    // 3. 哈希密码
    let password_hash = hash_password(&req.password)
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))?;

    // 4. 创建用户
    let db_user = db
        .users()
        .create_user(
            &req.username,
            req.email.as_deref(),
            Some(&password_hash),
            role,
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

    // 5. 创建 JWT
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret-key-at-least-32-characters-long".to_string());

    let access_token = create_jwt(
        db_user.id,
        &db_user.role,
        Some(db_user.username.clone()),
        &jwt_secret,
        7, // 7 天
    )
    .map_err(|e| ServerError::Internal(format!("JWT creation failed: {}", e)))?;

    // 6. 创建 Refresh Token
    let refresh_token = generate_refresh_token();
    db.users()
        .create_refresh_token(db_user.id, &refresh_token, 30, None) // 30 天
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 7. 更新最后登录时间
    db.users()
        .update_last_login(db_user.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 8. 返回响应
    let expires_at = (Utc::now() + chrono::Duration::days(7)).timestamp();

    Ok(Json(ApiResponse::success(LoginResponse {
        access_token,
        refresh_token,
        user: db_user.to_public_user(),
        expires_at,
    })))
}

/// 用户登录
///
/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ServerError> {
    let db = &state.db;
    // 1. 查找用户
    let db_user = db
        .users()
        .find_by_username(&req.username)
        .await
        .map_err(|_| ServerError::Unauthorized("Invalid username or password".to_string()))?;

    // 2. 检查用户是否激活
    if !db_user.is_active {
        return Err(ServerError::Forbidden("Account is disabled".to_string()));
    }

    // 3. 验证密码
    let password_hash = db_user.password_hash.as_ref().ok_or_else(|| {
        ServerError::BadRequest("OAuth user cannot login with password".to_string())
    })?;

    let is_valid = verify_password(&req.password, password_hash)
        .map_err(|e| ServerError::Internal(format!("Password verification failed: {}", e)))?;

    if !is_valid {
        return Err(ServerError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    // 4. 创建 JWT
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret-key-at-least-32-characters-long".to_string());

    let access_token = create_jwt(
        db_user.id,
        &db_user.role,
        Some(db_user.username.clone()),
        &jwt_secret,
        7,
    )
    .map_err(|e| ServerError::Internal(format!("JWT creation failed: {}", e)))?;

    // 5. 创建 Refresh Token
    let refresh_token = generate_refresh_token();
    db.users()
        .create_refresh_token(db_user.id, &refresh_token, 30, None)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 6. 更新最后登录时间
    db.users()
        .update_last_login(db_user.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 7. 返回响应
    let expires_at = (Utc::now() + chrono::Duration::days(7)).timestamp();

    Ok(Json(ApiResponse::success(LoginResponse {
        access_token,
        refresh_token,
        user: db_user.to_public_user(),
        expires_at,
    })))
}

/// 刷新 Access Token
///
/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<RefreshTokenResponse>>, ServerError> {
    let db = &state.db;
    // 1. 验证 Refresh Token
    let refresh_token_record = db
        .users()
        .verify_refresh_token(&req.refresh_token)
        .await
        .map_err(|_| ServerError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    // 2. 查找用户
    let db_user = db
        .users()
        .find_by_id(refresh_token_record.user_id)
        .await
        .map_err(|_| ServerError::Unauthorized("User not found".to_string()))?;

    // 3. 检查用户是否激活
    if !db_user.is_active {
        return Err(ServerError::Forbidden("Account is disabled".to_string()));
    }

    // 4. 创建新的 JWT
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret-key-at-least-32-characters-long".to_string());

    let access_token = create_jwt(
        db_user.id,
        &db_user.role,
        Some(db_user.username.clone()),
        &jwt_secret,
        7,
    )
    .map_err(|e| ServerError::Internal(format!("JWT creation failed: {}", e)))?;

    // 5. 更新 Refresh Token 最后使用时间
    db.users()
        .update_refresh_token_last_used(refresh_token_record.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 6. 返回响应 (不做 token rotation)
    let expires_at = (Utc::now() + chrono::Duration::days(7)).timestamp();

    Ok(Json(ApiResponse::success(RefreshTokenResponse {
        access_token,
        refresh_token: None, // 不进行 token rotation
        expires_at,
    })))
}

/// 登出
///
/// POST /api/v1/auth/logout
pub async fn logout(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    let db = &state.db;
    // 删除 Refresh Token
    db.users()
        .delete_refresh_token(&req.refresh_token)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    tracing::info!("User {} logged out", auth.username);

    Ok(Json(ApiResponse::success(())))
}

/// 获取当前用户信息
///
/// GET /api/v1/auth/me
pub async fn me(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<User>>, ServerError> {
    let db = &state.db;
    let db_user = db
        .users()
        .find_by_id(auth.id)
        .await
        .map_err(|_| ServerError::Unauthorized("User not found".to_string()))?;

    Ok(Json(ApiResponse::success(db_user.to_public_user())))
}

/// 修改密码
///
/// POST /api/v1/auth/change-password
pub async fn change_password(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    let db = &state.db;
    // 1. 查找用户
    let db_user = db
        .users()
        .find_by_id(auth.id)
        .await
        .map_err(|_| ServerError::Internal("User not found".to_string()))?;

    // 2. 验证旧密码
    let password_hash = db_user
        .password_hash
        .as_ref()
        .ok_or_else(|| ServerError::BadRequest("OAuth user cannot change password".to_string()))?;

    let is_valid = verify_password(&req.old_password, password_hash)
        .map_err(|e| ServerError::Internal(format!("Password verification failed: {}", e)))?;

    if !is_valid {
        return Err(ServerError::Unauthorized(
            "Incorrect old password".to_string(),
        ));
    }

    // 3. 哈希新密码
    let new_password_hash = hash_password(&req.new_password)
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))?;

    // 4. 更新密码
    db.users()
        .update_password(auth.id, &new_password_hash)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 5. 删除所有 Refresh Tokens (强制重新登录)
    db.users()
        .delete_user_refresh_tokens(auth.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    tracing::info!("User {} changed password", auth.username);

    Ok(Json(ApiResponse::success(())))
}
