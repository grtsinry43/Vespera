use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Agent 配置
    pub agent: AgentConfig,

    /// 认证配置
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 节点 ID（唯一标识符）
    pub node_id: String,

    /// Server URL
    pub server_url: String,

    /// 上报间隔（秒）
    #[serde(default = "default_report_interval")]
    pub report_interval: u64,

    /// 请求超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 重试次数
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 认证密钥
    pub secret: String,
}

fn default_report_interval() -> u64 {
    5
}

fn default_timeout() -> u64 {
    10
}

fn default_retry_attempts() -> u32 {
    3
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 从环境变量加载配置（用于 Docker）
    pub fn from_env() -> Result<Self, ConfigError> {
        let node_id = std::env::var("VESPERA_NODE_ID")
            .unwrap_or_else(|_| hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown-node".to_string())
            );

        let server_url = std::env::var("VESPERA_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let secret = std::env::var("VESPERA_SECRET")
            .map_err(|_| ConfigError::ValidationError("VESPERA_SECRET not set".to_string()))?;

        let report_interval = std::env::var("VESPERA_REPORT_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let config = Config {
            agent: AgentConfig {
                node_id,
                server_url,
                report_interval,
                timeout: default_timeout(),
                retry_attempts: default_retry_attempts(),
            },
            auth: AuthConfig { secret },
        };

        config.validate()?;
        Ok(config)
    }

    /// 验证配置的有效性
    fn validate(&self) -> Result<(), ConfigError> {
        if self.agent.node_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "node_id cannot be empty".to_string(),
            ));
        }

        if self.agent.server_url.is_empty() {
            return Err(ConfigError::ValidationError(
                "server_url cannot be empty".to_string(),
            ));
        }

        if !self.agent.server_url.starts_with("http://")
            && !self.agent.server_url.starts_with("https://")
        {
            return Err(ConfigError::ValidationError(
                "server_url must start with http:// or https://".to_string(),
            ));
        }

        if self.auth.secret.is_empty() {
            return Err(ConfigError::ValidationError(
                "secret cannot be empty".to_string(),
            ));
        }

        if self.agent.report_interval == 0 {
            return Err(ConfigError::ValidationError(
                "report_interval must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agent: AgentConfig {
                node_id: "default-node".to_string(),
                server_url: "http://localhost:3000".to_string(),
                report_interval: default_report_interval(),
                timeout: default_timeout(),
                retry_attempts: default_retry_attempts(),
            },
            auth: AuthConfig {
                secret: "change-me".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.auth.secret = "valid-secret".to_string();
        assert!(config.validate().is_ok());

        // Test empty node_id
        config.agent.node_id = "".to_string();
        assert!(config.validate().is_err());

        config.agent.node_id = "test-node".to_string();

        // Test invalid URL
        config.agent.server_url = "invalid-url".to_string();
        assert!(config.validate().is_err());

        config.agent.server_url = "http://localhost:3000".to_string();

        // Test empty secret
        config.auth.secret = "".to_string();
        assert!(config.validate().is_err());
    }
}
