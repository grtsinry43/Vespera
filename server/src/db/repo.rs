use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use super::{
    alert_repo::{AlertRepository, SqliteAlertRepo},
    error::{DbError, DbResult},
    models::*,
    service_repo::{ServiceRepository, SqliteServiceRepo},
    user_repo::UserRepository,
};

/// SQLite 数据库仓库
///
/// 轻量级嵌入式数据库，零依赖部署
pub struct SqliteRepo {
    pool: Pool<Sqlite>,
}

impl SqliteRepo {
    /// 创建新的 SQLite 仓库实例
    ///
    /// # 参数
    /// - `url`: SQLite 连接字符串 (例如: sqlite:monitor.db?mode=rwc)
    ///
    /// # 性能优化
    /// - WAL 模式：支持并发读写
    /// - NORMAL 同步：平衡性能和安全性
    /// - 大缓存：提升查询性能
    pub async fn new(url: &str) -> DbResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;

        // 性能优化配置
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA cache_size = 10000")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&pool)
            .await?;

        // 运行数据库迁移
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DbError::MigrationFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    // ========== User Repository Accessor ==========

    /// 获取用户 Repository
    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    // ========== Alert Repository Accessor ==========

    /// 获取告警 Repository
    pub fn alerts(&self) -> SqliteAlertRepo {
        SqliteAlertRepo::new(self.pool.clone())
    }

    // ========== Service Repository Accessor ==========

    /// 获取服务监控 Repository
    pub fn services(&self) -> SqliteServiceRepo {
        SqliteServiceRepo::new(self.pool.clone())
    }

    // ========== Node 操作 ==========

    /// 根据 UUID 查询节点
    pub async fn get_node_by_uuid(&self, uuid: &str) -> DbResult<Option<Node>> {
        let node = sqlx::query_as!(
            Node,
            r#"
            SELECT id as "id!: i64",
                   uuid, name, ip_address, agent_version, os_type,
                   os_version,
                   cpu_cores as "cpu_cores!: i64",
                   total_memory as "total_memory!: i64",
                   status,
                   last_seen as "last_seen!: i64",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64",
                   is_public as "is_public!: bool",
                   tags
            FROM nodes
            WHERE uuid = ?
            "#,
            uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(node)
    }

    /// 根据 ID 查询节点
    pub async fn get_node_by_id(&self, id: i64) -> DbResult<Option<Node>> {
        let node = sqlx::query_as!(
            Node,
            r#"
            SELECT id as "id!: i64",
                   uuid, name, ip_address, agent_version, os_type,
                   os_version,
                   cpu_cores as "cpu_cores!: i64",
                   total_memory as "total_memory!: i64",
                   status,
                   last_seen as "last_seen!: i64",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64",
                   is_public as "is_public!: bool",
                   tags
            FROM nodes
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(node)
    }

    /// 创建新节点
    pub async fn create_node(&self, node: &NodeCreate) -> DbResult<Node> {
        let now = chrono::Utc::now().timestamp();
        let tags_json = node.tags_json()?;

        let result = sqlx::query!(
            r#"
            INSERT INTO nodes (
                uuid, name, ip_address, agent_version, os_type, os_version,
                cpu_cores, total_memory, status, last_seen, created_at, updated_at,
                is_public, tags
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'online', ?, ?, ?, ?, ?)
            "#,
            node.uuid,
            node.name,
            node.ip_address,
            node.agent_version,
            node.os_type,
            node.os_version,
            node.cpu_cores,
            node.total_memory,
            now,
            now,
            now,
            node.is_public,
            tags_json
        )
        .execute(&self.pool)
        .await?;

        // 获取刚插入的节点
        let created_node = self
            .get_node_by_id(result.last_insert_rowid())
            .await?
            .ok_or(DbError::NotFound)?;

        Ok(created_node)
    }

    /// 更新节点状态和最后在线时间
    pub async fn update_node_status(&self, id: i64, status: &str, last_seen: i64) -> DbResult<()> {
        sqlx::query!(
            r#"
            UPDATE nodes
            SET status = ?, last_seen = ?, updated_at = ?
            WHERE id = ?
            "#,
            status,
            last_seen,
            last_seen,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 列出所有节点(分页)
    pub async fn list_nodes(&self, limit: i64, offset: i64) -> DbResult<Vec<Node>> {
        let nodes = sqlx::query_as!(
            Node,
            r#"
            SELECT id as "id!: i64",
                   uuid, name, ip_address, agent_version, os_type,
                   os_version,
                   cpu_cores as "cpu_cores!: i64",
                   total_memory as "total_memory!: i64",
                   status,
                   last_seen as "last_seen!: i64",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64",
                   is_public as "is_public!: bool",
                   tags
            FROM nodes
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    /// 列出公开节点(分页)
    pub async fn list_public_nodes(&self, limit: i64, offset: i64) -> DbResult<Vec<Node>> {
        let nodes = sqlx::query_as!(
            Node,
            r#"
            SELECT id as "id!: i64",
                   uuid, name, ip_address, agent_version, os_type,
                   os_version,
                   cpu_cores as "cpu_cores!: i64",
                   total_memory as "total_memory!: i64",
                   status,
                   last_seen as "last_seen!: i64",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64",
                   is_public as "is_public!: bool",
                   tags
            FROM nodes
            WHERE is_public = 1
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    /// 更新节点信息（名称、标签、公开性）
    pub async fn update_node(
        &self,
        id: i64,
        name: Option<&str>,
        tags: Option<&str>,
        is_public: Option<bool>,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().timestamp();

        if let Some(node_name) = name {
            sqlx::query!(
                r#"
                UPDATE nodes
                SET name = ?, updated_at = ?
                WHERE id = ?
                "#,
                node_name,
                now,
                id
            )
            .execute(&self.pool)
            .await?;
        }

        if let Some(tags_json) = tags {
            sqlx::query!(
                r#"
                UPDATE nodes
                SET tags = ?, updated_at = ?
                WHERE id = ?
                "#,
                tags_json,
                now,
                id
            )
            .execute(&self.pool)
            .await?;
        }

        if let Some(is_public_value) = is_public {
            sqlx::query!(
                r#"
                UPDATE nodes
                SET is_public = ?, updated_at = ?
                WHERE id = ?
                "#,
                is_public_value,
                now,
                id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// 删除节点（同时删除关联的指标数据）
    pub async fn delete_node(&self, id: i64) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;

        // 1. 删除关联的指标数据
        sqlx::query!(
            r#"
            DELETE FROM metrics WHERE node_id = ?
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // 2. 删除节点
        sqlx::query!(
            r#"
            DELETE FROM nodes WHERE id = ?
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 查询所有超时的在线节点
    ///
    /// 返回 last_seen 早于 before_timestamp 且状态为 online 的节点
    pub async fn get_stale_nodes(&self, before_timestamp: i64) -> DbResult<Vec<Node>> {
        let nodes = sqlx::query_as!(
            Node,
            r#"
            SELECT id as "id!: i64",
                   uuid, name, ip_address, agent_version, os_type,
                   os_version,
                   cpu_cores as "cpu_cores!: i64",
                   total_memory as "total_memory!: i64",
                   status,
                   last_seen as "last_seen!: i64",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64",
                   is_public as "is_public!: bool",
                   tags
            FROM nodes
            WHERE status = 'online' AND last_seen < ?
            "#,
            before_timestamp
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    // ========== Metrics 操作 ==========

    /// 插入单条监控指标
    pub async fn insert_metrics(&self, metrics: &Metric) -> DbResult<i64> {
        let disk_json = metrics.disk_info_json()?;

        let result = sqlx::query!(
            r#"
            INSERT INTO metrics (node_id, timestamp, cpu_usage, cpu_cores, memory_used,
                                memory_total, memory_usage, disk_info, net_in_bytes,
                                net_out_bytes, load_1, load_5, load_15)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            metrics.node_id,
            metrics.timestamp,
            metrics.cpu_usage,
            metrics.cpu_cores,
            metrics.memory_used,
            metrics.memory_total,
            metrics.memory_usage,
            disk_json,
            metrics.net_in_bytes,
            metrics.net_out_bytes,
            metrics.load_1,
            metrics.load_5,
            metrics.load_15
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 批量插入监控指标（使用事务）
    pub async fn insert_metrics_batch(&self, metrics: &[Metric]) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;

        for metric in metrics {
            let disk_json = metric.disk_info_json()?;

            sqlx::query!(
                r#"
                INSERT INTO metrics (node_id, timestamp, cpu_usage, cpu_cores, memory_used,
                                    memory_total, memory_usage, disk_info, net_in_bytes,
                                    net_out_bytes, load_1, load_5, load_15)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                metric.node_id,
                metric.timestamp,
                metric.cpu_usage,
                metric.cpu_cores,
                metric.memory_used,
                metric.memory_total,
                metric.memory_usage,
                disk_json,
                metric.net_in_bytes,
                metric.net_out_bytes,
                metric.load_1,
                metric.load_5,
                metric.load_15
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 获取节点最新的 N 条指标
    pub async fn get_latest_metrics(&self, node_id: i64, limit: i64) -> DbResult<Vec<Metric>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, node_id, timestamp, cpu_usage, cpu_cores, memory_used, memory_total,
                   memory_usage, disk_info, net_in_bytes, net_out_bytes, load_1, load_5, load_15
            FROM metrics
            WHERE node_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            node_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(Metric {
                id: row.id,
                node_id: row.node_id,
                timestamp: row.timestamp,
                cpu_usage: row.cpu_usage,
                cpu_cores: row.cpu_cores,
                memory_used: row.memory_used,
                memory_total: row.memory_total,
                memory_usage: row.memory_usage,
                disk_info: Metric::from_disk_info_json(&row.disk_info)?,
                net_in_bytes: row.net_in_bytes,
                net_out_bytes: row.net_out_bytes,
                load_1: row.load_1,
                load_5: row.load_5,
                load_15: row.load_15,
            });
        }

        Ok(metrics)
    }

    /// 获取指定时间范围的指标
    pub async fn get_metrics_range(
        &self,
        node_id: i64,
        start_time: i64,
        end_time: i64,
        limit: i64,
    ) -> DbResult<Vec<Metric>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, node_id, timestamp, cpu_usage, cpu_cores, memory_used, memory_total,
                   memory_usage, disk_info, net_in_bytes, net_out_bytes, load_1, load_5, load_15
            FROM metrics
            WHERE node_id = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            node_id,
            start_time,
            end_time,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(Metric {
                id: row.id,
                node_id: row.node_id,
                timestamp: row.timestamp,
                cpu_usage: row.cpu_usage,
                cpu_cores: row.cpu_cores,
                memory_used: row.memory_used,
                memory_total: row.memory_total,
                memory_usage: row.memory_usage,
                disk_info: Metric::from_disk_info_json(&row.disk_info)?,
                net_in_bytes: row.net_in_bytes,
                net_out_bytes: row.net_out_bytes,
                load_1: row.load_1,
                load_5: row.load_5,
                load_15: row.load_15,
            });
        }

        Ok(metrics)
    }

    /// 清理旧指标（返回删除的行数）
    pub async fn cleanup_old_metrics(&self, before_timestamp: i64) -> DbResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM metrics WHERE timestamp < ?
            "#,
            before_timestamp
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // ========== AlertRule 操作 ==========
}
