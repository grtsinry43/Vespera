use serde::{Deserialize, Serialize};

/// Agent 采集的系统指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// 节点 ID
    pub node_id: String,

    /// 采集时间戳 (Unix timestamp, seconds)
    pub timestamp: i64,

    /// CPU 使用率 (0-100)
    pub cpu_usage: f32,

    /// 总内存 (bytes)
    pub memory_total: u64,

    /// 已使用内存 (bytes)
    pub memory_used: u64,

    /// 总 Swap (bytes)
    pub swap_total: u64,

    /// 已使用 Swap (bytes)
    pub swap_used: u64,

    /// 总磁盘空间 (bytes)
    pub disk_total: u64,

    /// 已使用磁盘空间 (bytes)
    pub disk_used: u64,

    /// 网络接收流量 (bytes, since last report)
    pub network_in: u64,

    /// 网络发送流量 (bytes, since last report)
    pub network_out: u64,

    /// 1 分钟平均负载
    pub load_1: f64,

    /// 5 分钟平均负载
    pub load_5: f64,

    /// 15 分钟平均负载
    pub load_15: f64,

    /// 系统运行时间 (seconds)
    pub uptime: u64,
}

impl Metrics {
    /// 验证指标数据的合法性
    pub fn validate(&self) -> Result<(), String> {
        if self.cpu_usage < 0.0 || self.cpu_usage > 100.0 {
            return Err(format!("Invalid CPU usage: {}", self.cpu_usage));
        }

        if self.memory_used > self.memory_total {
            return Err("Memory used exceeds total".to_string());
        }

        if self.swap_used > self.swap_total {
            return Err("Swap used exceeds total".to_string());
        }

        if self.disk_used > self.disk_total {
            return Err("Disk used exceeds total".to_string());
        }

        Ok(())
    }

    /// 计算内存使用百分比
    pub fn memory_usage_percent(&self) -> f32 {
        if self.memory_total == 0 {
            return 0.0;
        }
        (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32
    }

    /// 计算磁盘使用百分比
    pub fn disk_usage_percent(&self) -> f32 {
        if self.disk_total == 0 {
            return 0.0;
        }
        (self.disk_used as f64 / self.disk_total as f64 * 100.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_validation() {
        let mut metrics = Metrics {
            node_id: "test-node".to_string(),
            timestamp: 1234567890,
            cpu_usage: 50.0,
            memory_total: 1024 * 1024 * 1024, // 1GB
            memory_used: 512 * 1024 * 1024,   // 512MB
            swap_total: 1024 * 1024 * 1024,
            swap_used: 0,
            disk_total: 10 * 1024 * 1024 * 1024, // 10GB
            disk_used: 5 * 1024 * 1024 * 1024,   // 5GB
            network_in: 1024,
            network_out: 2048,
            load_1: 0.5,
            load_5: 0.6,
            load_15: 0.7,
            uptime: 3600,
        };

        assert!(metrics.validate().is_ok());

        // Test invalid CPU
        metrics.cpu_usage = 150.0;
        assert!(metrics.validate().is_err());

        metrics.cpu_usage = 50.0;

        // Test memory overflow
        metrics.memory_used = metrics.memory_total + 1;
        assert!(metrics.validate().is_err());
    }

    #[test]
    fn test_percentage_calculations() {
        let metrics = Metrics {
            node_id: "test-node".to_string(),
            timestamp: 1234567890,
            cpu_usage: 50.0,
            memory_total: 1000,
            memory_used: 500,
            swap_total: 0,
            swap_used: 0,
            disk_total: 1000,
            disk_used: 750,
            network_in: 0,
            network_out: 0,
            load_1: 0.0,
            load_5: 0.0,
            load_15: 0.0,
            uptime: 0,
        };

        assert_eq!(metrics.memory_usage_percent(), 50.0);
        assert_eq!(metrics.disk_usage_percent(), 75.0);
    }
}
