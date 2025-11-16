use std::time::Instant;

/// 应用共享状态
///
/// 使用 Arc 包装后可以安全地在多个 Handler 之间共享
/// 通过 axum::extract::State 注入到 Handler 中
#[derive(Clone)]
pub struct AppState {
    /// 服务器启动时间
    pub start_time: Instant,

    // 未来扩展字段（预留）：
    // pub db_pool: Arc<SqlitePool>,
    // pub metrics: Arc<Metrics>,
    // pub config: Arc<Settings>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// 获取服务器运行时长（秒）
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.uptime_secs() == 0);
    }

    #[test]
    fn test_uptime() {
        let state = AppState::new();
        sleep(Duration::from_millis(100));
        assert!(state.uptime_secs() == 0); // Less than 1 second

        sleep(Duration::from_millis(950));
        assert!(state.uptime_secs() >= 1);
    }

    #[test]
    fn test_clone() {
        let state1 = AppState::new();
        let state2 = state1.clone();

        // 克隆后的状态应该有相同的启动时间
        assert_eq!(state1.start_time, state2.start_time);
    }
}
