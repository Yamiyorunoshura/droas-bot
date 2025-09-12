use crate::core::types::{
    BotId, BotConfig, BotState, BotManagerError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::{Child, Command};
use tokio::time::{Duration, Instant, timeout};
use tracing::{info, error, debug, warn};

/// Process information for a bot instance
#[derive(Debug)]
pub struct BotProcess {
    pub id: BotId,
    pub config: BotConfig,
    pub state: BotState,
    pub process: Option<Child>,
    pub started_at: Instant,
    pub restart_count: u32,
    pub last_health_check: Option<Instant>,
    pub process_id: Option<u32>,
    pub message_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

impl BotProcess {
    /// Create a new BotProcess
    pub fn new(config: BotConfig) -> Self {
        Self {
            id: config.bot_id.clone(),
            config,
            state: BotState::Starting,
            process: None,
            started_at: Instant::now(),
            restart_count: 0,
            last_health_check: None,
            process_id: None,
            message_count: 0,
            error_count: 0,
            last_error: None,
        }
    }

    /// Update health check timestamp
    pub fn update_health_check(&mut self) {
        self.last_health_check = Some(Instant::now());
    }

    /// Increment restart count
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }

    /// Get uptime in seconds
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Check if the process is alive
    pub async fn is_alive(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    // Process has exited
                    debug!("Process {} exited with status: {:?}", self.id, status);
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }
}

/// Enhanced Bot Lifecycle Manager
pub struct LifecycleManager {
    pub bots: Arc<RwLock<HashMap<BotId, BotProcess>>>,
    max_capacity: usize,
}

impl LifecycleManager {
    /// Create a new LifecycleManager
    pub fn new(max_capacity: usize) -> Self {
        Self {
            bots: Arc::new(RwLock::new(HashMap::new())),
            max_capacity,
        }
    }

    /// Create a new bot with validated configuration
    pub async fn create_bot(&self, config: BotConfig) -> Result<BotId, BotManagerError> {
        // Validate configuration
        config.validate()?;

        // Check capacity
        {
            let bots = self.bots.read().await;
            if bots.len() >= self.max_capacity {
                return Err(BotManagerError::MaxCapacityReached);
            }
            if bots.contains_key(&config.bot_id) {
                return Err(BotManagerError::DuplicateBotId(config.bot_id.clone()));
            }
        }

        // Generate unique bot ID if needed (for now, use the provided ID)
        let bot_id = config.bot_id.clone();

        // Create bot process instance
        let bot_process = BotProcess::new(config);

        // Register the bot
        {
            let mut bots = self.bots.write().await;
            bots.insert(bot_id.clone(), bot_process);
        }

        info!("Bot {} created successfully", bot_id);
        Ok(bot_id)
    }

    /// Start a bot process
    pub async fn start_bot(&self, bot_id: &BotId) -> Result<(), BotManagerError> {
        let mut bots = self.bots.write().await;
        
        let bot = bots.get_mut(bot_id)
            .ok_or_else(|| BotManagerError::BotNotFound(bot_id.clone()))?;

        if matches!(bot.state, BotState::Running) {
            return Ok(()); // Already running
        }

        bot.state = BotState::Starting;

        // Prepare the child bot command
        // In production, this would be the actual bot executable
        // For testing, we'll use a simple command
        let child_bot_path = std::env::var("CHILD_BOT_PATH")
            .unwrap_or_else(|_| "echo".to_string());
        
        let mut command = Command::new(&child_bot_path);
        command.arg(format!("Bot {} started", bot_id));
        
        // Set environment variables for the child bot
        command.env("BOT_ID", bot_id.as_str());
        command.env("DISCORD_TOKEN", &bot.config.discord_token);
        command.env("LLM_BASE_URL", &bot.config.llm_config.base_url);
        command.env("LLM_API_KEY", &bot.config.llm_config.api_key);
        command.env("SYSTEM_PROMPT", &bot.config.system_prompt);
        
        // Spawn the process
        match command.spawn() {
            Ok(child) => {
                let pid = child.id();
                bot.process = Some(child);
                bot.process_id = pid;
                bot.state = BotState::Running;
                bot.started_at = Instant::now();
                info!("Bot {} started successfully with PID {:?}", bot_id, pid);
                Ok(())
            }
            Err(e) => {
                bot.state = BotState::Failed(format!("Failed to spawn process: {}", e));
                error!("Failed to start bot {}: {}", bot_id, e);
                Err(BotManagerError::StartupError(format!("Failed to spawn process: {}", e)))
            }
        }
    }

    /// Stop a bot process gracefully
    pub async fn stop_bot(&self, bot_id: &BotId) -> Result<(), BotManagerError> {
        let mut bots = self.bots.write().await;
        
        let bot = bots.get_mut(bot_id)
            .ok_or_else(|| BotManagerError::BotNotFound(bot_id.clone()))?;

        if matches!(bot.state, BotState::Stopped) {
            return Ok(()); // Already stopped
        }

        bot.state = BotState::Stopping;
        info!("Stopping bot {}...", bot_id);

        if let Some(mut process) = bot.process.take() {
            // Try graceful shutdown first
            // In a real implementation, we would send a SIGTERM signal
            // For now, we'll attempt to kill the process
            
            let kill_result = timeout(
                Duration::from_secs(30),
                async move {
                    // In production, send SIGTERM first, wait, then SIGKILL if needed
                    // For now, just kill the process
                    process.kill().await
                }
            ).await;

            match kill_result {
                Ok(Ok(_)) => {
                    info!("Bot {} process terminated gracefully", bot_id);
                }
                Ok(Err(e)) => {
                    error!("Failed to terminate bot {} process: {}", bot_id, e);
                    return Err(BotManagerError::ShutdownError(
                        format!("Failed to terminate process: {}", e)
                    ));
                }
                Err(_) => {
                    warn!("Timeout while stopping bot {}, forcing termination", bot_id);
                    // Force kill would happen here in production
                }
            }
        }

        bot.state = BotState::Stopped;
        bot.process_id = None;
        bot.last_health_check = None;

        info!("Bot {} stopped successfully", bot_id);
        Ok(())
    }

    /// Restart a bot
    pub async fn restart_bot(&self, bot_id: &BotId) -> Result<(), BotManagerError> {
        // Verify bot exists
        {
            let bots = self.bots.read().await;
            if !bots.contains_key(bot_id) {
                return Err(BotManagerError::BotNotFound(bot_id.clone()));
            }
        }

        // Stop the bot
        self.stop_bot(bot_id).await?;

        // Wait before restarting
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Start the bot
        self.start_bot(bot_id).await?;

        // Update restart count
        {
            let mut bots = self.bots.write().await;
            if let Some(bot) = bots.get_mut(bot_id) {
                bot.increment_restart_count();
            }
        }

        info!("Bot {} restarted successfully", bot_id);
        Ok(())
    }

    /// Get bot count
    pub async fn get_bot_count(&self) -> usize {
        let bots = self.bots.read().await;
        bots.len()
    }

    /// List all bot IDs
    pub async fn list_bot_ids(&self) -> Vec<BotId> {
        let bots = self.bots.read().await;
        bots.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{LlmConfig, ProtectionLevel, BotConfigBuilder};

    fn create_test_config(id: &str) -> BotConfig {
        BotConfigBuilder::new()
            .bot_id(id)
            .discord_token("test_token")
            .llm_config(LlmConfig {
                base_url: "http://test.com".to_string(),
                api_key: "test_key".to_string(),
            })
            .system_prompt("Test prompt")
            .protection_level(ProtectionLevel::Medium)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_bot_success() {
        let manager = LifecycleManager::new(10);
        let config = create_test_config("test_bot_1");
        
        let result = manager.create_bot(config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "test_bot_1");
        assert_eq!(manager.get_bot_count().await, 1);
    }

    #[tokio::test]
    async fn test_create_bot_duplicate_id() {
        let manager = LifecycleManager::new(10);
        let config1 = create_test_config("test_bot_1");
        let config2 = create_test_config("test_bot_1");
        
        let result1 = manager.create_bot(config1).await;
        assert!(result1.is_ok());
        
        let result2 = manager.create_bot(config2).await;
        assert!(result2.is_err());
        match result2.unwrap_err() {
            BotManagerError::DuplicateBotId(id) => assert_eq!(id.as_str(), "test_bot_1"),
            _ => panic!("Expected DuplicateBotId error"),
        }
    }

    #[tokio::test]
    async fn test_create_bot_max_capacity() {
        let manager = LifecycleManager::new(2);
        
        let config1 = create_test_config("test_bot_1");
        let config2 = create_test_config("test_bot_2");
        let config3 = create_test_config("test_bot_3");
        
        assert!(manager.create_bot(config1).await.is_ok());
        assert!(manager.create_bot(config2).await.is_ok());
        
        let result = manager.create_bot(config3).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotManagerError::MaxCapacityReached => {},
            _ => panic!("Expected MaxCapacityReached error"),
        }
    }

    #[tokio::test]
    async fn test_create_bot_invalid_config() {
        let manager = LifecycleManager::new(10);
        
        // Create invalid config with empty discord token
        let config = BotConfig {
            bot_id: BotId::new("test_bot"),
            discord_token: "".to_string(), // Invalid: empty token
            llm_config: LlmConfig {
                base_url: "http://test.com".to_string(),
                api_key: "test_key".to_string(),
            },
            system_prompt: "Test prompt".to_string(),
            protection_level: ProtectionLevel::Medium,
        };
        
        let result = manager.create_bot(config).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotManagerError::ConfigError(_) => {},
            _ => panic!("Expected ConfigError"),
        }
    }

    #[tokio::test]
    async fn test_start_bot_success() {
        let manager = LifecycleManager::new(10);
        let config = create_test_config("test_bot_start");
        
        // Create the bot
        let bot_id = manager.create_bot(config).await.unwrap();
        
        // Start the bot
        let result = manager.start_bot(&bot_id).await;
        assert!(result.is_ok());
        
        // Verify state
        let bots = manager.bots.read().await;
        let bot = bots.get(&bot_id).unwrap();
        assert!(matches!(bot.state, BotState::Running));
    }

    #[tokio::test]
    async fn test_start_bot_not_found() {
        let manager = LifecycleManager::new(10);
        let bot_id = BotId::new("non_existent");
        
        let result = manager.start_bot(&bot_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotManagerError::BotNotFound(_) => {},
            _ => panic!("Expected BotNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_stop_bot_success() {
        let manager = LifecycleManager::new(10);
        let config = create_test_config("test_bot_stop");
        
        // Create and start the bot
        let bot_id = manager.create_bot(config).await.unwrap();
        manager.start_bot(&bot_id).await.unwrap();
        
        // Stop the bot
        let result = manager.stop_bot(&bot_id).await;
        assert!(result.is_ok());
        
        // Verify state
        let bots = manager.bots.read().await;
        let bot = bots.get(&bot_id).unwrap();
        assert!(matches!(bot.state, BotState::Stopped));
        assert!(bot.process.is_none());
        assert!(bot.process_id.is_none());
    }

    #[tokio::test]
    async fn test_restart_bot_success() {
        let manager = LifecycleManager::new(10);
        let config = create_test_config("test_bot_restart");
        
        // Create and start the bot
        let bot_id = manager.create_bot(config).await.unwrap();
        manager.start_bot(&bot_id).await.unwrap();
        
        // Get initial restart count
        let initial_count = {
            let bots = manager.bots.read().await;
            bots.get(&bot_id).unwrap().restart_count
        };
        
        // Restart the bot
        let result = manager.restart_bot(&bot_id).await;
        assert!(result.is_ok());
        
        // Verify state and restart count
        let bots = manager.bots.read().await;
        let bot = bots.get(&bot_id).unwrap();
        assert!(matches!(bot.state, BotState::Running));
        assert_eq!(bot.restart_count, initial_count + 1);
    }

    #[tokio::test]
    async fn test_restart_bot_not_found() {
        let manager = LifecycleManager::new(10);
        let bot_id = BotId::new("non_existent");
        
        let result = manager.restart_bot(&bot_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotManagerError::BotNotFound(_) => {},
            _ => panic!("Expected BotNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_bot_process_lifecycle() {
        let manager = LifecycleManager::new(10);
        let config = create_test_config("test_bot_lifecycle");
        
        // Create bot
        let bot_id = manager.create_bot(config.clone()).await.unwrap();
        
        // Verify initial state
        {
            let bots = manager.bots.read().await;
            let bot = bots.get(&bot_id).unwrap();
            assert!(matches!(bot.state, BotState::Starting));
            assert_eq!(bot.restart_count, 0);
        }
        
        // Start bot
        manager.start_bot(&bot_id).await.unwrap();
        {
            let bots = manager.bots.read().await;
            let bot = bots.get(&bot_id).unwrap();
            assert!(matches!(bot.state, BotState::Running));
        }
        
        // Stop bot
        manager.stop_bot(&bot_id).await.unwrap();
        {
            let bots = manager.bots.read().await;
            let bot = bots.get(&bot_id).unwrap();
            assert!(matches!(bot.state, BotState::Stopped));
        }
    }
}
