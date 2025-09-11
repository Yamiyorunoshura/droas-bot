use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 主配置結構，包含所有bot配置資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub bot_config: BotConfigInner,
}

/// 內部配置結構，包含具體的配置欄位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfigInner {
    /// Discord bot token，支援環境變數注入
    pub discord_token: String,
    
    /// LLM相關配置
    pub llm_config: LlmConfig,
    
    /// 系統提示詞
    pub system_prompt: String,
    
    /// 防護等級: low, medium, high
    pub protection_level: String,
    
    /// 是否啟用此bot
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// 自定義參數（可選）
    #[serde(default)]
    pub custom_params: HashMap<String, serde_yaml::Value>,
}

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM API base URL
    pub base_url: String,
    
    /// API key，支援環境變數注入
    pub api_key: String,
    
    /// 模型名稱 (e.g., gpt-4, claude-3)
    pub model: String,
    
    /// 最大token數
    pub max_tokens: i32,
    
    /// 溫度參數（可選）
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// 請求超時秒數（可選）
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

/// 配置版本資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version: String,
    pub min_compatible_version: String,
}

impl BotConfig {
    /// 從YAML字串建立配置
    pub fn from_yaml_str(yaml_str: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml_str)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
    
    /// 從YAML字串建立配置，並注入環境變數
    pub fn from_yaml_with_env(yaml_str: &str) -> Result<Self, ConfigError> {
        let processed = Self::inject_env_vars(yaml_str)?;
        Self::from_yaml_str(&processed)
    }
    
    /// 注入環境變數，替換 ${VAR_NAME} 格式的佔位符
    fn inject_env_vars(content: &str) -> Result<String, ConfigError> {
        use regex::Regex;
        
        let re = Regex::new(r"\$\{([^}]+)\}")
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        let mut result = content.to_string();
        let mut missing_vars = Vec::new();
        
        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            match std::env::var(var_name) {
                Ok(value) => {
                    let placeholder = format!("${{{}}}", var_name);
                    result = result.replace(&placeholder, &value);
                }
                Err(_) => {
                    missing_vars.push(var_name.to_string());
                }
            }
        }
        
        if !missing_vars.is_empty() {
            return Err(ConfigError::MissingEnvVars(missing_vars));
        }
        
        Ok(result)
    }
    
    /// 驗證配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 驗證 discord token 不為空
        if self.bot_config.discord_token.trim().is_empty() {
            return Err(ConfigError::ValidationError("Discord token cannot be empty".to_string()));
        }
        
        // 驗證 protection level
        let valid_levels = ["low", "medium", "high"];
        if !valid_levels.contains(&self.bot_config.protection_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid protection_level: {}, must be one of: low, medium, high", 
                        self.bot_config.protection_level)
            ));
        }
        
        // 驗證 LLM 配置
        self.bot_config.llm_config.validate()?;
        
        Ok(())
    }
}

impl LlmConfig {
    /// 驗證LLM配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 驗證 URL 格式
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(ConfigError::ValidationError(
                format!("Invalid base_url: {}, must start with http:// or https://", self.base_url)
            ));
        }
        
        // 驗證 API key 不為空
        if self.api_key.trim().is_empty() {
            return Err(ConfigError::ValidationError("API key cannot be empty".to_string()));
        }
        
        // 驗證 max_tokens 範圍
        if self.max_tokens <= 0 || self.max_tokens > 32000 {
            return Err(ConfigError::ValidationError(
                format!("Invalid max_tokens: {}, must be between 1 and 32000", self.max_tokens)
            ));
        }
        
        // 驗證 temperature 範圍
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(ConfigError::ValidationError(
                format!("Invalid temperature: {}, must be between 0.0 and 2.0", self.temperature)
            ));
        }
        
        Ok(())
    }
}

/// 配置錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Missing environment variables: {0:?}")]
    MissingEnvVars(Vec<String>),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration not loaded")]
    NotLoaded,
}

// 預設值函數
fn default_enabled() -> bool {
    true
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let config = BotConfig {
            bot_config: BotConfigInner {
                discord_token: "valid_token".to_string(),
                llm_config: LlmConfig {
                    base_url: "http://localhost:8080".to_string(),
                    api_key: "test_key".to_string(),
                    model: "gpt-4".to_string(),
                    max_tokens: 2000,
                    temperature: 0.7,
                    timeout_seconds: 30,
                },
                system_prompt: "Test prompt".to_string(),
                protection_level: "medium".to_string(),
                enabled: true,
                custom_params: HashMap::new(),
            }
        };
        
        assert!(config.validate().is_ok());
    }
}
