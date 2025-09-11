use crate::config::schema::{BotConfig, ConfigError};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use tracing::{info, warn, error, debug};

/// 配置服務，提供centralized配置管理
/// 使用Arc<RwLock<>>實現thread-safe存取
pub struct ConfigService {
    config: Arc<RwLock<Option<BotConfig>>>,
    validation_engine: ValidationEngine,
}

impl ConfigService {
    /// 創建新的ConfigService實例
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            validation_engine: ValidationEngine::new(),
        }
    }
    
    /// 從檔案載入配置
    pub async fn load_config(&self, path: &Path) -> Result<(), ConfigError> {
        info!("Loading configuration from: {:?}", path);
        
        // 檢查檔案是否存在
        if !path.exists() {
            error!("Configuration file not found: {:?}", path);
            return Err(ConfigError::FileNotFound(path.to_string_lossy().to_string()));
        }
        
        // 讀取檔案內容
        let content = fs::read_to_string(path).await?;
        debug!("Configuration file read successfully, size: {} bytes", content.len());
        
        // 解析並注入環境變數
        let config = BotConfig::from_yaml_with_env(&content)?;
        debug!("Configuration parsed successfully");
        
        // 驗證配置
        self.validate_config(&config).await?;
        info!("Configuration validation passed");
        
        // 更新配置（原子操作）
        let mut guard = self.config.write().await;
        *guard = Some(config);
        info!("Configuration loaded successfully");
        
        Ok(())
    }
    
    /// 獲取當前配置
    pub async fn get_config(&self) -> Result<BotConfig, ConfigError> {
        let guard = self.config.read().await;
        guard.as_ref().cloned().ok_or(ConfigError::NotLoaded)
    }
    
    /// 設置配置（用於測試或動態更新）
    pub async fn set_config(&self, config: BotConfig) -> Result<(), ConfigError> {
        // 驗證配置
        self.validate_config(&config).await?;
        
        // 更新配置
        let mut guard = self.config.write().await;
        *guard = Some(config);
        debug!("Configuration updated directly");
        
        Ok(())
    }
    
    /// 驗證配置
    pub async fn validate_config(&self, config: &BotConfig) -> Result<(), ConfigError> {
        // 使用內建驗證
        config.validate()?;
        
        // 使用ValidationEngine進行額外驗證
        self.validation_engine.validate_bot_config(config)?;
        
        Ok(())
    }
    
    /// 重載配置
    pub async fn reload_config(&self, path: &Path) -> Result<(), ConfigError> {
        info!("Reloading configuration from: {:?}", path);
        
        // 保存舊配置作為備份
        let old_config = self.get_config().await.ok();
        
        // 嘗試載入新配置
        match self.load_config(path).await {
            Ok(_) => {
                info!("Configuration reloaded successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to reload configuration: {}", e);
                
                // 如果載入失敗，恢復舊配置
                if let Some(old) = old_config {
                    let mut guard = self.config.write().await;
                    *guard = Some(old);
                    warn!("Restored previous configuration due to reload failure");
                }
                
                Err(e)
            }
        }
    }
    
    /// 清除配置
    pub async fn clear_config(&self) {
        let mut guard = self.config.write().await;
        *guard = None;
        debug!("Configuration cleared");
    }
    
    /// 檢查是否已載入配置
    pub async fn is_loaded(&self) -> bool {
        let guard = self.config.read().await;
        guard.is_some()
    }
}

/// 驗證引擎，提供詳細的配置驗證邏輯
pub struct ValidationEngine {
    // 可以在此加入驗證規則配置
}

impl ValidationEngine {
    /// 創建新的ValidationEngine實例
    pub fn new() -> Self {
        Self {}
    }
    
    /// 驗證整個bot配置
    pub fn validate_bot_config(&self, config: &BotConfig) -> Result<(), ConfigError> {
        // 驗證discord token
        self.validate_discord_token(&config.bot_config.discord_token)?;
        
        // 驗證URL
        self.validate_url(&config.bot_config.llm_config.base_url)?;
        
        // 驗證protection level
        self.validate_protection_level(&config.bot_config.protection_level)?;
        
        // 驗證max tokens
        self.validate_max_tokens(config.bot_config.llm_config.max_tokens)?;
        
        Ok(())
    }
    
    /// 驗證Discord token格式
    pub fn validate_discord_token(&self, token: &str) -> Result<(), ConfigError> {
        if token.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Discord token cannot be empty".to_string()
            ));
        }
        
        // Discord token不應包含空格
        if token.contains(' ') {
            return Err(ConfigError::ValidationError(
                "Discord token cannot contain spaces".to_string()
            ));
        }
        
        // 可以加入更多Discord token格式驗證
        
        Ok(())
    }
    
    /// 驗證URL格式
    pub fn validate_url(&self, url: &str) -> Result<(), ConfigError> {
        if url.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "URL cannot be empty".to_string()
            ));
        }
        
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ConfigError::ValidationError(
                format!("Invalid URL format: {}", url)
            ));
        }
        
        // 可以使用url crate進行更詳細的驗證
        
        Ok(())
    }
    
    /// 驗證protection level
    pub fn validate_protection_level(&self, level: &str) -> Result<(), ConfigError> {
        const VALID_LEVELS: &[&str] = &["low", "medium", "high"];
        
        if !VALID_LEVELS.contains(&level) {
            return Err(ConfigError::ValidationError(
                format!("Invalid protection level: {}. Must be one of: {:?}", level, VALID_LEVELS)
            ));
        }
        
        Ok(())
    }
    
    /// 驗證max tokens範圍
    pub fn validate_max_tokens(&self, max_tokens: i32) -> Result<(), ConfigError> {
        if max_tokens <= 0 {
            return Err(ConfigError::ValidationError(
                format!("max_tokens must be positive, got: {}", max_tokens)
            ));
        }
        
        if max_tokens > 32000 {
            return Err(ConfigError::ValidationError(
                format!("max_tokens exceeds maximum limit (32000), got: {}", max_tokens)
            ));
        }
        
        Ok(())
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::write;
    
    #[tokio::test]
    async fn test_config_service_lifecycle() {
        let service = ConfigService::new();
        
        // 初始狀態應該是未載入
        assert!(!service.is_loaded().await);
        assert!(service.get_config().await.is_err());
        
        // 創建測試配置
        let config = BotConfig {
            bot_config: crate::config::schema::BotConfigInner {
                discord_token: "test_token".to_string(),
                llm_config: crate::config::schema::LlmConfig {
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
                custom_params: std::collections::HashMap::new(),
            }
        };
        
        // 設置配置
        service.set_config(config.clone()).await.unwrap();
        assert!(service.is_loaded().await);
        
        // 獲取配置
        let retrieved = service.get_config().await.unwrap();
        assert_eq!(retrieved.bot_config.discord_token, "test_token");
        
        // 清除配置
        service.clear_config().await;
        assert!(!service.is_loaded().await);
    }
}
