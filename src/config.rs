use std::env;
use std::time::Duration;
use crate::error::{DiscordError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: Duration,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub enable_redis: bool,
    pub fallback_to_memory: bool,
    pub namespace: String,
}

#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub authorized_admins: Vec<i64>,
}

impl AdminConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let admin_ids_str = env::var("ADMIN_USER_IDS")
            .unwrap_or_else(|_| "".to_string());

        let authorized_admins = if admin_ids_str.is_empty() {
            // 如果沒有設置環境變數，使用預設的測試 ID
            vec![123456789_i64] // 預設管理員 ID，生產環境中應該從環境變數設置
        } else {
            // 解析逗號分隔的用戶 ID 列表
            admin_ids_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect()
        };

        if authorized_admins.is_empty() {
            return Err(DiscordError::ConfigError(
                "至少需要一個授權管理員 ID".to_string()
            ));
        }

        Ok(AdminConfig {
            authorized_admins,
        })
    }

    /// 創建測試用管理員配置
    pub fn for_test() -> Self {
        AdminConfig {
            authorized_admins: vec![123456789_i64, 987654321_i64],
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/droas".to_string());

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("DATABASE_MAX_CONNECTIONS must be a number".to_string()))?;

        let min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("DATABASE_MIN_CONNECTIONS must be a number".to_string()))?;

        let connection_timeout = env::var("DATABASE_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("DATABASE_CONNECTION_TIMEOUT must be a number".to_string()))?;

        Ok(DatabaseConfig {
            url: database_url,
            max_connections,
            min_connections,
            connection_timeout,
        })
    }

    /// 創建測試用資料庫配置
    pub fn for_test() -> Self {
        DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/droas_test".to_string()),
            max_connections: 5,
            min_connections: 1,
            connection_timeout: 30,
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let default_ttl_secs = env::var("CACHE_DEFAULT_TTL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("CACHE_DEFAULT_TTL_SECS must be a number".to_string()))?;

        let max_connections = env::var("CACHE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("CACHE_MAX_CONNECTIONS must be a number".to_string()))?;

        let connection_timeout_secs = env::var("CACHE_CONNECTION_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("CACHE_CONNECTION_TIMEOUT_SECS must be a number".to_string()))?;

        let enable_redis = env::var("CACHE_ENABLE_REDIS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("CACHE_ENABLE_REDIS must be a boolean".to_string()))?;

        let fallback_to_memory = env::var("CACHE_FALLBACK_TO_MEMORY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|_| DiscordError::ConfigError("CACHE_FALLBACK_TO_MEMORY must be a boolean".to_string()))?;

        let namespace = env::var("CACHE_NAMESPACE")
            .unwrap_or_else(|_| "droas".to_string());

        Ok(CacheConfig {
            redis_url,
            default_ttl: Duration::from_secs(default_ttl_secs),
            max_connections,
            connection_timeout: Duration::from_secs(connection_timeout_secs),
            enable_redis,
            fallback_to_memory,
            namespace,
        })
    }

    /// 創建預設快取配置
    pub fn default() -> Self {
        CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: Duration::from_secs(300), // 5 分鐘
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            enable_redis: true,
            fallback_to_memory: true,
            namespace: "droas".to_string(),
        }
    }

    /// 創建僅記憶體快取配置
    pub fn memory_only() -> Self {
        let mut config = Self::default();
        config.enable_redis = false;
        config.fallback_to_memory = true;
        config
    }

    /// 創建測試用快取配置
    pub fn for_test() -> Self {
        CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: Duration::from_secs(1), // 1 秒 TTL 用於測試
            max_connections: 2,
            connection_timeout: Duration::from_secs(1),
            enable_redis: false, // 測試時預設不使用 Redis
            fallback_to_memory: true,
            namespace: "droas_test".to_string(),
        }
    }

    /// 生成帶命名空間的快取鍵
    pub fn namespaced_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }

    /// 驗證快取配置
    pub fn validate(&self) -> Result<()> {
        if self.enable_redis && self.redis_url.is_empty() {
            return Err(DiscordError::ConfigError("Redis URL is required when Redis is enabled".to_string()));
        }

        if self.default_ttl.is_zero() {
            return Err(DiscordError::ConfigError("Cache TTL must be greater than zero".to_string()));
        }

        if self.max_connections == 0 {
            return Err(DiscordError::ConfigError("Max connections must be greater than zero".to_string()));
        }

        if self.connection_timeout.is_zero() {
            return Err(DiscordError::ConfigError("Connection timeout must be greater than zero".to_string()));
        }

        if self.namespace.is_empty() {
            return Err(DiscordError::ConfigError("Cache namespace cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| DiscordError::ConfigError("DISCORD_TOKEN environment variable not set".to_string()))?;

        // 驗證 Discord Token 格式
        Self::validate_discord_token(&discord_token)?;

        let database = DatabaseConfig::from_env()?;
        let cache = CacheConfig::from_env()?;
        let admin = AdminConfig::from_env()?;

        // 驗證配置
        cache.validate()?;

        Ok(Config {
            discord_token,
            database,
            cache,
            admin
        })
    }

    /// 驗證 Discord Token 格式
    fn validate_discord_token(token: &str) -> Result<()> {
        // 只檢查 token 是否為空
        if token.is_empty() {
            return Err(DiscordError::ConfigError("Discord token cannot be empty".to_string()));
        }

        Ok(())
    }

    pub fn new_with_token(token: String) -> Result<Self> {
        // 驗證 Discord Token 格式
        Self::validate_discord_token(&token)?;

        Ok(Config {
            discord_token: token,
            database: DatabaseConfig {
                url: "postgres://localhost/droas".to_string(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
            },
            cache: CacheConfig::default(),
            admin: AdminConfig::for_test(),
        })
    }

    /// 創建測試配置
    pub fn for_test() -> Self {
        Config {
            discord_token: "test_token".to_string(),
            database: DatabaseConfig {
                url: "postgres://localhost/droas_test".to_string(),
                max_connections: 2,
                min_connections: 1,
                connection_timeout: 5,
            },
            cache: CacheConfig::for_test(),
            admin: AdminConfig::for_test(),
        }
    }

    /// 驗證整個配置
    pub fn validate(&self) -> Result<()> {
        self.cache.validate()
    }
}