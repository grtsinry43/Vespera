-- ============================================
-- 迁移: 为节点和服务添加 is_public 字段
-- 版本: 20250201000001
-- 描述: 节点/服务支持公开可见控制
-- ============================================

-- ============ 节点 is_public 字段 ============
ALTER TABLE nodes ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT 0;

-- 为常见查询创建索引
CREATE INDEX IF NOT EXISTS idx_nodes_is_public ON nodes(is_public);

-- ============ 服务 is_public 字段 ============
ALTER TABLE services ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_services_is_public ON services(is_public);

-- ============================================
-- 迁移完成
-- ============================================
