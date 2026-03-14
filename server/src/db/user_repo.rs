//! 用户数据库操作模块

use crate::db::models::{DbRefreshToken, DbUser};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};

/// 用户数据库操作错误
#[derive(thiserror::Error, Debug)]
pub enum UserRepoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("User not found")]
    UserNotFound,

    #[error("Username already exists")]
    UsernameExists,

    #[error("Email already exists")]
    EmailExists,
}

/// 用户 Repository
pub struct UserRepository {
    pool: Pool<Sqlite>,
}

impl UserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// 统计用户总数
    pub async fn count_users(&self) -> Result<i64, UserRepoError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    /// 通过用户名查找用户
    pub async fn find_by_username(&self, username: &str) -> Result<DbUser, UserRepoError> {
        sqlx::query_as!(
            DbUser,
            r#"
            SELECT
                id as "id!",
                username as "username!",
                email,
                password_hash,
                role as "role!",
                avatar_url,
                is_active as "is_active!: bool",
                created_at as "created_at!",
                updated_at as "updated_at!",
                last_login_at
            FROM users
            WHERE username = ?
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserRepoError::UserNotFound,
            _ => UserRepoError::DatabaseError(e),
        })
    }

    /// 通过 ID 查找用户
    pub async fn find_by_id(&self, id: i64) -> Result<DbUser, UserRepoError> {
        sqlx::query_as!(
            DbUser,
            r#"
            SELECT
                id as "id!",
                username as "username!",
                email,
                password_hash,
                role as "role!",
                avatar_url,
                is_active as "is_active!: bool",
                created_at as "created_at!",
                updated_at as "updated_at!",
                last_login_at
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserRepoError::UserNotFound,
            _ => UserRepoError::DatabaseError(e),
        })
    }

    /// 创建用户
    pub async fn create_user(
        &self,
        username: &str,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: &str,
    ) -> Result<DbUser, UserRepoError> {
        let now = Utc::now().timestamp();

        let result = sqlx::query!(
            r#"
            INSERT INTO users (username, email, password_hash, role, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            username,
            email,
            password_hash,
            role,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // 检查唯一性约束错误
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("username") {
                    return UserRepoError::UsernameExists;
                }
                if db_err.message().contains("email") {
                    return UserRepoError::EmailExists;
                }
            }
            UserRepoError::DatabaseError(e)
        })?;

        self.find_by_id(result.last_insert_rowid()).await
    }

    /// 更新最后登录时间
    pub async fn update_last_login(&self, user_id: i64) -> Result<(), UserRepoError> {
        let now = Utc::now().timestamp();

        sqlx::query!(
            r#"
            UPDATE users
            SET last_login_at = ?
            WHERE id = ?
            "#,
            now,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新用户信息
    pub async fn update_user(
        &self,
        user_id: i64,
        email: Option<&str>,
        avatar_url: Option<&str>,
        is_active: Option<bool>,
        role: Option<&str>,
    ) -> Result<DbUser, UserRepoError> {
        let now = Utc::now().timestamp();

        // 先获取当前用户数据
        let current_user = self.find_by_id(user_id).await?;

        // 使用传入值或保留原值
        let email = email.or(current_user.email.as_deref());
        let avatar_url = avatar_url.or(current_user.avatar_url.as_deref());
        let is_active = is_active.unwrap_or(current_user.is_active);
        let role = role.unwrap_or(&current_user.role);

        sqlx::query!(
            r#"
            UPDATE users
            SET email = ?, avatar_url = ?, is_active = ?, role = ?, updated_at = ?
            WHERE id = ?
            "#,
            email,
            avatar_url,
            is_active,
            role,
            now,
            user_id
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(user_id).await
    }

    /// 更新密码
    pub async fn update_password(
        &self,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<(), UserRepoError> {
        let now = Utc::now().timestamp();

        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = ?, updated_at = ?
            WHERE id = ?
            "#,
            new_password_hash,
            now,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除用户
    pub async fn delete_user(&self, user_id: i64) -> Result<(), UserRepoError> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 列出所有用户
    pub async fn list_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DbUser>, UserRepoError> {
        sqlx::query_as!(
            DbUser,
            r#"
            SELECT
                id as "id!",
                username as "username!",
                email,
                password_hash,
                role as "role!",
                avatar_url,
                is_active as "is_active!: bool",
                created_at as "created_at!",
                updated_at as "updated_at!",
                last_login_at
            FROM users
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(UserRepoError::DatabaseError)
    }

    // ============================================
    // Refresh Token 操作
    // ============================================

    /// 创建 Refresh Token
    pub async fn create_refresh_token(
        &self,
        user_id: i64,
        token: &str,
        expires_in_days: i64,
        device_info: Option<&str>,
    ) -> Result<DbRefreshToken, UserRepoError> {
        let now = Utc::now();
        let created_at = now.timestamp();
        let expires_at = (now + chrono::Duration::days(expires_in_days)).timestamp();

        // 计算 token 的 SHA-256 哈希
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        let result = sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at, created_at, device_info)
            VALUES (?, ?, ?, ?, ?)
            "#,
            user_id,
            token_hash,
            expires_at,
            created_at,
            device_info
        )
        .execute(&self.pool)
        .await?;

        Ok(DbRefreshToken {
            id: result.last_insert_rowid(),
            user_id,
            token_hash,
            expires_at,
            created_at,
            last_used_at: None,
            device_info: device_info.map(String::from),
        })
    }

    /// 验证 Refresh Token
    pub async fn verify_refresh_token(
        &self,
        token: &str,
    ) -> Result<DbRefreshToken, UserRepoError> {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let now = Utc::now().timestamp();

        sqlx::query_as!(
            DbRefreshToken,
            r#"
            SELECT
                id as "id!",
                user_id as "user_id!",
                token_hash as "token_hash!",
                expires_at as "expires_at!",
                created_at as "created_at!",
                last_used_at,
                device_info
            FROM refresh_tokens
            WHERE token_hash = ? AND expires_at > ?
            "#,
            token_hash,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserRepoError::UserNotFound,
            _ => UserRepoError::DatabaseError(e),
        })
    }

    /// 更新 Refresh Token 最后使用时间
    pub async fn update_refresh_token_last_used(
        &self,
        token_id: i64,
    ) -> Result<(), UserRepoError> {
        let now = Utc::now().timestamp();

        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET last_used_at = ?
            WHERE id = ?
            "#,
            now,
            token_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据 token 明文更新 Refresh Token 最后使用时间
    pub async fn update_refresh_token_last_used_for_user(
        &self,
        user_id: i64,
        token: &str,
    ) -> Result<(), UserRepoError> {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let now = Utc::now().timestamp();

        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET last_used_at = ?
            WHERE user_id = ? AND token_hash = ?
            "#,
            now,
            user_id,
            token_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除用户的所有 Refresh Token (登出/修改密码时使用)
    pub async fn delete_user_refresh_tokens(&self, user_id: i64) -> Result<(), UserRepoError> {
        sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE user_id = ?
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除特定 Refresh Token
    pub async fn delete_refresh_token(&self, token: &str) -> Result<(), UserRepoError> {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE token_hash = ?
            "#,
            token_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据主键删除 Refresh Token
    pub async fn delete_refresh_token_by_id(&self, token_id: i64) -> Result<(), UserRepoError> {
        sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE id = ?
            "#,
            token_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 清理过期的 Refresh Tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<u64, UserRepoError> {
        let now = Utc::now().timestamp();

        let result = sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at <= ?
            "#,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
