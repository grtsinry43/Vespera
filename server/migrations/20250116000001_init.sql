-- SQLite 初始化脚本
-- 节点表
CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    os_type TEXT NOT NULL,
    os_version TEXT,
    cpu_cores INTEGER NOT NULL,
    total_memory INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'online',
    last_seen INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    tags TEXT
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_nodes_uuid ON nodes(uuid);
CREATE INDEX IF NOT EXISTS idx_nodes_status ON nodes(status);
CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON nodes(last_seen);

-- 指标表（时序数据）
CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,

    -- CPU
    cpu_usage REAL NOT NULL,
    cpu_cores INTEGER NOT NULL,

    -- Memory
    memory_used INTEGER NOT NULL,
    memory_total INTEGER NOT NULL,
    memory_usage REAL NOT NULL,

    -- Disk (JSON)
    disk_info TEXT NOT NULL,

    -- Network
    net_in_bytes INTEGER NOT NULL,
    net_out_bytes INTEGER NOT NULL,

    -- Load Average
    load_1 REAL,
    load_5 REAL,
    load_15 REAL,

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_metrics_node_timestamp ON metrics(node_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);
