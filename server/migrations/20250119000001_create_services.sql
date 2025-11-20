-- 服务监控配置表
CREATE TABLE IF NOT EXISTS services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER,                       -- 关联的节点（可选，为空表示不关联特定节点）
    name TEXT NOT NULL,                    -- 服务名称
    type TEXT NOT NULL,                    -- 'http' 或 'tcp'
    target TEXT NOT NULL,                  -- URL 或 IP:Port
    check_interval INTEGER NOT NULL DEFAULT 3600,  -- 检查间隔（秒），默认1小时
    timeout INTEGER NOT NULL DEFAULT 10,   -- 超时时间（秒）
    method TEXT DEFAULT 'GET',             -- HTTP 方法
    expected_code INTEGER DEFAULT 200,     -- 期望状态码
    expected_body TEXT,                    -- 期望响应内容（可选）
    headers TEXT,                          -- 自定义请求头（JSON）
    enabled BOOLEAN NOT NULL DEFAULT 1,    -- 是否启用
    created_at INTEGER NOT NULL,           -- 创建时间
    updated_at INTEGER NOT NULL,           -- 更新时间
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 服务状态历史表
CREATE TABLE IF NOT EXISTS service_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL,           -- 关联 services.id
    agent_id INTEGER,                      -- 执行检查的 agent（可选）
    status TEXT NOT NULL,                  -- 'up', 'down', 'timeout', 'error'
    response_time INTEGER,                 -- 响应时间（毫秒）
    status_code INTEGER,                   -- HTTP 状态码
    error_message TEXT,                    -- 错误信息
    checked_at INTEGER NOT NULL,           -- 检查时间
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES nodes(id) ON DELETE SET NULL
);

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_services_node_id ON services(node_id);
CREATE INDEX IF NOT EXISTS idx_services_enabled ON services(enabled);
CREATE INDEX IF NOT EXISTS idx_service_status_service_id ON service_status(service_id);
CREATE INDEX IF NOT EXISTS idx_service_status_checked_at ON service_status(checked_at);
CREATE INDEX IF NOT EXISTS idx_service_status_service_checked ON service_status(service_id, checked_at DESC);
