use sysinfo::{Disks, Networks, System};
use vespera_common::Metrics;
use chrono::Utc;

/// 系统信息采集器
pub struct SystemCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    node_id: String,
    // 上次网络统计（用于计算增量）
    last_network_in: u64,
    last_network_out: u64,
}

impl SystemCollector {
    /// 创建新的采集器实例
    pub fn new(node_id: String) -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            node_id,
            last_network_in: 0,
            last_network_out: 0,
        }
    }

    /// 采集系统指标
    pub fn collect(&mut self) -> Metrics {
        // 刷新系统信息
        self.system.refresh_all();
        self.networks.refresh();
        // disks 不需要频繁刷新，保持初始化时的状态

        let timestamp = Utc::now().timestamp();

        // CPU 使用率（全局平均值）
        let cpu_usage = self.system.global_cpu_usage();

        // 内存信息
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();

        // Swap 信息
        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();

        // 磁盘信息（汇总所有磁盘）
        let (disk_total, disk_used) = self.disks.iter().fold((0u64, 0u64), |(total, used), disk| {
            (
                total + disk.total_space(),
                used + (disk.total_space() - disk.available_space()),
            )
        });

        // 网络流量（所有网卡总和）
        let (network_in, network_out) = self.networks.iter().fold((0u64, 0u64), |(rx, tx), (_, data)| {
            (
                rx + data.total_received(),
                tx + data.total_transmitted(),
            )
        });

        // 计算自上次采集以来的网络增量
        let network_in_delta = if self.last_network_in == 0 {
            0 // 第一次采集，增量为 0
        } else {
            network_in.saturating_sub(self.last_network_in)
        };

        let network_out_delta = if self.last_network_out == 0 {
            0
        } else {
            network_out.saturating_sub(self.last_network_out)
        };

        // 更新上次的值
        self.last_network_in = network_in;
        self.last_network_out = network_out;

        // 系统负载
        let load_avg = System::load_average();
        let load_1 = load_avg.one;
        let load_5 = load_avg.five;
        let load_15 = load_avg.fifteen;

        // 系统运行时间
        let uptime = System::uptime();

        Metrics {
            node_id: self.node_id.clone(),
            timestamp,
            cpu_usage,
            memory_total,
            memory_used,
            swap_total,
            swap_used,
            disk_total,
            disk_used,
            network_in: network_in_delta,
            network_out: network_out_delta,
            load_1,
            load_5,
            load_15,
            uptime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_basic() {
        let mut collector = SystemCollector::new("test-node".to_string());
        let metrics = collector.collect();

        // 验证基本字段
        assert_eq!(metrics.node_id, "test-node");
        assert!(metrics.timestamp > 0);

        // 验证 CPU 使用率在合理范围内
        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);

        // 验证内存数据合理性
        assert!(metrics.memory_total > 0);
        assert!(metrics.memory_used <= metrics.memory_total);

        // 验证磁盘数据
        if metrics.disk_total > 0 {
            assert!(metrics.disk_used <= metrics.disk_total);
        }

        // 验证系统运行时间
        assert!(metrics.uptime > 0);
    }

    #[test]
    fn test_network_delta() {
        let mut collector = SystemCollector::new("test-node".to_string());

        // 第一次采集，网络增量应该为 0
        let metrics1 = collector.collect();
        assert_eq!(metrics1.network_in, 0);
        assert_eq!(metrics1.network_out, 0);

        // 模拟等待（实际测试中可能没有网络活动，增量仍可能为 0）
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 第二次采集，增量应该是合理值
        let _metrics2 = collector.collect();
        // 网络增量会是非负数（因为是 u64 类型）
        // 如果没有网络活动，增量就是 0
    }

    #[test]
    fn test_metrics_validation() {
        let mut collector = SystemCollector::new("test-node".to_string());
        let metrics = collector.collect();

        // 使用 common 库中的验证方法
        assert!(metrics.validate().is_ok());
    }
}
