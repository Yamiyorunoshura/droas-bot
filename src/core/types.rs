use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, Duration};
use thiserror::Error;

/// Unique identifier for a bot instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotId(String);

impl BotId {
    /// Create a new BotId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation of the BotId
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// LLM configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
}

/// Protection level for the bot
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProtectionLevel {
    Low,
    Medium,
    High,
}

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub bot_id: BotId,
    pub discord_token: String,
    pub llm_config: LlmConfig,
    pub system_prompt: String,
    pub protection_level: ProtectionLevel,
}

impl BotConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), BotManagerError> {
        if self.discord_token.is_empty() {
            return Err(BotManagerError::ConfigError("Discord token cannot be empty".to_string()));
        }
        if self.llm_config.api_key.is_empty() {
            return Err(BotManagerError::ConfigError("LLM API key cannot be empty".to_string()));
        }
        if self.llm_config.base_url.is_empty() {
            return Err(BotManagerError::ConfigError("LLM base URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

/// Health status of a bot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Restarting,
    NotFound,
}

/// Bot state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BotState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

/// Bot instance representing a running bot
#[derive(Debug, Clone)]
pub struct BotInstance {
    pub id: BotId,
    pub config: BotConfig,
    pub state: BotState,
    pub started_at: SystemTime,
    pub restart_count: u32,
    pub last_health_check: Option<SystemTime>,
}

impl BotInstance {
    /// Create a new BotInstance
    pub fn new(config: BotConfig) -> Self {
        Self {
            id: config.bot_id.clone(),
            config,
            state: BotState::Starting,
            started_at: SystemTime::now(),
            restart_count: 0,
            last_health_check: None,
        }
    }

    /// Check if the bot is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, BotState::Running)
    }

    /// Update the health check timestamp
    pub fn update_health_check(&mut self) {
        self.last_health_check = Some(SystemTime::now());
    }

    /// Increment restart count
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }
}

/// Error types for BotManager operations
#[derive(Debug, Error)]
pub enum BotManagerError {
    #[error("Bot not found: {0}")]
    BotNotFound(BotId),
    
    #[error("Maximum capacity reached (10 bots)")]
    MaxCapacityReached,
    
    #[error("Duplicate bot ID: {0}")]
    DuplicateBotId(BotId),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Startup error: {0}")]
    StartupError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Restart policy for supervisor
#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub restart_interval: Duration,
    pub backoff_multiplier: f32,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_interval: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// Builder for BotConfig
pub struct BotConfigBuilder {
    bot_id: Option<BotId>,
    discord_token: Option<String>,
    llm_config: Option<LlmConfig>,
    system_prompt: Option<String>,
    protection_level: Option<ProtectionLevel>,
}

impl BotConfigBuilder {
    /// Create a new BotConfigBuilder
    pub fn new() -> Self {
        Self {
            bot_id: None,
            discord_token: None,
            llm_config: None,
            system_prompt: None,
            protection_level: None,
        }
    }

    /// Set the bot ID
    pub fn bot_id(mut self, id: impl Into<String>) -> Self {
        self.bot_id = Some(BotId::new(id));
        self
    }

    /// Set the Discord token
    pub fn discord_token(mut self, token: impl Into<String>) -> Self {
        self.discord_token = Some(token.into());
        self
    }

    /// Set the LLM configuration
    pub fn llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// Set the system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the protection level
    pub fn protection_level(mut self, level: ProtectionLevel) -> Self {
        self.protection_level = Some(level);
        self
    }

    /// Build the BotConfig
    pub fn build(self) -> Result<BotConfig, BotManagerError> {
        Ok(BotConfig {
            bot_id: self.bot_id.ok_or_else(|| BotManagerError::ConfigError("Bot ID is required".to_string()))?,
            discord_token: self.discord_token.ok_or_else(|| BotManagerError::ConfigError("Discord token is required".to_string()))?,
            llm_config: self.llm_config.ok_or_else(|| BotManagerError::ConfigError("LLM config is required".to_string()))?,
            system_prompt: self.system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string()),
            protection_level: self.protection_level.unwrap_or(ProtectionLevel::Medium),
        })
    }
}
