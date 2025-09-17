//! 配置管理模組
//! 
//! 此模組負責處理 DROAS Bot 的所有配置，包括：
//! - Discord API 配置和令牌管理
//! - 資料庫連接配置
//! - 應用程序設定
//! - 安全的敏感資料處理

pub mod secrets;
pub mod models;
pub mod repository;
pub mod cache;
pub mod transaction;
pub mod service;

// Re-export main service for convenience
pub use service::{GuildConfigService, ConfigServiceStats, ConfigUpdateResult};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use secrets::{validate_discord_token_format, validate_discord_token_with_api, sanitize_token_error};

/// Main configuration structure for the DROAS Bot
/// 
/// This structure contains all necessary configuration for the bot to operate,
/// including Discord API settings, database configuration, and application settings.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// Discord API configuration
    pub discord: DiscordConfig,
    /// Database connection configuration  
    pub database: DatabaseConfig,
    /// Application-specific configuration
    pub app: AppConfig,
}

// Custom Debug implementation to hide sensitive information
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("discord", &self.discord)
            .field("database", &"[DATABASE CONFIG]")
            .field("app", &self.app)
            .finish()
    }
}

/// Discord API configuration with secure token handling
/// 
/// This structure stores Discord-related configuration including the bot token
/// (which is kept private to prevent accidental exposure) and application ID.
#[derive(Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Bot token (kept private for security)
    token: String,
    /// Discord application ID
    pub application_id: u64,
}

// Custom Debug implementation to hide the token
impl fmt::Debug for DiscordConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscordConfig")
            .field("token", &"[HIDDEN]")
            .field("application_id", &self.application_id)
            .finish()
    }
}

impl DiscordConfig {
    /// Get the Discord bot token
    /// 
    /// This is the only way to access the token, ensuring it's accessed intentionally
    /// and not accidentally exposed through debug output or serialization.
    pub fn token(&self) -> &str {
        &self.token
    }
    
    /// Create a test configuration (for testing only)
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            token: "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.test_token_do_not_use_in_production".to_string(),
            application_id: 491638711814368277,
        }
    }
}

/// Database configuration
/// 
/// Contains settings for database connections including connection pool settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
}

/// Application-specific configuration
/// 
/// Contains general application settings that don't fit into other categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Logging level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Directory for caching images
    pub image_cache_dir: String,
    /// Maximum allowed image size in megabytes
    pub max_image_size_mb: u64,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// This method reads configuration from environment variables and .env file.
    /// Required variables: DISCORD_BOT_TOKEN, DISCORD_APPLICATION_ID
    /// Optional variables have sensible defaults.
    /// 
    /// # Errors
    /// 
    /// Returns an error if required environment variables are missing or invalid.
    pub async fn load() -> Result<Self> {
        // Load environment variables from .env file if present
        dotenv::dotenv().ok();

        let discord_token = env::var("DISCORD_BOT_TOKEN")
            .context("DISCORD_BOT_TOKEN environment variable is required")?;
        
        let discord_app_id = env::var("DISCORD_APPLICATION_ID")
            .context("DISCORD_APPLICATION_ID environment variable is required")?
            .parse::<u64>()
            .context("DISCORD_APPLICATION_ID must be a valid u64")?;

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://droas_bot.db".to_string());

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);

        let min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u32>()
            .unwrap_or(1);

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        let image_cache_dir = env::var("IMAGE_CACHE_DIR")
            .unwrap_or_else(|_| "./assets/cache".to_string());

        let max_image_size_mb = env::var("MAX_IMAGE_SIZE_MB")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .unwrap_or(5);

        let config = Config {
            discord: DiscordConfig {
                token: discord_token,
                application_id: discord_app_id,
            },
            database: DatabaseConfig {
                url: database_url,
                max_connections,
                min_connections,
            },
            app: AppConfig {
                log_level,
                image_cache_dir,
                max_image_size_mb,
            },
        };

        // Validate configuration before returning
        config.validate().await?;
        
        Ok(config)
    }

    /// Validate the loaded configuration
    /// 
    /// This method performs sanity checks on the configuration to ensure
    /// it contains valid values that won't cause runtime errors.
    /// Now includes Discord token format and API validation.
    /// 
    /// # Errors
    /// 
    /// Returns an error if any configuration value is invalid.
    pub async fn validate(&self) -> Result<()> {
        // 基本令牌檢查
        if self.discord.token().is_empty() {
            anyhow::bail!("Discord token cannot be empty. Please set DISCORD_BOT_TOKEN environment variable.");
        }

        // Discord 令牌格式驗證
        if let Err(e) = validate_discord_token_format(self.discord.token()) {
            let sanitized_error = sanitize_token_error(&e.to_string(), self.discord.token());
            anyhow::bail!("Discord token format validation failed: {}", sanitized_error);
        }

        // Discord API 令牌驗證（可選，因為需要網路連接）
        if std::env::var("SKIP_DISCORD_API_VALIDATION").is_err() {
            if let Err(e) = validate_discord_token_with_api(self.discord.token()).await {
                let sanitized_error = sanitize_token_error(&e.to_string(), self.discord.token());
                tracing::warn!("Discord API token validation failed: {}. Continuing anyway.", sanitized_error);
                // 注意：我們記錄警告但不阻止啟動，因為網路問題可能是暫時的
            }
        }

        if self.discord.application_id == 0 {
            anyhow::bail!("Discord application ID cannot be 0. Please set DISCORD_APPLICATION_ID environment variable.");
        }

        if self.database.max_connections < self.database.min_connections {
            anyhow::bail!(
                "Database max_connections ({}) must be >= min_connections ({})",
                self.database.max_connections,
                self.database.min_connections
            );
        }

        if self.app.max_image_size_mb == 0 {
            anyhow::bail!("Max image size must be greater than 0 MB");
        }

        Ok(())
    }

    /// 為測試目的提供的簡化驗證方法
    /// 
    /// 此方法跳過需要網路連接的驗證，主要用於單元測試。
    #[cfg(test)]
    pub fn validate_offline(&self) -> Result<()> {
        if self.discord.token().is_empty() {
            anyhow::bail!("Discord token cannot be empty. Please set DISCORD_BOT_TOKEN environment variable.");
        }

        if let Err(e) = validate_discord_token_format(self.discord.token()) {
            let sanitized_error = sanitize_token_error(&e.to_string(), self.discord.token());
            anyhow::bail!("Discord token format validation failed: {}", sanitized_error);
        }

        if self.discord.application_id == 0 {
            anyhow::bail!("Discord application ID cannot be 0. Please set DISCORD_APPLICATION_ID environment variable.");
        }

        if self.database.max_connections < self.database.min_connections {
            anyhow::bail!(
                "Database max_connections ({}) must be >= min_connections ({})",
                self.database.max_connections,
                self.database.min_connections
            );
        }

        if self.app.max_image_size_mb == 0 {
            anyhow::bail!("Max image size must be greater than 0 MB");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // 測試助手函數：清理環境變數
    fn clear_test_env() {
        env::remove_var("DISCORD_BOT_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DATABASE_URL");
        env::remove_var("LOG_LEVEL");
        env::remove_var("SKIP_DISCORD_API_VALIDATION");
    }

    // 測試助手函數：設置有效的測試環境
    fn setup_valid_test_env() {
        env::set_var("DISCORD_BOT_TOKEN", "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.example_token_do_not_use");
        env::set_var("DISCORD_APPLICATION_ID", "491638711814368277");
        env::set_var("SKIP_DISCORD_API_VALIDATION", "1"); // 跳過網路驗證
    }

    #[tokio::test]
    async fn test_token_reading_success() {
        clear_test_env();
        setup_valid_test_env();

        let result = Config::load().await;
        assert!(result.is_ok(), "配置加載應該成功");
        
        let config = result.unwrap();
        assert_eq!(config.discord.token(), "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.example_token_do_not_use");
        assert_eq!(config.discord.application_id, 491638711814368277);
        
        clear_test_env();
    }

    #[tokio::test]
    async fn test_token_missing_environment_variable() {
        clear_test_env();
        env::set_var("DISCORD_APPLICATION_ID", "123456789");

        let result = Config::load().await;
        assert!(result.is_err(), "缺少 DISCORD_BOT_TOKEN 時應該失敗");
        assert!(result.unwrap_err().to_string().contains("DISCORD_BOT_TOKEN"));
        
        clear_test_env();
    }

    #[tokio::test]
    async fn test_token_empty_environment_variable() {
        clear_test_env();
        env::set_var("DISCORD_BOT_TOKEN", "");
        env::set_var("DISCORD_APPLICATION_ID", "123456789");
        env::set_var("SKIP_DISCORD_API_VALIDATION", "1");

        let result = Config::load().await;
        assert!(result.is_err(), "空令牌應該在驗證時失敗");
        
        clear_test_env();
    }

    #[test]
    fn test_token_not_logged_in_debug() {
        clear_test_env();
        
        let config = Config {
            discord: DiscordConfig {
                token: "sensitive_token_should_not_appear".to_string(),
                application_id: 123456789,
            },
            database: DatabaseConfig {
                url: "sqlite://test.db".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            app: AppConfig {
                log_level: "info".to_string(),
                image_cache_dir: "./test_cache".to_string(),
                max_image_size_mb: 5,
            },
        };

        let debug_output = format!("{:?}", config);
        assert!(!debug_output.contains("sensitive_token_should_not_appear"), 
                "Debug 輸出不應包含實際令牌");
        assert!(debug_output.contains("[HIDDEN]"), 
                "Debug 輸出應顯示 [HIDDEN] 而非實際令牌");
        
        clear_test_env();
    }

    #[test]
    fn test_token_not_in_error_messages() {
        let token = "sensitive_token_in_error";
        let config = Config {
            discord: DiscordConfig {
                token: token.to_string(),
                application_id: 0, // 無效 ID 會觸發驗證錯誤
            },
            database: DatabaseConfig {
                url: "sqlite://test.db".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            app: AppConfig {
                log_level: "info".to_string(),
                image_cache_dir: "./test_cache".to_string(),
                max_image_size_mb: 5,
            },
        };

        let validation_result = config.validate_offline();
        assert!(validation_result.is_err(), "驗證應該失敗");
        
        let error_message = validation_result.unwrap_err().to_string();
        assert!(!error_message.contains(token), 
                "錯誤訊息不應包含敏感令牌");
    }

    #[test]
    fn test_config_validation_success() {
        let config = Config {
            discord: DiscordConfig {
                token: "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.example_token_do_not_use".to_string(),
                application_id: 123456789,
            },
            database: DatabaseConfig {
                url: "sqlite://test.db".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            app: AppConfig {
                log_level: "info".to_string(),
                image_cache_dir: "./test_cache".to_string(),
                max_image_size_mb: 5,
            },
        };

        let result = config.validate_offline();
        assert!(result.is_ok(), "有效配置的驗證應該成功");
    }

    #[test]
    fn test_config_validation_invalid_database_connections() {
        let config = Config {
            discord: DiscordConfig {
                token: "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.example_token_do_not_use".to_string(),
                application_id: 123456789,
            },
            database: DatabaseConfig {
                url: "sqlite://test.db".to_string(),
                max_connections: 1, // 小於 min_connections
                min_connections: 5,
            },
            app: AppConfig {
                log_level: "info".to_string(),
                image_cache_dir: "./test_cache".to_string(),
                max_image_size_mb: 5,
            },
        };

        let result = config.validate_offline();
        assert!(result.is_err(), "無效的資料庫連接配置應該失敗");
        assert!(result.unwrap_err().to_string().contains("max_connections"));
    }
}