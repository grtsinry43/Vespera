use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::server::AppError;

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
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
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
