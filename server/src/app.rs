use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::db::DbRepo;
use crate::middleware::verify_agent_token;
use crate::openapi::ApiDoc;
use crate::routes;
use crate::state::AppState;

/// 创建 Axum 应用
///
/// 组装所有路由,配置中间件,返回可运行的 Router
pub fn create_app(db: DbRepo) -> Router {
    // 创建共享状态
    let state = Arc::new(AppState::new(db));

    // 启动节点健康检查后台任务
    crate::health_check::spawn_health_check_task(state.clone());

    // 启动数据清理后台任务
    crate::cleanup::spawn_cleanup_task(state.clone());

    // 配置 CORS(允许所有来源,生产环境需要限制)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Agent 上报端点 (需要鉴权)
    let report_route = Router::new()
        .route("/report", post(routes::api::v1::report::report_handler))
        .layer(middleware::from_fn(verify_agent_token));

    // 认证路由 (无需鉴权)
    let auth_routes = Router::new()
        .route("/auth/register", post(routes::api::v1::auth::register))
        .route("/auth/login", post(routes::api::v1::auth::login))
        .route("/auth/refresh", post(routes::api::v1::auth::refresh))
        .route("/auth/logout", post(routes::api::v1::auth::logout))
        .route("/auth/me", get(routes::api::v1::auth::me))
        .route(
            "/auth/change-password",
            post(routes::api::v1::auth::change_password),
        );

    // 用户管理路由 (需要管理员权限)
    let user_routes = Router::new()
        .route(
            "/users",
            get(routes::api::v1::users::list_users).post(routes::api::v1::users::create_user),
        )
        .route(
            "/users/{id}",
            get(routes::api::v1::users::get_user)
                .put(routes::api::v1::users::update_user)
                .delete(routes::api::v1::users::delete_user),
        )
        .route(
            "/users/{id}/reset-password",
            post(routes::api::v1::users::reset_password),
        );

    // 告警管理路由 (需要认证)
    let alert_routes = Router::new()
        .route(
            "/alerts/rules",
            get(routes::api::v1::alerts::list_rules).post(routes::api::v1::alerts::create_rule),
        )
        .route(
            "/alerts/rules/{id}",
            get(routes::api::v1::alerts::get_rule)
                .put(routes::api::v1::alerts::update_rule)
                .delete(routes::api::v1::alerts::delete_rule),
        )
        .route("/alerts", get(routes::api::v1::alerts::list_alerts))
        .route(
            "/alerts/node/{node_id}",
            get(routes::api::v1::alerts::list_node_alerts),
        )
        .route(
            "/alerts/{id}/resolve",
            post(routes::api::v1::alerts::resolve_alert),
        );

    // 节点管理路由 (普通用户可见)
    let node_routes = Router::new()
        .route(
            "/nodes",
            get(routes::api::v1::nodes::list_nodes),
        )
        .route(
            "/nodes/{id}",
            get(routes::api::v1::nodes::get_node),
        )
        .route(
            "/nodes/{id}/metrics",
            get(routes::api::v1::nodes::get_node_metrics),
        );

    // 管理员节点管理路由
    let admin_node_routes = Router::new()
        .route(
            "/admin/nodes",
            get(routes::api::v1::nodes::admin_list_nodes),
        )
        .route(
            "/admin/nodes/{id}",
            get(routes::api::v1::nodes::admin_get_node)
                .put(routes::api::v1::nodes::admin_update_node)
                .delete(routes::api::v1::nodes::admin_delete_node),
        )
        .route(
            "/admin/nodes/{id}/visibility",
            put(routes::api::v1::nodes::admin_update_node_visibility),
        );

    // 服务监控路由
    let service_routes = Router::new()
        .route(
            "/services",
            get(routes::api::v1::services::list_services)
                .post(routes::api::v1::services::create_service),
        )
        .route(
            "/services/all/overview",
            get(routes::api::v1::services::get_all_services_overview),
        )
        .route(
            "/services/{id}",
            get(routes::api::v1::services::get_service)
                .put(routes::api::v1::services::update_service)
                .delete(routes::api::v1::services::delete_service),
        )
        .route(
            "/services/{id}/visibility",
            put(routes::api::v1::services::update_service_visibility),
        )
        .route(
            "/services/{id}/status",
            get(routes::api::v1::services::get_service_status),
        )
        .route(
            "/services/{id}/overview",
            get(routes::api::v1::services::get_service_overview),
        );

    // Agent 服务监控路由 (需要鉴权)
    let agent_service_routes = Router::new()
        .route(
            "/agent/services",
            get(routes::api::v1::services::agent_get_services),
        )
        .route(
            "/agent/service-status",
            post(routes::api::v1::services::agent_report_status),
        )
        .layer(middleware::from_fn(verify_agent_token));

    // API v1 路由
    let api_v1_routes = Router::new()
        .merge(report_route)
        .merge(auth_routes)
        .merge(user_routes)
        .merge(alert_routes)
        .merge(node_routes)
        .merge(admin_node_routes)
        .merge(service_routes)
        .merge(agent_service_routes)
        .route("/ws", get(crate::ws::ws_handler)); // WebSocket 端点

    // API 路由
    let api_routes = Router::new().nest("/v1", api_v1_routes);

    // 创建 OpenAPI 实例
    let openapi = ApiDoc::openapi();

    // 主路由
    Router::new()
        .route("/health", get(routes::health::health_check))
        .nest("/api", api_routes)
        .merge(Scalar::with_url("/scalar", openapi.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
