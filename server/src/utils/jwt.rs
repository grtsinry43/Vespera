//! JWT (JSON Web Token) 工具模块
//!
//! 用于创建和验证 JWT access tokens。
//!
//! # 安全性
//! - 使用 HS256 (HMAC-SHA256) 算法
//! - 包含过期时间 (exp claim)
//! - 包含用户 ID 和角色信息

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID (subject)
    pub sub: String,

    /// 用户角色 ('admin' | 'user')
    pub role: String,

    /// 用户名 (可选,用于显示)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// 过期时间 (UNIX timestamp)
    pub exp: i64,

    /// 签发时间 (UNIX timestamp)
    pub iat: i64,
}

/// JWT 错误
#[derive(thiserror::Error, Debug)]
pub enum JwtError {
    #[error("Failed to create JWT")]
    CreateError,

    #[error("JWT_SECRET environment variable is required")]
    MissingSecret,

    #[error("Invalid JWT")]
    InvalidToken,

    #[error("JWT expired")]
    TokenExpired,

    #[error("Invalid JWT signature")]
    InvalidSignature,
}

/// 从环境变量加载 JWT 密钥
pub fn jwt_secret_from_env() -> Result<String, JwtError> {
    std::env::var("JWT_SECRET").map_err(|_| JwtError::MissingSecret)
}

/// 创建 JWT
///
/// # 参数
/// - `user_id`: 用户 ID
/// - `role`: 用户角色 ("admin" | "user")
/// - `username`: 用户名 (可选)
/// - `secret`: JWT 签名密钥
/// - `expires_in_days`: 过期天数 (默认 7 天)
///
/// # 返回
/// - `Ok(String)`: JWT 字符串
/// - `Err(JwtError)`: 创建失败
pub fn create_jwt(
    user_id: i64,
    role: &str,
    username: Option<String>,
    secret: &str,
    expires_in_days: i64,
) -> Result<String, JwtError> {
    let now = Utc::now();
    let exp = (now + Duration::days(expires_in_days)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        username,
        exp,
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| JwtError::CreateError)
}

/// 验证 JWT
///
/// # 参数
/// - `token`: JWT 字符串
/// - `secret`: JWT 签名密钥
///
/// # 返回
/// - `Ok(Claims)`: 验证成功,返回 claims
/// - `Err(JwtError)`: 验证失败
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        use jsonwebtoken::errors::ErrorKind;
        match e.kind() {
            ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            _ => JwtError::InvalidToken,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-at-least-32-characters-long";

    #[test]
    fn test_create_jwt() {
        let token = create_jwt(1, "admin", Some("admin".to_string()), TEST_SECRET, 7)
            .expect("Failed to create JWT");

        // JWT 应该有三个部分,用 . 分隔
        assert_eq!(token.matches('.').count(), 2);
    }

    #[test]
    fn test_verify_jwt_success() {
        let token = create_jwt(42, "user", Some("testuser".to_string()), TEST_SECRET, 7).unwrap();

        let claims = verify_jwt(&token, TEST_SECRET).expect("Failed to verify JWT");

        assert_eq!(claims.sub, "42");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.username.unwrap(), "testuser");
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn test_verify_jwt_invalid_secret() {
        let token = create_jwt(1, "admin", None, TEST_SECRET, 7).unwrap();

        let result = verify_jwt(&token, "wrong-secret");
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtError::InvalidSignature)));
    }

    #[test]
    fn test_verify_jwt_invalid_token() {
        let result = verify_jwt("invalid.token.format", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_expired() {
        // 创建一个已过期的 token (过期时间为负数)
        let now = Utc::now();
        let claims = Claims {
            sub: "1".to_string(),
            role: "user".to_string(),
            username: None,
            exp: (now - Duration::hours(1)).timestamp(), // 1 小时前过期
            iat: (now - Duration::hours(2)).timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap();

        let result = verify_jwt(&token, TEST_SECRET);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }

    #[test]
    fn test_jwt_contains_role() {
        let admin_token = create_jwt(1, "admin", None, TEST_SECRET, 7).unwrap();
        let user_token = create_jwt(2, "user", None, TEST_SECRET, 7).unwrap();

        let admin_claims = verify_jwt(&admin_token, TEST_SECRET).unwrap();
        let user_claims = verify_jwt(&user_token, TEST_SECRET).unwrap();

        assert_eq!(admin_claims.role, "admin");
        assert_eq!(user_claims.role, "user");
    }
}
