//! 用户数据模型
//!
//! 定义用户相关的数据结构,在 Agent、Server 和 Frontend 之间共享

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 用户角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            _ => None,
        }
    }
}

/// 用户信息 (对外暴露,不包含敏感信息)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_login_at: Option<i64>,
}

/// 登录请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    /// 是否为管理员 (仅首次注册或管理员创建时有效)
    #[serde(default)]
    pub is_admin: bool,
}

/// 登录响应
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user: User,
    pub expires_at: i64,
}

/// Refresh Token 请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// Refresh Token 响应
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>, // Token rotation (可选)
    pub expires_at: i64,
}

/// 创建用户请求 (管理员)
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: UserRole,
}

/// 更新用户请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<UserRole>,
}

/// 修改密码请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 重置密码请求 (管理员)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_serialization() {
        assert_eq!(
            serde_json::to_string(&UserRole::Admin).unwrap(),
            "\"admin\""
        );
        assert_eq!(serde_json::to_string(&UserRole::User).unwrap(), "\"user\"");
    }

    #[test]
    fn test_user_role_from_str() {
        assert_eq!(UserRole::from_str("admin"), Some(UserRole::Admin));
        assert_eq!(UserRole::from_str("user"), Some(UserRole::User));
        assert_eq!(UserRole::from_str("invalid"), None);
    }
}
