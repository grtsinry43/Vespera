use std::time::Instant;
use std::sync::Arc;
use crate::db::DbRepo;
use crate::ws::Broadcaster;
use crate::alert::AlertEngine;

/// 应用共享状态
///
/// 使用 Arc 包装后可以安全地在多个 Handler 之间共享
/// 通过 axum::extract::State 注入到 Handler 中
#[derive(Clone)]
pub struct AppState {
    /// 服务器启动时间
    pub start_time: Instant,

    /// 数据库连接池（Arc<dyn DbApi>）
    pub db: DbRepo,

    /// WebSocket 广播器
    pub broadcaster: Broadcaster,

    /// 告警引擎 (可选)
    pub alert_engine: Option<Arc<AlertEngine>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db: DbRepo) -> Self {
        let broadcaster = Broadcaster::new();

        // 创建告警引擎
        let alert_engine = Some(Arc::new(AlertEngine::new(db.clone(), broadcaster.clone())));

        Self {
            start_time: Instant::now(),
            db,
            broadcaster,
            alert_engine,
        }
    }

    /// 获取服务器运行时长（秒）
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
