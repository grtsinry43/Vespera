use axum::{extract::State, response::Json};
use serde::Serialize;
use std::sync::Arc;

use crate::common::Response;
use crate::server::AppState;

/// 健康检查响应数据
#[derive(Serialize)]
pub struct HealthCheckData {
    /// 服务状态
    pub status: String,
    /// 服务器运行时长（秒）
    pub uptime_secs: u64,
    /// 版本信息
    pub version: String,
}

/// 健康检查端点
///
/// GET /health
///
/// 用于检测服务是否正常运行，同时返回服务器运行时长等信息
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<Response<HealthCheckData>> {
    let data = HealthCheckData {
        status: "OK".to_string(),
        uptime_secs: state.uptime_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(Response::success(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::db;

    #[tokio::test]
    async fn test_health_check() {
        // 创建测试数据库
        let db_repo = db::init_db()
            .await
            .expect("Failed to initialize test database");

        let state = Arc::new(AppState::new(db_repo));

        let response = health_check(State(state)).await;
        let resp = response.0;

        assert_eq!(resp.code, 0);
        assert!(resp.data.is_some());

        let data = resp.data.unwrap();
        assert_eq!(data.status, "OK");
        assert_eq!(data.version, env!("CARGO_PKG_VERSION"));
    }
}
