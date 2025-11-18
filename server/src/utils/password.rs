//! 密码哈希工具模块
//!
//! 使用 Argon2id 算法进行密码哈希,这是 OWASP 推荐的密码存储方案。
//!
//! # 安全性
//! - 使用 Argon2id 变体(结合数据依赖和数据独立内存访问)
//! - 自动生成随机 salt
//! - PHC 字符串格式存储(包含算法参数)

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// 密码哈希错误
#[derive(thiserror::Error, Debug)]
pub enum PasswordError {
    #[error("Failed to hash password")]
    HashError,

    #[error("Failed to verify password")]
    VerifyError,

    #[error("Invalid password hash format")]
    InvalidHashFormat,
}

/// 对密码进行哈希
///
/// # 参数
/// - `password`: 明文密码
///
/// # 返回
/// - `Ok(String)`: PHC 格式的哈希字符串
/// - `Err(PasswordError)`: 哈希失败
///
/// # 示例
/// ```
/// let hash = hash_password("my-secret-password")?;
/// // 返回类似: $argon2id$v=19$m=19456,t=2,p=1$...
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| PasswordError::HashError)?;

    Ok(password_hash.to_string())
}

/// 验证密码
///
/// # 参数
/// - `password`: 用户输入的明文密码
/// - `hash`: 数据库中存储的哈希
///
/// # 返回
/// - `Ok(true)`: 密码正确
/// - `Ok(false)`: 密码错误
/// - `Err(PasswordError)`: 验证过程出错
///
/// # 示例
/// ```
/// let is_valid = verify_password("user-input", &stored_hash)?;
/// if is_valid {
///     // 密码正确,允许登录
/// }
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| PasswordError::InvalidHashFormat)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "my-secure-password-123";
        let hash = hash_password(password).expect("Failed to hash password");

        // PHC 格式应该以 $argon2id$ 开头
        assert!(hash.starts_with("$argon2id$"));
        // 长度应该合理 (通常 > 80 字符)
        assert!(hash.len() > 80);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "correct-password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).expect("Failed to verify password");
        assert!(result, "Valid password should verify successfully");
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "correct-password";
        let wrong_password = "wrong-password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(wrong_password, &hash).expect("Failed to verify password");
        assert!(!result, "Invalid password should fail verification");
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("any-password", "invalid-hash-format");
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::InvalidHashFormat)));
    }

    #[test]
    fn test_different_salts() {
        let password = "same-password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // 相同密码应该产生不同的哈希(因为 salt 不同)
        assert_ne!(hash1, hash2);

        // 但都应该能验证成功
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
