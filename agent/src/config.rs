use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

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
    /// 节点 UUID（唯一标识符，首次启动自动生成）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_uuid: Option<String>,

    /// 节点名称（可自定义）
    pub node_name: String,

    /// Server URL
    pub server_url: String,

    /// 上报间隔（秒）
    #[serde(default = "default_report_interval")]
    pub report_interval: u64,

    /// 服务检查间隔（秒）
    #[serde(default = "default_service_check_interval")]
    pub service_check_interval: u64,

    /// 请求超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 重试次数
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,

    /// 节点标签（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 认证密钥
    pub secret: String,
}

fn default_report_interval() -> u64 {
    5
}

fn default_service_check_interval() -> u64 {
    300 // 默认 5 分钟检查一次服务
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
        let content = fs::read_to_string(&path)?;
        let mut config: Config = toml::from_str(&content)?;

        // 如果没有 UUID，生成一个并保存
        if config.agent.node_uuid.is_none() {
            let uuid = Uuid::new_v4().to_string();
            tracing::info!("Generated new node UUID: {}", uuid);
            config.agent.node_uuid = Some(uuid);

            // 保存回配置文件
            if let Err(e) = config.save_to_file(&path) {
                tracing::warn!("Failed to save UUID to config file: {}", e);
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// 保存配置到文件
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::ValidationError(format!("Failed to serialize config: {}", e))
        })?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 从环境变量加载配置（用于 Docker）
    pub fn from_env() -> Result<Self, ConfigError> {
        // 尝试从持久化文件读取 UUID
        let uuid_file = PathBuf::from("/var/lib/vespera/node.uuid");
        let node_uuid = if uuid_file.exists() {
            fs::read_to_string(&uuid_file).ok()
        } else {
            let uuid = Uuid::new_v4().to_string();
            // 尝试创建目录并保存 UUID
            if let Some(parent) = uuid_file.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&uuid_file, &uuid);
            Some(uuid)
        };

        let node_name = std::env::var("VESPERA_NODE_NAME").unwrap_or_else(|_| {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown-node".to_string())
        });

        let server_url = std::env::var("VESPERA_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let secret = std::env::var("VESPERA_SECRET")
            .map_err(|_| ConfigError::ValidationError("VESPERA_SECRET not set".to_string()))?;

        let report_interval = std::env::var("VESPERA_REPORT_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let service_check_interval = std::env::var("VESPERA_SERVICE_CHECK_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        let tags = std::env::var("VESPERA_TAGS")
            .ok()
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

        let config = Config {
            agent: AgentConfig {
                node_uuid,
                node_name,
                server_url,
                report_interval,
                service_check_interval,
                timeout: default_timeout(),
                retry_attempts: default_retry_attempts(),
                tags,
            },
            auth: AuthConfig { secret },
        };

        config.validate()?;
        Ok(config)
    }

    /// 验证配置的有效性
    fn validate(&self) -> Result<(), ConfigError> {
        if self.agent.node_name.is_empty() {
            return Err(ConfigError::ValidationError(
                "node_name cannot be empty".to_string(),
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

    /// 获取节点 UUID（确保一定有值）
    pub fn get_node_uuid(&self) -> Uuid {
        self.agent
            .node_uuid
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(|| {
                tracing::warn!("Invalid UUID in config, generating new one");
                Uuid::new_v4()
            })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agent: AgentConfig {
                node_uuid: None,
                node_name: "default-node".to_string(),
                server_url: "http://localhost:3000".to_string(),
                report_interval: default_report_interval(),
                service_check_interval: default_service_check_interval(),
                timeout: default_timeout(),
                retry_attempts: default_retry_attempts(),
                tags: None,
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

        // Test empty node_name
        config.agent.node_name = "".to_string();
        assert!(config.validate().is_err());

        config.agent.node_name = "test-node".to_string();

        // Test invalid URL
        config.agent.server_url = "invalid-url".to_string();
        assert!(config.validate().is_err());

        config.agent.server_url = "http://localhost:3000".to_string();

        // Test empty secret
        config.auth.secret = "".to_string();
        assert!(config.validate().is_err());
    }
}
