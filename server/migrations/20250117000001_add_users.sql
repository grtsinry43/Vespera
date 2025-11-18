-- ============================================
-- 用户认证与授权系统数据库迁移
-- 版本: 20250117000001
-- 描述: 添加用户、OAuth 关联、Refresh Token 表
-- ============================================

-- ============ 用户表 ============
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT,              -- 本地登录使用, OAuth 可为 NULL
    role TEXT NOT NULL DEFAULT 'user', -- 'admin' | 'user'
    avatar_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at INTEGER NOT NULL,     -- UNIX timestamp
    updated_at INTEGER NOT NULL,
    last_login_at INTEGER,

    -- 约束
    CHECK (role IN ('admin', 'user')),
    CHECK (length(username) >= 3 AND length(username) <= 32),
    CHECK (email IS NULL OR email LIKE '%@%')
);

-- 用户表索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email) WHERE email IS NOT NULL;
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_is_active ON users(is_active);

-- ============ OAuth 关联表 ============
CREATE TABLE oauth_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    provider TEXT NOT NULL,          -- 'google' | 'github'
    provider_user_id TEXT NOT NULL,  -- OAuth 提供商的 user ID
    access_token TEXT,               -- 加密存储 (可选)
    refresh_token TEXT,              -- 加密存储 (可选)
    expires_at INTEGER,              -- UNIX timestamp
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    -- 约束
    CHECK (provider IN ('google', 'github')),
    UNIQUE(provider, provider_user_id)
);

-- OAuth 关联表索引
CREATE INDEX idx_oauth_accounts_user ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_accounts_provider ON oauth_accounts(provider, provider_user_id);

-- ============ Refresh Token 表 ============
CREATE TABLE refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,  -- SHA-256 哈希
    expires_at INTEGER NOT NULL,      -- UNIX timestamp
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    device_info TEXT,                 -- User-Agent (可选)

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    -- 约束
    CHECK (expires_at > created_at)
);

-- Refresh Token 表索引
CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens(expires_at);

-- ============ 用户-节点权限表 (可选功能) ============
CREATE TABLE user_node_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    node_id INTEGER NOT NULL,
    can_view BOOLEAN NOT NULL DEFAULT TRUE,
    can_manage BOOLEAN NOT NULL DEFAULT FALSE,
    created_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,

    UNIQUE(user_id, node_id)
);

-- 用户-节点权限表索引
CREATE INDEX idx_user_node_permissions_user ON user_node_permissions(user_id);
CREATE INDEX idx_user_node_permissions_node ON user_node_permissions(node_id);

-- ============================================
-- 迁移完成
-- ============================================
