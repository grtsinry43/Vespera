//! OpenAPI 文档定义
//!
//! 使用 utoipa 自动生成 OpenAPI 规范

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

/// Vespera 监控系统 OpenAPI 文档
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Vespera LightMonitor API",
        version = "0.1.0",
        description = "轻量级服务器监控面板系统 API 文档",
        contact(
            name = "Vespera Team",
        )
    ),
    paths(
        // 认证 API
        crate::routes::api::v1::auth::register,
        crate::routes::api::v1::auth::login,
        crate::routes::api::v1::auth::refresh,
        crate::routes::api::v1::auth::logout,
        crate::routes::api::v1::auth::me,
        crate::routes::api::v1::auth::change_password,

        // 用户管理 API
        crate::routes::api::v1::users::list_users,
        crate::routes::api::v1::users::get_user,
        crate::routes::api::v1::users::create_user,
        crate::routes::api::v1::users::update_user,
        crate::routes::api::v1::users::delete_user,
        crate::routes::api::v1::users::reset_password,
    ),
    components(
        schemas(
            // 用户相关类型
            vespera_common::User,
            vespera_common::UserRole,
            vespera_common::LoginRequest,
            vespera_common::LoginResponse,
            vespera_common::RegisterRequest,
            vespera_common::RefreshTokenRequest,
            vespera_common::RefreshTokenResponse,
            vespera_common::ChangePasswordRequest,
            vespera_common::CreateUserRequest,
            vespera_common::UpdateUserRequest,
            vespera_common::ResetPasswordRequest,
        )
    ),
    tags(
        (name = "认证", description = "用户认证相关 API"),
        (name = "用户管理", description = "用户管理相关 API (需要管理员权限)")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// 添加 Security Scheme (JWT Bearer Token)
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
