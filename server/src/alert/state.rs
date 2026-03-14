//! 告警状态存储
//!
//! 内存存储,用于跟踪告警静默期和持续时间

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::alert::models::*;

/// 告警状态存储 (内存)
///
/// 用于实现:
/// 1. 静默期检查 (避免重复告警)
/// 2. 持续时间跟踪 (CPU 持续 5 分钟高于阈值才告警)
pub struct AlertStateStore {
    /// 告警静默状态: (node_id, rule_id) -> last_fired_at
    silence_map: RwLock<HashMap<(i64, i64), Instant>>,

    /// 持续时间跟踪: (node_id, rule_type) -> first_triggered_at
    duration_map: RwLock<HashMap<(i64, String), Instant>>,
}

impl AlertStateStore {
    pub fn new() -> Self {
        Self {
            silence_map: RwLock::new(HashMap::new()),
            duration_map: RwLock::new(HashMap::new()),
        }
    }

    /// 检查是否应该触发告警 (静默期检查)
    pub async fn should_fire(
        &self,
        node_id: i64,
        rule_id: i64,
        silence_duration_secs: i64,
    ) -> bool {
        let map = self.silence_map.read().await;
        let key = (node_id, rule_id);

        if let Some(last_fired) = map.get(&key) {
            // 检查是否仍在静默期
            let elapsed = last_fired.elapsed().as_secs() as i64;
            elapsed >= silence_duration_secs
        } else {
            // 从未触发过,可以触发
            true
        }
    }

    /// 标记告警已触发
    pub async fn mark_fired(&self, node_id: i64, rule_id: i64) {
        let mut map = self.silence_map.write().await;
        let key = (node_id, rule_id);
        map.insert(key, Instant::now());
    }

    /// 检查持续时间是否超过阈值
    ///
    /// 返回 true 表示已持续超过 duration_secs
    pub async fn check_duration_exceeded(
        &self,
        node_id: i64,
        rule_type: &AlertRuleType,
        duration_secs: i64,
    ) -> bool {
        let mut map = self.duration_map.write().await;
        let key = (node_id, rule_type.as_str().to_string());

        let first_triggered = map.entry(key).or_insert_with(Instant::now);

        first_triggered.elapsed().as_secs() as i64 >= duration_secs
    }

    /// 清除持续状态 (当指标恢复正常时调用)
    pub async fn clear_duration_state(&self, node_id: i64, rule_type: &AlertRuleType) {
        let mut map = self.duration_map.write().await;
        let key = (node_id, rule_type.as_str().to_string());
        map.remove(&key);
    }

    /// 清理过期的静默状态 (可选,定期清理)
    pub async fn cleanup_expired_silence(&self, max_age_secs: i64) {
        let mut map = self.silence_map.write().await;
        let now = Instant::now();

        map.retain(|_, last_fired| now.duration_since(*last_fired).as_secs() < max_age_secs as u64);
    }

    /// 清理过期的持续状态，避免长期运行时状态无界增长
    pub async fn cleanup_expired_duration(&self, max_age_secs: i64) {
        let mut map = self.duration_map.write().await;
        let now = Instant::now();

        map.retain(|_, first_triggered| {
            now.duration_since(*first_triggered).as_secs() < max_age_secs as u64
        });
    }

    /// 获取统计信息 (用于监控)
    pub async fn stats(&self) -> StateStats {
        let silence_map = self.silence_map.read().await;
        let duration_map = self.duration_map.read().await;

        StateStats {
            silence_count: silence_map.len(),
            duration_count: duration_map.len(),
        }
    }
}

impl Default for AlertStateStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态统计
#[derive(Debug, Clone)]
pub struct StateStats {
    pub silence_count: usize,
    pub duration_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_should_fire_first_time() {
        let store = AlertStateStore::new();

        // 首次应该触发
        assert!(store.should_fire(1, 1, 300).await);
    }

    #[tokio::test]
    async fn test_should_fire_silence_period() {
        let store = AlertStateStore::new();

        // 标记已触发
        store.mark_fired(1, 1).await;

        // 立即检查,应该在静默期内
        assert!(!store.should_fire(1, 1, 300).await);

        // 等待 1 秒后仍在静默期
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        assert!(!store.should_fire(1, 1, 300).await);
    }

    #[tokio::test]
    async fn test_duration_check() {
        let store = AlertStateStore::new();
        let rule_type = AlertRuleType::CpuHigh;

        // 第一次检查,刚开始计时
        assert!(!store.check_duration_exceeded(1, &rule_type, 2).await);

        // 等待 2 秒
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 第二次检查,应该已超过
        assert!(store.check_duration_exceeded(1, &rule_type, 2).await);
    }

    #[tokio::test]
    async fn test_clear_duration_state() {
        let store = AlertStateStore::new();
        let rule_type = AlertRuleType::CpuHigh;

        // 开始计时
        store.check_duration_exceeded(1, &rule_type, 1).await;

        // 等待 1 秒
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // 清除状态
        store.clear_duration_state(1, &rule_type).await;

        // 再次检查,应该重新开始计时
        assert!(!store.check_duration_exceeded(1, &rule_type, 2).await);
    }

    #[tokio::test]
    async fn test_stats() {
        let store = AlertStateStore::new();

        store.mark_fired(1, 1).await;
        store.mark_fired(2, 1).await;
        store
            .check_duration_exceeded(1, &AlertRuleType::CpuHigh, 1)
            .await;

        let stats = store.stats().await;
        assert_eq!(stats.silence_count, 2);
        assert_eq!(stats.duration_count, 1);
    }
}
