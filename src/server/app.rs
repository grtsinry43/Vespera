use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::server::routes;
use crate::server::state::AppState;

/// 创建 Axum 应用
///
/// 组装所有路由，配置中间件，返回可运行的 Router
pub fn create_app() -> Router {
    // 创建共享状态
    let state = Arc::new(AppState::new());

    // 配置 CORS（允许所有来源，生产环境需要限制）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API v1 路由
    let api_v1_routes = Router::new()
        .merge(routes::api::v1::nodes::routes())
        .with_state(state.clone());

    // API 路由
    let api_routes = Router::new().nest("/v1", api_v1_routes);

    // 主路由
    Router::new()
        // 健康检查端点
        .route("/health", get(routes::health::health_check))
        // API 路由组
        .nest("/api", api_routes)
        // 全局中间件
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        // 全局状态
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_health_check_route() {
        let app = create_app();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = create_app();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/nonexistent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
