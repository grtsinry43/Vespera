-- 告警系统完整 Schema
-- 创建告警规则表
CREATE TABLE IF NOT EXISTS alert_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    node_id INTEGER,  -- NULL 表示全局规则
    rule_type TEXT NOT NULL,  -- cpu_high, memory_high, disk_full, node_offline, load_high
    severity TEXT NOT NULL,   -- info, warning, critical
    config TEXT NOT NULL,     -- JSON: 规则配置（阈值、持续时间等）
    notification_channels TEXT NOT NULL,  -- JSON: 通知渠道配置
    silence_duration_secs INTEGER NOT NULL DEFAULT 300,  -- 静默期（秒）
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_alert_rules_node_id ON alert_rules(node_id);

-- 告警记录表
CREATE TABLE IF NOT EXISTS alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    node_id INTEGER NOT NULL,
    node_name TEXT NOT NULL,
    severity TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    message TEXT NOT NULL,
    triggered_at INTEGER NOT NULL,
    resolved_at INTEGER,
    metadata TEXT,  -- JSON: 告警相关的元数据
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_alerts_node_id ON alerts(node_id);
CREATE INDEX IF NOT EXISTS idx_alerts_triggered_at ON alerts(triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alerts_resolved_at ON alerts(resolved_at);

-- 通知设置表（用于 Email 等全局通知配置）
CREATE TABLE IF NOT EXISTS notification_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    smtp_server TEXT,
    smtp_username TEXT,
    smtp_password TEXT,
    smtp_from_address TEXT,
    smtp_use_tls INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 插入默认告警规则模板（可选）
INSERT INTO alert_rules (name, node_id, rule_type, severity, config, notification_channels, silence_duration_secs, enabled, created_at, updated_at)
VALUES
    -- 全局 CPU 高负载规则
    ('全局 CPU 高负载', NULL, 'cpu_high', 'warning',
     '{"type":"CpuHigh","threshold_percent":80.0,"duration_secs":60}',
     '[{"type":"WebSocket"}]',
     300, 1, strftime('%s', 'now'), strftime('%s', 'now')),

    -- 全局内存高使用率规则
    ('全局内存高使用率', NULL, 'memory_high', 'warning',
     '{"type":"MemoryHigh","threshold_percent":85.0,"duration_secs":60}',
     '[{"type":"WebSocket"}]',
     300, 1, strftime('%s', 'now'), strftime('%s', 'now')),

    -- 全局磁盘空间不足规则
    ('全局磁盘空间不足', NULL, 'disk_full', 'critical',
     '{"type":"DiskFull","threshold_percent":90.0,"mount_point":null}',
     '[{"type":"WebSocket"}]',
     600, 1, strftime('%s', 'now'), strftime('%s', 'now')),

    -- 全局节点离线规则
    ('全局节点离线', NULL, 'node_offline', 'critical',
     '{"type":"NodeOffline","timeout_secs":300}',
     '[{"type":"WebSocket"}]',
     0, 1, strftime('%s', 'now'), strftime('%s', 'now')),

    -- 全局负载过高规则
    ('全局负载过高', NULL, 'load_high', 'warning',
     '{"type":"LoadHigh","threshold":5.0,"duration_secs":120}',
     '[{"type":"WebSocket"}]',
     300, 1, strftime('%s', 'now'), strftime('%s', 'now')),

    -- 严重 CPU 负载规则
    ('严重 CPU 负载', NULL, 'cpu_high', 'critical',
     '{"type":"CpuHigh","threshold_percent":95.0,"duration_secs":30}',
     '[{"type":"WebSocket"}]',
     600, 1, strftime('%s', 'now'), strftime('%s', 'now'));
