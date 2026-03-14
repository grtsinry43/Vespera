use chrono::Utc;
use sysinfo::{Disks, Networks, System};
use uuid::Uuid;
use vespera_common::{DiskInfo, MetricsData, ReportRequest};

/// 节点基础信息（在整个生命周期内不变）
pub struct NodeInfo {
    pub uuid: Uuid,
    pub name: String,
    pub ip_address: String,
    pub agent_version: String,
    pub os_type: String,
    pub os_version: Option<String>,
    pub cpu_cores: i64,
    pub total_memory: i64,
    pub tags: Option<Vec<String>>,
}

/// 系统信息采集器
pub struct SystemCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    node_info: NodeInfo,
    // 上次网络统计（用于计算累计值）
    last_network_in: u64,
    last_network_out: u64,
}

impl SystemCollector {
    /// 创建新的采集器实例
    pub fn new(node_info: NodeInfo) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            node_info,
            last_network_in: 0,
            last_network_out: 0,
        }
    }

    /// 采集系统指标，返回完整的上报请求
    pub fn collect(&mut self) -> ReportRequest {
        // 刷新系统信息
        self.system.refresh_all();
        self.networks.refresh();
        self.disks.refresh();

        let timestamp = Utc::now().timestamp();

        // CPU 使用率（全局平均值）
        let cpu_usage = self.system.global_cpu_usage() as f64;

        // 内存信息
        let memory_total = self.system.total_memory() as i64;
        let memory_used = self.system.used_memory() as i64;
        let memory_usage = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };

        // 磁盘信息（每个挂载点单独记录）
        let disk_info: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|disk| {
                let total = disk.total_space() as i64;
                let used = (disk.total_space() - disk.available_space()) as i64;
                let usage = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                DiskInfo {
                    mount: disk.mount_point().to_string_lossy().to_string(),
                    used,
                    total,
                    usage,
                }
            })
            .collect();

        // 网络流量（所有网卡总和，累计值）
        let (network_in, network_out) = self
            .networks
            .iter()
            .fold((0u64, 0u64), |(rx, tx), (_, data)| {
                (rx + data.total_received(), tx + data.total_transmitted())
            });

        // 更新上次的值（用于下次增量计算）
        self.last_network_in = network_in;
        self.last_network_out = network_out;

        // 系统负载
        let load_avg = System::load_average();

        ReportRequest {
            node_uuid: self.node_info.uuid,
            node_name: self.node_info.name.clone(),
            ip_address: self.node_info.ip_address.clone(),
            agent_version: self.node_info.agent_version.clone(),
            os_type: self.node_info.os_type.clone(),
            os_version: self.node_info.os_version.clone(),
            cpu_cores: self.node_info.cpu_cores,
            total_memory: self.node_info.total_memory,
            tags: self.node_info.tags.clone(),
            metrics: MetricsData {
                timestamp,
                cpu_usage,
                memory_used,
                memory_usage,
                disk_info,
                net_in_bytes: network_in as i64,
                net_out_bytes: network_out as i64,
                load_1: Some(load_avg.one),
                load_5: Some(load_avg.five),
                load_15: Some(load_avg.fifteen),
            },
        }
    }
}

/// 获取本机 IP 地址
pub fn get_local_ip() -> String {
    // 尝试获取非回环地址
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_, ip) in interfaces {
            if !ip.is_loopback() {
                return ip.to_string();
            }
        }
    }

    // 回退到 hostname
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node_info() -> NodeInfo {
        NodeInfo {
            uuid: Uuid::new_v4(),
            name: "test-node".to_string(),
            ip_address: "192.168.1.1".to_string(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            os_type: std::env::consts::OS.to_string(),
            os_version: None,
            cpu_cores: 8,
            total_memory: 17179869184,
            tags: Some(vec!["test".to_string()]),
        }
    }

    #[test]
    fn test_collector_basic() {
        let node_info = create_test_node_info();
        let mut collector = SystemCollector::new(node_info);
        let request = collector.collect();

        // 验证基本字段
        assert_eq!(request.node_name, "test-node");
        assert!(request.metrics.timestamp > 0);

        // 验证 CPU 使用率在合理范围内
        assert!(request.metrics.cpu_usage >= 0.0 && request.metrics.cpu_usage <= 100.0);

        // 验证内存数据合理性
        assert!(request.total_memory > 0);
        assert!(request.metrics.memory_used >= 0);
    }

    #[test]
    fn test_disk_info_collection() {
        let node_info = create_test_node_info();
        let mut collector = SystemCollector::new(node_info);
        let request = collector.collect();

        // 每个磁盘的使用率应该在 0-100 范围内
        for disk in &request.metrics.disk_info {
            assert!(disk.usage >= 0.0 && disk.usage <= 100.0);
            assert!(disk.used <= disk.total);
        }
    }
}
