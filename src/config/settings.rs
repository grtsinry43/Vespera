// Configuration settings module

/// 应用配置（未来扩展）
#[derive(Clone, Debug)]
pub struct Settings {
    pub server_addr: String,
    pub server_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_addr: "0.0.0.0".to_string(),
            server_port: 3000,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
}
