//! 认证 API 处理器
//!
//! 包含用户注册、登录、刷新 token、登出等功能

use axum::{
    extract::State,
    http::{header::COOKIE, header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use vespera_common::{
    ChangePasswordRequest, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest, Response as ApiResponse, User,
};

use crate::{
    db::UserRepoError,
    middleware::auth::AuthUser,
    utils::{create_jwt, hash_password, jwt_secret_from_env, verify_password},
};

use crate::state::AppState;
use vespera_common::ServerError;

const REFRESH_TOKEN_COOKIE: &str = "vespera_refresh_token";
const REFRESH_TOKEN_COOKIE_MAX_AGE: i64 = 30 * 24 * 60 * 60;
const ACCESS_TOKEN_COOKIE: &str = "vespera_access_token";
const ACCESS_TOKEN_COOKIE_MAX_AGE: i64 = 7 * 24 * 60 * 60;

fn refresh_cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn build_refresh_cookie(token: &str) -> String {
    let secure = if refresh_cookie_secure() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{REFRESH_TOKEN_COOKIE}={token}; Max-Age={REFRESH_TOKEN_COOKIE_MAX_AGE}; Path=/; HttpOnly; SameSite=Strict{secure}"
    )
}

fn build_access_cookie(token: &str) -> String {
    let secure = if refresh_cookie_secure() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{ACCESS_TOKEN_COOKIE}={token}; Max-Age={ACCESS_TOKEN_COOKIE_MAX_AGE}; Path=/; HttpOnly; SameSite=Strict{secure}"
    )
}

fn clear_refresh_cookie() -> String {
    let secure = if refresh_cookie_secure() {
        "; Secure"
    } else {
        ""
    };
    format!("{REFRESH_TOKEN_COOKIE}=; Max-Age=0; Path=/; HttpOnly; SameSite=Strict{secure}")
}

fn clear_access_cookie() -> String {
    let secure = if refresh_cookie_secure() {
        "; Secure"
    } else {
        ""
    };
    format!("{ACCESS_TOKEN_COOKIE}=; Max-Age=0; Path=/; HttpOnly; SameSite=Strict{secure}")
}

fn append_cookie(headers: &mut HeaderMap, cookie: String) -> Result<(), ServerError> {
    let value = HeaderValue::from_str(&cookie)
        .map_err(|_| ServerError::Internal("Failed to encode auth cookie".to_string()))?;
    headers.append(SET_COOKIE, value);
    Ok(())
}

fn extract_refresh_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;

    cookie_header.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        if name == REFRESH_TOKEN_COOKIE && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn resolve_refresh_token(
    headers: &HeaderMap,
    req: &RefreshTokenRequest,
) -> Result<String, ServerError> {
    req.refresh_token
        .clone()
        .or_else(|| extract_refresh_token_from_cookie(headers))
        .ok_or_else(|| ServerError::Unauthorized("Missing refresh token".to_string()))
}

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
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册成功", body = inline(vespera_common::Response<vespera_common::LoginResponse>)),
        (status = 400, description = "用户名或邮箱已存在")
    ),
    tag = "认证"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Response, ServerError> {
    let db = &state.db;
    // 1. 哈希密码
    let password_hash = hash_password(&req.password)
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))?;

    // 2. 创建用户。角色在 SQL 内原子决定，避免首次管理员注册竞态。
    let db_user = db
        .users()
        .create_registered_user(
            &req.username,
            req.email.as_deref(),
            &password_hash,
            req.is_admin,
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

    // 3. 创建 JWT
    let jwt_secret = jwt_secret_from_env()
        .map_err(|e| ServerError::Internal(format!("JWT configuration failed: {}", e)))?;

    let access_token = create_jwt(
        db_user.id,
        &db_user.role,
        Some(db_user.username.clone()),
        &jwt_secret,
        7, // 7 天
    )
    .map_err(|e| ServerError::Internal(format!("JWT creation failed: {}", e)))?;

    // 4. 创建 Refresh Token
    let refresh_token = generate_refresh_token();
    db.users()
        .create_refresh_token(db_user.id, &refresh_token, 30, None) // 30 天
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 5. 更新最后登录时间
    db.users()
        .update_last_login(db_user.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 6. 返回响应
    let expires_at = (Utc::now() + chrono::Duration::days(7)).timestamp();
    let body = Json(ApiResponse::success(LoginResponse {
        access_token: access_token.clone(),
        refresh_token: None,
        user: db_user.to_public_user(),
        expires_at,
    }));

    let mut headers = HeaderMap::new();
    append_cookie(&mut headers, build_access_cookie(&access_token))?;
    append_cookie(&mut headers, build_refresh_cookie(&refresh_token))?;

    Ok((StatusCode::OK, headers, body).into_response())
}

/// 用户登录
///
/// POST /api/v1/auth/login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功", body = inline(vespera_common::Response<vespera_common::LoginResponse>)),
        (status = 401, description = "用户名或密码错误"),
        (status = 403, description = "账号已被禁用")
    ),
    tag = "认证"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, ServerError> {
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
    let jwt_secret = jwt_secret_from_env()
        .map_err(|e| ServerError::Internal(format!("JWT configuration failed: {}", e)))?;

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

    let body = Json(ApiResponse::success(LoginResponse {
        access_token: access_token.clone(),
        refresh_token: None,
        user: db_user.to_public_user(),
        expires_at,
    }));

    let mut headers = HeaderMap::new();
    append_cookie(&mut headers, build_access_cookie(&access_token))?;
    append_cookie(&mut headers, build_refresh_cookie(&refresh_token))?;

    Ok((StatusCode::OK, headers, body).into_response())
}

/// 刷新 Access Token
///
/// POST /api/v1/auth/refresh
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "��新成功", body = inline(vespera_common::Response<vespera_common::RefreshTokenResponse>)),
        (status = 401, description = "Refresh Token 无效或已过期")
    ),
    tag = "认证"
)]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Response, ServerError> {
    let db = &state.db;
    let refresh_token = resolve_refresh_token(&headers, &req)?;
    // 1. 验证 Refresh Token
    let refresh_token_record = db
        .users()
        .verify_refresh_token(&refresh_token)
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
    let jwt_secret = jwt_secret_from_env()
        .map_err(|e| ServerError::Internal(format!("JWT configuration failed: {}", e)))?;

    let access_token = create_jwt(
        db_user.id,
        &db_user.role,
        Some(db_user.username.clone()),
        &jwt_secret,
        7,
    )
    .map_err(|e| ServerError::Internal(format!("JWT creation failed: {}", e)))?;

    // 5. 进行 refresh token rotation
    let new_refresh_token = generate_refresh_token();
    db.users()
        .create_refresh_token(db_user.id, &new_refresh_token, 30, None)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    db.users()
        .delete_refresh_token_by_id(refresh_token_record.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 6. 更新新 Refresh Token 最后使用时间
    db.users()
        .update_refresh_token_last_used_for_user(db_user.id, &new_refresh_token)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // 7. 返回响应
    let expires_at = (Utc::now() + chrono::Duration::days(7)).timestamp();

    let body = Json(ApiResponse::success(RefreshTokenResponse {
        access_token: access_token.clone(),
        refresh_token: None,
        expires_at,
    }));

    let mut response_headers = HeaderMap::new();
    append_cookie(&mut response_headers, build_access_cookie(&access_token))?;
    append_cookie(
        &mut response_headers,
        build_refresh_cookie(&new_refresh_token),
    )?;

    Ok((StatusCode::OK, response_headers, body).into_response())
}

/// 登出
///
/// POST /api/v1/auth/logout
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "登出成功")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "认证"
)]
pub async fn logout(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Response, ServerError> {
    let db = &state.db;
    let refresh_token = resolve_refresh_token(&headers, &req)?;
    let refresh_token_record = db
        .users()
        .verify_refresh_token(&refresh_token)
        .await
        .map_err(|_| ServerError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    if refresh_token_record.user_id != auth.id {
        return Err(ServerError::Forbidden(
            "Cannot revoke another user's refresh token".to_string(),
        ));
    }

    // 删除 Refresh Token
    db.users()
        .delete_refresh_token_by_id(refresh_token_record.id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    tracing::info!("User {} logged out", auth.username);

    let mut response_headers = HeaderMap::new();
    append_cookie(&mut response_headers, clear_access_cookie())?;
    append_cookie(&mut response_headers, clear_refresh_cookie())?;

    Ok((
        StatusCode::OK,
        response_headers,
        Json(ApiResponse::success(())),
    )
        .into_response())
}

/// 获取当前用户信息
///
/// GET /api/v1/auth/me
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "获取成功", body = inline(vespera_common::Response<vespera_common::User>)),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "认证"
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "修改成功"),
        (status = 401, description = "旧密码错误")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "认证"
)]
pub async fn change_password(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Response, ServerError> {
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

    let mut response_headers = HeaderMap::new();
    append_cookie(&mut response_headers, clear_access_cookie())?;
    append_cookie(&mut response_headers, clear_refresh_cookie())?;

    Ok((
        StatusCode::OK,
        response_headers,
        Json(ApiResponse::success(())),
    )
        .into_response())
}
