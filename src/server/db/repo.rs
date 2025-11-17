use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use super::{
    error::{DbError, DbResult},
    models::*,
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
        sqlx::migrate!("./migrations/sqlite")
            .run(&pool)
            .await
            .map_err(|e| DbError::MigrationFailed(e.to_string()))?;

        Ok(Self { pool })
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
            INSERT INTO nodes (uuid, name, ip_address, agent_version, os_type, os_version,
                              cpu_cores, total_memory, status, last_seen, created_at, updated_at, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'online', ?, ?, ?, ?)
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

    /// 删除节点
    pub async fn delete_node(&self, id: i64) -> DbResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM nodes WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
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

    /// 创建告警规则
    pub async fn create_alert_rule(&self, rule: &AlertRuleCreate) -> DbResult<AlertRule> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query!(
            r#"
            INSERT INTO alert_rules (name, node_id, metric_type, condition, threshold, duration,
                                    enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)
            "#,
            rule.name,
            rule.node_id,
            rule.metric_type,
            rule.condition,
            rule.threshold,
            rule.duration,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        // 查询刚创建的规则
        let rule_id = result.last_insert_rowid();
        let created_rule = sqlx::query_as!(
            AlertRule,
            r#"
            SELECT id as "id!: i64",
                   name, node_id, metric_type, condition,
                   threshold as "threshold!: f64",
                   duration as "duration!: i64",
                   enabled as "enabled!: bool",
                   created_at as "created_at!: i64",
                   updated_at as "updated_at!: i64"
            FROM alert_rules
            WHERE id = ?
            "#,
            rule_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_rule)
    }

    /// 列出所有告警规则
    pub async fn list_alert_rules(&self, enabled_only: bool) -> DbResult<Vec<AlertRule>> {
        let rules = if enabled_only {
            sqlx::query_as!(
                AlertRule,
                r#"
                SELECT id as "id!: i64",
                       name, node_id, metric_type, condition,
                       threshold as "threshold!: f64",
                       duration as "duration!: i64",
                       enabled as "enabled!: bool",
                       created_at as "created_at!: i64",
                       updated_at as "updated_at!: i64"
                FROM alert_rules
                WHERE enabled = 1
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                AlertRule,
                r#"
                SELECT id as "id!: i64",
                       name, node_id, metric_type, condition,
                       threshold as "threshold!: f64",
                       duration as "duration!: i64",
                       enabled as "enabled!: bool",
                       created_at as "created_at!: i64",
                       updated_at as "updated_at!: i64"
                FROM alert_rules
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rules)
    }

    /// 更新告警规则启用状态
    pub async fn update_alert_rule_enabled(&self, id: i64, enabled: bool) -> DbResult<()> {
        let now = chrono::Utc::now().timestamp();
        let enabled_int = if enabled { 1 } else { 0 };

        sqlx::query!(
            r#"
            UPDATE alert_rules
            SET enabled = ?, updated_at = ?
            WHERE id = ?
            "#,
            enabled_int,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除告警规则
    pub async fn delete_alert_rule(&self, id: i64) -> DbResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM alert_rules WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========== Alert 操作 ==========

    /// 创建告警记录
    pub async fn create_alert(&self, alert: &AlertCreate) -> DbResult<Alert> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query!(
            r#"
            INSERT INTO alerts (rule_id, node_id, level, message, value, status, triggered_at, resolved_at)
            VALUES (?, ?, ?, ?, ?, 'active', ?, NULL)
            "#,
            alert.rule_id,
            alert.node_id,
            alert.level,
            alert.message,
            alert.value,
            now
        )
        .execute(&self.pool)
        .await?;

        // 查询刚创建的告警
        let alert_id = result.last_insert_rowid();
        let created_alert = sqlx::query_as!(
            Alert,
            r#"
            SELECT id as "id!: i64",
                   rule_id as "rule_id!: i64",
                   node_id as "node_id!: i64",
                   level, message,
                   value as "value!: f64",
                   status,
                   triggered_at as "triggered_at!: i64",
                   resolved_at
            FROM alerts
            WHERE id = ?
            "#,
            alert_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_alert)
    }

    /// 解决告警（标记为已解决）
    pub async fn resolve_alert(&self, id: i64, resolved_at: i64) -> DbResult<()> {
        sqlx::query!(
            r#"
            UPDATE alerts
            SET status = 'resolved', resolved_at = ?
            WHERE id = ?
            "#,
            resolved_at,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 列出活跃的告警
    pub async fn list_active_alerts(&self, node_id: Option<i64>) -> DbResult<Vec<Alert>> {
        let alerts = match node_id {
            Some(nid) => {
                sqlx::query_as!(
                    Alert,
                    r#"
                    SELECT id as "id!: i64",
                           rule_id as "rule_id!: i64",
                           node_id as "node_id!: i64",
                           level, message,
                           value as "value!: f64",
                           status,
                           triggered_at as "triggered_at!: i64",
                           resolved_at
                    FROM alerts
                    WHERE node_id = ? AND status = 'active'
                    ORDER BY triggered_at DESC
                    "#,
                    nid
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Alert,
                    r#"
                    SELECT id as "id!: i64",
                           rule_id as "rule_id!: i64",
                           node_id as "node_id!: i64",
                           level, message,
                           value as "value!: f64",
                           status,
                           triggered_at as "triggered_at!: i64",
                           resolved_at
                    FROM alerts
                    WHERE status = 'active'
                    ORDER BY triggered_at DESC
                    "#
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(alerts)
    }

    /// 列出所有告警（分页）
    pub async fn list_alerts(
        &self,
        node_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> DbResult<Vec<Alert>> {
        let alerts = match node_id {
            Some(nid) => {
                sqlx::query_as!(
                    Alert,
                    r#"
                    SELECT id as "id!: i64",
                           rule_id as "rule_id!: i64",
                           node_id as "node_id!: i64",
                           level, message,
                           value as "value!: f64",
                           status,
                           triggered_at as "triggered_at!: i64",
                           resolved_at
                    FROM alerts
                    WHERE node_id = ?
                    ORDER BY triggered_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                    nid,
                    limit,
                    offset
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Alert,
                    r#"
                    SELECT id as "id!: i64",
                           rule_id as "rule_id!: i64",
                           node_id as "node_id!: i64",
                           level, message,
                           value as "value!: f64",
                           status,
                           triggered_at as "triggered_at!: i64",
                           resolved_at
                    FROM alerts
                    ORDER BY triggered_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                    limit,
                    offset
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(alerts)
    }
}
