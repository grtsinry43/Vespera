use axum::{extract::Request, http::header::AUTHORIZATION, middleware::Next, response::Response};

use vespera_common::ServerError as AppError;

/// Agent 注册令牌验证中间件
///
/// 验证所有上报请求必须携带正确的 Authorization Header
///
/// # 环境变量
/// - `AGENT_REGISTRATION_TOKEN`: Agent 注册令牌（必需）
///
/// # Header 格式
/// ```
/// Authorization: Bearer <token>
/// ```
///
/// # 安全性
/// - 令牌必须是高熵随机字符串（建议 32+ 字符）
/// - 通过环境变量配置，不要硬编码
/// - 令牌泄露时立即更换
pub async fn verify_agent_token(req: Request, next: Next) -> Result<Response, AppError> {
    // 从环境变量读取令牌
    let expected_token = std::env::var("AGENT_REGISTRATION_TOKEN").map_err(|_| {
        tracing::error!("AGENT_REGISTRATION_TOKEN not set in environment");
        AppError::Internal("Server configuration error".to_string())
    })?;

    // 获取 Authorization Header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Missing Authorization header");
            AppError::Unauthorized("Missing Authorization header".to_string())
        })?;

    // 验证格式: "Bearer <token>"
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        tracing::warn!("Invalid Authorization header format: {}", auth_header);
        AppError::Unauthorized("Invalid Authorization header format".to_string())
    })?;

    // 验证令牌是否匹配
    if token != expected_token {
        tracing::warn!(
            "Invalid token attempt from IP: {:?}",
            req.headers().get("x-forwarded-for")
        );
        return Err(AppError::Unauthorized("Invalid token".to_string()));
    }

    // 令牌验证通过，放行请求
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_valid_token() {
        std::env::set_var("AGENT_REGISTRATION_TOKEN", "test-secret-token");

        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(verify_agent_token));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(AUTHORIZATION, "Bearer test-secret-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_token() {
        std::env::set_var("AGENT_REGISTRATION_TOKEN", "test-secret-token");

        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(verify_agent_token));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        std::env::set_var("AGENT_REGISTRATION_TOKEN", "test-secret-token");

        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(verify_agent_token));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(AUTHORIZATION, "Bearer wrong-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_format() {
        std::env::set_var("AGENT_REGISTRATION_TOKEN", "test-secret-token");

        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(verify_agent_token));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(AUTHORIZATION, "test-secret-token") // Missing "Bearer "
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

// ============================================
// 用户认证中间件 (JWT)
// ============================================

use axum::{extract::FromRequestParts, http::request::Parts};
use vespera_common::UserRole;

/// 认证用户信息 (从 JWT 提取)
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
}

impl AuthUser {
    /// 检查是否为管理员
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 从 Authorization header 提取 Bearer token
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        // 2. 验证格式: "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format".to_string()))?;

        // 3. 获取 JWT secret (从环境变量)
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("JWT_SECRET not set, using default");
            "change-this-secret-key-at-least-32-characters-long".to_string()
        });

        // 4. 验证 JWT
        let claims = crate::utils::verify_jwt(token, &jwt_secret).map_err(|e| {
            tracing::warn!("JWT verification failed: {:?}", e);
            AppError::Unauthorized(format!("Invalid token: {}", e))
        })?;

        // 5. 解析用户 ID
        let id = claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

        // 6. 解析角色
        let role = UserRole::from_str(&claims.role)
            .ok_or_else(|| AppError::Unauthorized("Invalid role in token".to_string()))?;

        // 7. 构造 AuthUser
        Ok(AuthUser {
            id,
            username: claims.username.unwrap_or_else(|| format!("user_{}", id)),
            role,
        })
    }
}

/// 管理员用户 (需要 admin 角色)
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 先验证是否为认证用户
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        // 2. 检查是否为管理员
        if !auth_user.is_admin() {
            tracing::warn!(
                "User {} attempted admin action without permission",
                auth_user.username
            );
            return Err(AppError::Forbidden("Admin permission required".to_string()));
        }

        Ok(AdminUser(auth_user))
    }
}

/// 可选认证用户 (游客模式)
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 尝试提取 AuthUser,失败则返回 None
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
