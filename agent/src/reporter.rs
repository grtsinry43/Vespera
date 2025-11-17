use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use vespera_common::Metrics;
use std::time::Duration;

#[derive(Debug, Error)]
pub enum ReporterError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Server returned error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Failed to serialize metrics: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("All retry attempts failed")]
    RetryExhausted,
}

/// API 响应格式
#[derive(Debug, Deserialize)]
struct ApiResponse {
    code: i32,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
    msg: Option<String>,
}

/// 数据上报器
pub struct Reporter {
    client: Client,
    server_url: String,
    secret: String,
    retry_attempts: u32,
}

impl Reporter {
    /// 创建新的上报器实例
    pub fn new(server_url: String, secret: String, timeout: Duration, retry_attempts: u32) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_url,
            secret,
            retry_attempts,
        }
    }

    /// 上报指标数据到 Server
    pub async fn report(&self, metrics: &Metrics) -> Result<(), ReporterError> {
        let url = format!("{}/api/v1/report", self.server_url.trim_end_matches('/'));

        let mut last_error = None;

        // 指数退避重试
        for attempt in 0..self.retry_attempts {
            match self.send_request(&url, metrics).await {
                Ok(_) => {
                    if attempt > 0 {
                        tracing::info!("Report succeeded after {} retries", attempt);
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt + 1 < self.retry_attempts {
                        let delay = Duration::from_secs(2u64.pow(attempt));
                        tracing::warn!(
                            "Report attempt {} failed, retrying in {:?}...",
                            attempt + 1,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(ReporterError::RetryExhausted))
    }

    /// 发送单次请求
    async fn send_request(&self, url: &str, metrics: &Metrics) -> Result<(), ReporterError> {
        tracing::debug!("Sending metrics to {}", url);

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .header("Content-Type", "application/json")
            .json(metrics)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ReporterError::ServerError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // 解析响应
        let api_response: ApiResponse = response.json().await?;

        if api_response.code != 0 {
            return Err(ReporterError::ServerError {
                status: 200,
                message: api_response.msg.unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        tracing::debug!("Metrics reported successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reporter_creation() {
        let reporter = Reporter::new(
            "http://localhost:3000".to_string(),
            "test-secret".to_string(),
            Duration::from_secs(10),
            3,
        );

        assert_eq!(reporter.server_url, "http://localhost:3000");
        assert_eq!(reporter.secret, "test-secret");
        assert_eq!(reporter.retry_attempts, 3);
    }

    // 集成测试需要实际的 Server 运行
    // 这里只做单元测试，集成测试在实际环境中进行
}
