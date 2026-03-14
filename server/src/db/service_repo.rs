use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use vespera_common::{
    Service, ServiceCheckResult, ServiceCreate, ServiceStatus, ServiceStatusPoint,
    ServiceStatusRecord, ServiceType, ServiceUpdate,
};

use super::error::{DbError, DbResult};

/// 服务仓库 Trait
#[async_trait::async_trait]
pub trait ServiceRepository: Send + Sync {
    /// 创建服务
    async fn create_service(&self, service: &ServiceCreate) -> DbResult<Service>;

    /// 获取服务详情
    async fn get_service(&self, id: i64) -> DbResult<Option<Service>>;

    /// 获取所有服务
    async fn list_services(&self) -> DbResult<Vec<Service>>;

    /// 获取公开服务
    async fn list_public_services(&self) -> DbResult<Vec<Service>>;

    /// 获取启用的服务
    async fn list_enabled_services(&self) -> DbResult<Vec<Service>>;

    /// 获取节点关联的服务
    async fn list_services_by_node(&self, node_id: i64) -> DbResult<Vec<Service>>;

    /// 更新服务
    async fn update_service(&self, id: i64, update: &ServiceUpdate) -> DbResult<Service>;

    /// 删除服务
    async fn delete_service(&self, id: i64) -> DbResult<()>;

    /// 记录服务检查结果
    async fn record_check_result(&self, result: &ServiceCheckResult) -> DbResult<i64>;

    /// 批量记录服务检查结果
    async fn record_check_results(&self, results: &[ServiceCheckResult]) -> DbResult<()>;

    /// 获取服务最新状态
    async fn get_latest_status(&self, service_id: i64) -> DbResult<Option<ServiceStatusRecord>>;

    /// 获取服务状态历史（最近30小时，每小时一个点）
    async fn get_status_history(&self, service_id: i64) -> DbResult<Vec<ServiceStatusPoint>>;

    /// 清理旧的服务状态数据（保留30小时）
    async fn cleanup_old_status(&self, hours: i64) -> DbResult<u64>;
}

/// SQLite Service Repository 实现
pub struct SqliteServiceRepo {
    pool: Pool<Sqlite>,
}

impl SqliteServiceRepo {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// 将 headers HashMap 序列化为 JSON
    fn serialize_headers(headers: &Option<HashMap<String, String>>) -> DbResult<Option<String>> {
        match headers {
            Some(h) => Ok(Some(
                serde_json::to_string(h).map_err(|e| DbError::SerializationError(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    /// 从 JSON 反序列化 headers
    fn deserialize_headers(json: Option<&str>) -> DbResult<Option<HashMap<String, String>>> {
        match json {
            Some(j) => Ok(Some(
                serde_json::from_str(j).map_err(|e| DbError::SerializationError(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl ServiceRepository for SqliteServiceRepo {
    async fn create_service(&self, service: &ServiceCreate) -> DbResult<Service> {
        let now = chrono::Utc::now().timestamp();
        let headers_json = Self::serialize_headers(&service.headers)?;

        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO services (
                node_id, name, type, target, check_interval, timeout,
                method, expected_code, expected_body, headers, enabled,
                is_public, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(service.node_id)
        .bind(&service.name)
        .bind(service.service_type.as_str())
        .bind(&service.target)
        .bind(service.check_interval)
        .bind(service.timeout)
        .bind(&service.method)
        .bind(service.expected_code)
        .bind(&service.expected_body)
        .bind(headers_json)
        .bind(service.enabled)
        .bind(service.is_public)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        self.get_service(id).await?.ok_or(DbError::NotFound)
    }

    async fn get_service(&self, id: i64) -> DbResult<Option<Service>> {
        let row = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, i64, i64, String, i64, Option<String>, Option<String>, bool, bool, i64, i64)>(
            "SELECT id, node_id, name, type, target, check_interval, timeout, method, expected_code, expected_body, headers, enabled, is_public, created_at, updated_at FROM services WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((
                id,
                node_id,
                name,
                service_type_str,
                target,
                check_interval,
                timeout,
                method,
                expected_code,
                expected_body,
                headers_json,
                enabled,
                is_public,
                created_at,
                updated_at,
            )) => {
                let service_type = ServiceType::from_str(&service_type_str).ok_or_else(|| {
                    DbError::SerializationError(format!(
                        "Invalid service type: {}",
                        service_type_str
                    ))
                })?;
                let headers = Self::deserialize_headers(headers_json.as_deref())?;

                Ok(Some(Service {
                    id,
                    node_id,
                    name,
                    service_type,
                    target,
                    check_interval,
                    timeout,
                    method,
                    expected_code,
                    expected_body,
                    headers,
                    enabled,
                    is_public,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn list_services(&self) -> DbResult<Vec<Service>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, i64, i64, String, i64, Option<String>, Option<String>, bool, bool, i64, i64)>(
            "SELECT id, node_id, name, type, target, check_interval, timeout, method, expected_code, expected_body, headers, enabled, is_public, created_at, updated_at FROM services ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    id,
                    node_id,
                    name,
                    service_type_str,
                    target,
                    check_interval,
                    timeout,
                    method,
                    expected_code,
                    expected_body,
                    headers_json,
                    enabled,
                    is_public,
                    created_at,
                    updated_at,
                )| {
                    let service_type =
                        ServiceType::from_str(&service_type_str).ok_or_else(|| {
                            DbError::SerializationError(format!(
                                "Invalid service type: {}",
                                service_type_str
                            ))
                        })?;
                    let headers = Self::deserialize_headers(headers_json.as_deref())?;

                    Ok(Service {
                        id,
                        node_id,
                        name,
                        service_type,
                        target,
                        check_interval,
                        timeout,
                        method,
                        expected_code,
                        expected_body,
                        headers,
                        enabled,
                        is_public,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect()
    }

    async fn list_public_services(&self) -> DbResult<Vec<Service>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, i64, i64, String, i64, Option<String>, Option<String>, bool, bool, i64, i64)>(
            "SELECT id, node_id, name, type, target, check_interval, timeout, method, expected_code, expected_body, headers, enabled, is_public, created_at, updated_at FROM services WHERE is_public = 1 ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    id,
                    node_id,
                    name,
                    service_type_str,
                    target,
                    check_interval,
                    timeout,
                    method,
                    expected_code,
                    expected_body,
                    headers_json,
                    enabled,
                    is_public,
                    created_at,
                    updated_at,
                )| {
                    let service_type =
                        ServiceType::from_str(&service_type_str).ok_or_else(|| {
                            DbError::SerializationError(format!(
                                "Invalid service type: {}",
                                service_type_str
                            ))
                        })?;
                    let headers = Self::deserialize_headers(headers_json.as_deref())?;

                    Ok(Service {
                        id,
                        node_id,
                        name,
                        service_type,
                        target,
                        check_interval,
                        timeout,
                        method,
                        expected_code,
                        expected_body,
                        headers,
                        enabled,
                        is_public,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect()
    }

    async fn list_enabled_services(&self) -> DbResult<Vec<Service>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, i64, i64, String, i64, Option<String>, Option<String>, bool, bool, i64, i64)>(
            "SELECT id, node_id, name, type, target, check_interval, timeout, method, expected_code, expected_body, headers, enabled, is_public, created_at, updated_at FROM services WHERE enabled = 1 ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    id,
                    node_id,
                    name,
                    service_type_str,
                    target,
                    check_interval,
                    timeout,
                    method,
                    expected_code,
                    expected_body,
                    headers_json,
                    enabled,
                    is_public,
                    created_at,
                    updated_at,
                )| {
                    let service_type =
                        ServiceType::from_str(&service_type_str).ok_or_else(|| {
                            DbError::SerializationError(format!(
                                "Invalid service type: {}",
                                service_type_str
                            ))
                        })?;
                    let headers = Self::deserialize_headers(headers_json.as_deref())?;

                    Ok(Service {
                        id,
                        node_id,
                        name,
                        service_type,
                        target,
                        check_interval,
                        timeout,
                        method,
                        expected_code,
                        expected_body,
                        headers,
                        enabled,
                        is_public,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect()
    }

    async fn list_services_by_node(&self, node_id: i64) -> DbResult<Vec<Service>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, i64, i64, String, i64, Option<String>, Option<String>, bool, bool, i64, i64)>(
            "SELECT id, node_id, name, type, target, check_interval, timeout, method, expected_code, expected_body, headers, enabled, is_public, created_at, updated_at FROM services WHERE node_id = ? ORDER BY created_at DESC"
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    id,
                    node_id,
                    name,
                    service_type_str,
                    target,
                    check_interval,
                    timeout,
                    method,
                    expected_code,
                    expected_body,
                    headers_json,
                    enabled,
                    is_public,
                    created_at,
                    updated_at,
                )| {
                    let service_type =
                        ServiceType::from_str(&service_type_str).ok_or_else(|| {
                            DbError::SerializationError(format!(
                                "Invalid service type: {}",
                                service_type_str
                            ))
                        })?;
                    let headers = Self::deserialize_headers(headers_json.as_deref())?;

                    Ok(Service {
                        id,
                        node_id,
                        name,
                        service_type,
                        target,
                        check_interval,
                        timeout,
                        method,
                        expected_code,
                        expected_body,
                        headers,
                        enabled,
                        is_public,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect()
    }

    async fn update_service(&self, id: i64, update: &ServiceUpdate) -> DbResult<Service> {
        let now = chrono::Utc::now().timestamp();

        // 先获取当前服务
        let current = self.get_service(id).await?.ok_or(DbError::NotFound)?;

        // 构建更新字段
        let name = update.name.as_ref().unwrap_or(&current.name);
        let target = update.target.as_ref().unwrap_or(&current.target);
        let check_interval = update.check_interval.unwrap_or(current.check_interval);
        let timeout = update.timeout.unwrap_or(current.timeout);
        let method = update.method.as_ref().unwrap_or(&current.method);
        let expected_code = update.expected_code.unwrap_or(current.expected_code);
        let expected_body = update
            .expected_body
            .as_ref()
            .or(current.expected_body.as_ref());
        let headers = update.headers.as_ref().or(current.headers.as_ref());
        let enabled = update.enabled.unwrap_or(current.enabled);
        let is_public = update.is_public.unwrap_or(current.is_public);

        let headers_json = Self::serialize_headers(&headers.cloned())?;

        sqlx::query(
            r#"
            UPDATE services
            SET name = ?, target = ?, check_interval = ?, timeout = ?,
                method = ?, expected_code = ?, expected_body = ?, headers = ?,
                enabled = ?, is_public = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(target)
        .bind(check_interval)
        .bind(timeout)
        .bind(method)
        .bind(expected_code)
        .bind(expected_body)
        .bind(headers_json)
        .bind(enabled)
        .bind(is_public)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_service(id).await?.ok_or(DbError::NotFound)
    }

    async fn delete_service(&self, id: i64) -> DbResult<()> {
        let result = sqlx::query("DELETE FROM services WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound);
        }

        Ok(())
    }

    async fn record_check_result(&self, result: &ServiceCheckResult) -> DbResult<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO service_status (
                service_id, agent_id, status, response_time,
                status_code, error_message, checked_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(result.service_id)
        .bind(result.agent_id)
        .bind(result.status.as_str())
        .bind(result.response_time)
        .bind(result.status_code)
        .bind(&result.error_message)
        .bind(result.checked_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn record_check_results(&self, results: &[ServiceCheckResult]) -> DbResult<()> {
        if results.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for result in results {
            sqlx::query(
                r#"
                INSERT INTO service_status (
                    service_id, agent_id, status, response_time,
                    status_code, error_message, checked_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(result.service_id)
            .bind(result.agent_id)
            .bind(result.status.as_str())
            .bind(result.response_time)
            .bind(result.status_code)
            .bind(&result.error_message)
            .bind(result.checked_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_latest_status(&self, service_id: i64) -> DbResult<Option<ServiceStatusRecord>> {
        let row = sqlx::query_as::<_, (i64, i64, Option<i64>, String, Option<i64>, Option<i64>, Option<String>, i64)>(
            r#"
            SELECT id, service_id, agent_id, status, response_time, status_code, error_message, checked_at
            FROM service_status
            WHERE service_id = ?
            ORDER BY checked_at DESC
            LIMIT 1
            "#
        )
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((
                id,
                service_id,
                agent_id,
                status_str,
                response_time,
                status_code,
                error_message,
                checked_at,
            )) => {
                let status = ServiceStatus::from_str(&status_str).ok_or_else(|| {
                    DbError::SerializationError(format!("Invalid service status: {}", status_str))
                })?;

                Ok(Some(ServiceStatusRecord {
                    id,
                    service_id,
                    agent_id,
                    status,
                    response_time,
                    status_code,
                    error_message,
                    checked_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_status_history(&self, service_id: i64) -> DbResult<Vec<ServiceStatusPoint>> {
        let now = chrono::Utc::now().timestamp();
        let thirty_hours_ago = now - (30 * 3600);

        // 获取最近30小时的所有数据点
        let rows = sqlx::query_as::<_, (String, Option<i64>, i64)>(
            r#"
            SELECT status, response_time, checked_at
            FROM service_status
            WHERE service_id = ? AND checked_at >= ?
            ORDER BY checked_at ASC
            "#,
        )
        .bind(service_id)
        .bind(thirty_hours_ago)
        .fetch_all(&self.pool)
        .await?;

        // 将数据按小时分组，每小时取最后一个点
        let mut hourly_points: HashMap<i64, ServiceStatusPoint> = HashMap::new();

        for (status_str, response_time, checked_at) in rows {
            let hour_key = checked_at / 3600; // 按小时分组
            let status = ServiceStatus::from_str(&status_str).ok_or_else(|| {
                DbError::SerializationError(format!("Invalid service status: {}", status_str))
            })?;

            hourly_points.insert(
                hour_key,
                ServiceStatusPoint {
                    timestamp: checked_at,
                    status,
                    response_time,
                },
            );
        }

        // 生成最近30个小时的完整时间序列
        let mut result = Vec::new();
        for i in 0..30 {
            let hour_timestamp = now - (i * 3600);
            let hour_key = hour_timestamp / 3600;

            if let Some(point) = hourly_points.get(&hour_key) {
                result.push(point.clone());
            } else {
                // 缺失数据用 unknown 填充
                result.push(ServiceStatusPoint {
                    timestamp: hour_timestamp,
                    status: ServiceStatus::Unknown,
                    response_time: None,
                });
            }
        }

        result.reverse(); // 按时间正序排列
        Ok(result)
    }

    async fn cleanup_old_status(&self, hours: i64) -> DbResult<u64> {
        let cutoff = chrono::Utc::now().timestamp() - (hours * 3600);

        let result = sqlx::query("DELETE FROM service_status WHERE checked_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
