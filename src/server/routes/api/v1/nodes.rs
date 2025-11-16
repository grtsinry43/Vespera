use axum::{routing::post, Router};
use std::sync::Arc;

use crate::server::AppState;

/// API v1 路由节点占位
///
/// POST /api/v1/nodes/report - Agent 数据上报（未来实现）
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/nodes/report", post(nodes_report_placeholder))
}

/// 占位 Handler - 待实现
async fn nodes_report_placeholder() -> &'static str {
    "Agent report endpoint - coming soon"
}
