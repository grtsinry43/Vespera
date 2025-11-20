//! 服务监控检查器模块
//!
//! 负责对配置的服务进行 HTTP/TCP 健康检查

use chrono::Utc;
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use vespera_common::{Service, ServiceCheckResult, ServiceStatus, ServiceType};

/// 服务检查错误
#[derive(Debug, Error)]
pub enum CheckError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("TCP connection failed: {0}")]
    TcpError(#[from] std::io::Error),

    #[error("Request timeout")]
    Timeout,

    #[error("Unexpected status code: {0}")]
    UnexpectedStatusCode(u16),

    #[error("Response body mismatch")]
    BodyMismatch,

    #[error("Invalid service type: {0:?}")]
    InvalidServiceType(ServiceType),

    #[error("Invalid target format: {0}")]
    InvalidTarget(String),
}

/// 服务检查器
pub struct ServiceChecker {
    client: Client,
    agent_id: Option<i64>,
}

impl ServiceChecker {
    /// 创建新的服务检查器
    ///
    /// # 性能优化
    /// - 使用连接池复用 HTTP 连接
    /// - 配置合理的超时时间
    /// - 限制最大连接数
    pub fn new(agent_id: Option<i64>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10) // 每个 host 最多保持 10 个空闲连接
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, agent_id }
    }

    /// 检查单个服务
    ///
    /// 根据服务类型执行相应的检查逻辑
    pub async fn check_service(&self, service: &Service) -> ServiceCheckResult {
        let start = std::time::Instant::now();

        let (status, status_code, error_message) = match service.service_type {
            ServiceType::Http => self.check_http(service).await,
            ServiceType::Tcp => self.check_tcp(service).await,
        };

        let response_time = if matches!(status, ServiceStatus::Up) {
            Some(start.elapsed().as_millis() as i64)
        } else {
            None
        };

        ServiceCheckResult {
            service_id: service.id,
            agent_id: self.agent_id,
            status,
            response_time,
            status_code,
            error_message,
            checked_at: Utc::now().timestamp(),
        }
    }

    /// 批量检查服务
    ///
    /// 并发执行检查，使用 Semaphore 限制并发数
    pub async fn check_services(&self, services: &[Service]) -> Vec<ServiceCheckResult> {
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        if services.is_empty() {
            return Vec::new();
        }

        // 限制并发检查数量（最多同时检查 20 个服务）
        let semaphore = Arc::new(Semaphore::new(20));
        let mut tasks = Vec::new();

        for service in services {
            let service = service.clone();
            let checker = self.clone_checker();
            let permit = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                checker.check_service(&service).await
            });

            tasks.push(task);
        }

        // 等待所有检查完成
        let mut results = Vec::new();
        for task in tasks {
            if let Ok(result) = task.await {
                results.push(result);
            }
        }

        results
    }

    /// HTTP 服务检查
    async fn check_http(
        &self,
        service: &Service,
    ) -> (ServiceStatus, Option<i64>, Option<String>) {
        // 构建请求
        let mut request = match service.method.to_uppercase().as_str() {
            "GET" => self.client.get(&service.target),
            "POST" => self.client.post(&service.target),
            "HEAD" => self.client.head(&service.target),
            "PUT" => self.client.put(&service.target),
            _ => self.client.get(&service.target), // 默认 GET
        };

        // 添加自定义 headers
        if let Some(headers) = &service.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        // 添加超时
        let timeout_duration = Duration::from_secs(service.timeout as u64);

        // 发送请求
        let response = match timeout(timeout_duration, request.send()).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                return (
                    ServiceStatus::Error,
                    None,
                    Some(format!("HTTP error: {}", e)),
                )
            }
            Err(_) => return (ServiceStatus::Timeout, None, Some("Request timeout".to_string())),
        };

        let status_code = response.status().as_u16() as i64;

        // 检查状态码
        if status_code != service.expected_code {
            return (
                ServiceStatus::Down,
                Some(status_code),
                Some(format!(
                    "Unexpected status code: {} (expected: {})",
                    status_code, service.expected_code
                )),
            );
        }

        // 如果配置了期望的响应体，则检查
        if let Some(expected_body) = &service.expected_body {
            match response.text().await {
                Ok(body) => {
                    if !body.contains(expected_body) {
                        return (
                            ServiceStatus::Down,
                            Some(status_code),
                            Some("Response body mismatch".to_string()),
                        );
                    }
                }
                Err(e) => {
                    return (
                        ServiceStatus::Error,
                        Some(status_code),
                        Some(format!("Failed to read response body: {}", e)),
                    )
                }
            }
        }

        // 所有检查通过
        (ServiceStatus::Up, Some(status_code), None)
    }

    /// TCP 端口检查
    ///
    /// 检查 TCP 端口是否可达
    ///
    /// # Target 格式
    /// - `host:port` (例如：example.com:3306, 192.168.1.1:22)
    ///
    /// # 检查流程
    /// 1. 解析 host:port
    /// 2. 尝试建立 TCP 连接
    /// 3. 可选：发送测试数据并读取响应（如果配置了 expected_body）
    async fn check_tcp(
        &self,
        service: &Service,
    ) -> (ServiceStatus, Option<i64>, Option<String>) {
        // 解析目标地址 (host:port)
        let addr = match self.parse_tcp_target(&service.target) {
            Ok(addr) => addr,
            Err(e) => {
                return (
                    ServiceStatus::Error,
                    None,
                    Some(format!("Invalid target format: {}", e)),
                )
            }
        };

        // 设置超时
        let timeout_duration = Duration::from_secs(service.timeout as u64);

        // 尝试建立 TCP 连接
        let stream = match timeout(timeout_duration, TcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return (
                    ServiceStatus::Down,
                    None,
                    Some(format!("Connection failed: {}", e)),
                )
            }
            Err(_) => return (ServiceStatus::Timeout, None, Some("Connection timeout".to_string())),
        };

        // 如果配置了 expected_body，则尝试发送/接收数据
        if let Some(expected_data) = &service.expected_body {
            match self.tcp_exchange_data(stream, expected_data, timeout_duration).await {
                Ok(()) => (ServiceStatus::Up, None, None),
                Err(e) => (
                    ServiceStatus::Down,
                    None,
                    Some(format!("Data exchange failed: {}", e)),
                ),
            }
        } else {
            // 仅连接测试，连接成功即表示端口开放
            (ServiceStatus::Up, None, None)
        }
    }

    /// 解析 TCP 目标地址
    ///
    /// 支持格式：
    /// - `host:port` (example.com:3306)
    /// - `ip:port` (192.168.1.1:22)
    fn parse_tcp_target(&self, target: &str) -> Result<String, String> {
        // 验证格式是否为 host:port
        if !target.contains(':') {
            return Err(format!(
                "Missing port. Expected format: host:port, got: {}",
                target
            ));
        }

        let parts: Vec<&str> = target.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid format: {}", target));
        }

        let port_str = parts[0];
        let host = parts[1];

        // 验证端口是否为有效数字
        if port_str.parse::<u16>().is_err() {
            return Err(format!("Invalid port: {}", port_str));
        }

        // 验证 host 不为空
        if host.is_empty() {
            return Err("Empty host".to_string());
        }

        Ok(target.to_string())
    }

    /// TCP 数据交换
    ///
    /// 用于发送测试数据并验证响应（可选功能）
    ///
    /// # 使用场景
    /// - Redis: 发送 "PING"，期望返回 "+PONG"
    /// - MySQL: 检查握手响应
    /// - 其他需要验证协议响应的场景
    async fn tcp_exchange_data(
        &self,
        mut stream: TcpStream,
        expected_data: &str,
        timeout_duration: Duration,
    ) -> Result<(), String> {
        // 如果 expected_data 包含 "|"，则分为发送数据和期望响应
        // 格式：send_data|expected_response
        // 例如：PING|PONG
        let (send_data, expected_response) = if expected_data.contains('|') {
            let parts: Vec<&str> = expected_data.splitn(2, '|').collect();
            (Some(parts[0]), Some(parts[1]))
        } else {
            // 如果没有 |，仅作为期望响应
            (None, Some(expected_data))
        };

        // 发送数据（如果有）
        if let Some(data) = send_data {
            let send_bytes = data.as_bytes();
            match timeout(timeout_duration, stream.write_all(send_bytes)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => return Err(format!("Failed to send data: {}", e)),
                Err(_) => return Err("Send timeout".to_string()),
            }
        }

        // 读取响应（如果需要验证）
        if let Some(expected) = expected_response {
            let mut buffer = vec![0u8; 1024];
            match timeout(timeout_duration, stream.read(&mut buffer)).await {
                Ok(Ok(n)) => {
                    if n == 0 {
                        return Err("Connection closed by remote".to_string());
                    }
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    if !response.contains(expected) {
                        return Err(format!(
                            "Response mismatch. Expected: {}, Got: {}",
                            expected, response
                        ));
                    }
                }
                Ok(Err(e)) => return Err(format!("Failed to read response: {}", e)),
                Err(_) => return Err("Read timeout".to_string()),
            }
        }

        Ok(())
    }

    /// 克隆检查器（共享 HTTP 客户端）
    fn clone_checker(&self) -> Self {
        Self {
            client: self.client.clone(),
            agent_id: self.agent_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_check_success() {
        // 需要一个测试 HTTP 服务器
        // 这里仅作为示例，实际测试需要 mock server
    }

    #[test]
    fn test_checker_creation() {
        let checker = ServiceChecker::new(Some(1));
        assert_eq!(checker.agent_id, Some(1));
    }

    #[test]
    fn test_parse_tcp_target_valid() {
        let checker = ServiceChecker::new(None);

        // 测试有效格式
        assert!(checker.parse_tcp_target("example.com:3306").is_ok());
        assert!(checker.parse_tcp_target("192.168.1.1:22").is_ok());
        assert!(checker.parse_tcp_target("localhost:8080").is_ok());
        assert!(checker.parse_tcp_target("db.example.com:5432").is_ok());
    }

    #[test]
    fn test_parse_tcp_target_invalid() {
        let checker = ServiceChecker::new(None);

        // 测试无效格式
        assert!(checker.parse_tcp_target("example.com").is_err()); // 缺少端口
        assert!(checker.parse_tcp_target(":3306").is_err()); // 缺少主机
        assert!(checker.parse_tcp_target("example.com:abc").is_err()); // 无效端口
        assert!(checker.parse_tcp_target("example.com:99999").is_err()); // 端口超出范围
    }

    #[tokio::test]
    async fn test_tcp_check_localhost() {
        // 测试本地端口检查
        // 注意：这个测试需要有一个本地服务在运行
        // 在实际环境中可能需要 skip 或 mock
    }
}

