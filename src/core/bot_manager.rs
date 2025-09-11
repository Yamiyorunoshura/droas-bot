use crate::core::types::{
    BotId, BotConfig, BotInstance, BotState, HealthStatus, BotManagerError,
    RestartPolicy
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};
use tracing::{info, warn, error, debug};

/// Maximum number of bots that can be managed
const MAX_BOT_COUNT: usize = 10;

/// Process Supervisor for monitoring bot health
pub struct ProcessSupervisor {
    restart_policy: RestartPolicy,
    manager: Arc<BotManager>,
    last_restart: Arc<RwLock<HashMap<BotId, Instant>>>,
}

impl ProcessSupervisor {
    /// Create a new ProcessSupervisor
pub fn new(manager: Arc<BotManager>) -> Self {
        Self {
            restart_policy: RestartPolicy::default(),
            manager,
            last_restart: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start monitoring bots
    pub async fn start_monitoring(&self) {
        let mut health_check_interval = interval(Duration::from_secs(30));
        
        loop {
            health_check_interval.tick().await;
            self.check_all_bots().await;
        }
    }

    /// Check health of all bots
    async fn check_all_bots(&self) {
        let bot_ids = self.manager.list_bot_ids().await;
        
        for bot_id in bot_ids {
            let health = self.manager.health_check(bot_id.clone()).await;
            
            if let HealthStatus::Unhealthy(reason) = health {
                warn!("Bot {} is unhealthy: {}", bot_id, reason);
                self.handle_unhealthy_bot(bot_id).await;
            }
        }
    }

    /// Handle an unhealthy bot
async fn handle_unhealthy_bot(&self, bot_id: BotId) {
        // Apply restart policy with backoff
        let now = Instant::now();
        {
            let mut last_map = self.last_restart.write().await;
            if let Some(last) = last_map.get(&bot_id).cloned() {
                let min_interval = self.restart_policy.restart_interval;
                if now.duration_since(last) < min_interval {
                    warn!("Skipping restart for {} due to backoff policy", bot_id);
                    return;
                }
            }
            last_map.insert(bot_id.clone(), now);
        }

        info!("Attempting to restart unhealthy bot: {}", bot_id);
        
        if let Err(e) = self.manager.restart_bot(bot_id.clone()).await {
            error!("Failed to restart bot {}: {}", bot_id, e);
        }
    }
}

/// Service Registry for maintaining active bots
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<BotId, BotInstance>>>,
}

impl ServiceRegistry {
    /// Create a new ServiceRegistry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new service
    pub async fn register(&self, instance: BotInstance) -> Result<(), BotManagerError> {
        let mut services = self.services.write().await;
        
        if services.contains_key(&instance.id) {
            return Err(BotManagerError::DuplicateBotId(instance.id.clone()));
        }
        
        services.insert(instance.id.clone(), instance);
        Ok(())
    }

    /// Deregister a service
    pub async fn deregister(&self, bot_id: &BotId) -> Result<BotInstance, BotManagerError> {
        let mut services = self.services.write().await;
        services.remove(bot_id)
            .ok_or_else(|| BotManagerError::BotNotFound(bot_id.clone()))
    }

    /// Get a service by ID
    pub async fn get(&self, bot_id: &BotId) -> Option<BotInstance> {
        let services = self.services.read().await;
        services.get(bot_id).cloned()
    }

    /// List all service IDs
    pub async fn list_ids(&self) -> Vec<BotId> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Get the count of active services
    pub async fn count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }
}

/// Main Bot Manager structure
pub struct BotManager {
    active_bots: Arc<RwLock<HashMap<BotId, BotInstance>>>,
}

impl BotManager {
    /// Create a new BotManager
pub fn new() -> Self {
        Self {
            active_bots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new bot with the given configuration
    pub async fn start_bot(&self, config: BotConfig) -> Result<BotId, BotManagerError> {
        // Validate configuration
        config.validate()?;
        
        // Check capacity
        if self.get_active_bot_count().await >= MAX_BOT_COUNT {
            return Err(BotManagerError::MaxCapacityReached);
        }
        
        // Check for duplicate ID
        let bots = self.active_bots.read().await;
        if bots.contains_key(&config.bot_id) {
            return Err(BotManagerError::DuplicateBotId(config.bot_id.clone()));
        }
        drop(bots);
        
        // Create new bot instance
        let mut instance = BotInstance::new(config);
        let bot_id = instance.id.clone();
        
        // Simulate bot startup
        self.initialize_bot(&mut instance).await?;
        
// Add to active bots
        let mut bots = self.active_bots.write().await;
        bots.insert(bot_id.clone(), instance);
        
        info!("Successfully started bot: {}", bot_id);
        Ok(bot_id)
    }

    /// Stop a running bot
    pub async fn stop_bot(&self, bot_id: BotId) -> Result<(), BotManagerError> {
        // Remove from active bots
        let mut bots = self.active_bots.write().await;
        let mut instance = bots.remove(&bot_id)
            .ok_or_else(|| BotManagerError::BotNotFound(bot_id.clone()))?;
        
        // Update state
        instance.state = BotState::Stopping;
        
// Simulate cleanup
        self.cleanup_bot(&instance).await?;
        
        info!("Successfully stopped bot: {}", bot_id);
        Ok(())
    }

    /// Restart a bot
    pub async fn restart_bot(&self, bot_id: BotId) -> Result<(), BotManagerError> {
        // Get the current configuration
        let config = {
            let bots = self.active_bots.read().await;
            let instance = bots.get(&bot_id)
                .ok_or_else(|| BotManagerError::BotNotFound(bot_id.clone()))?;
            instance.config.clone()
        };
        
        // Stop the bot
        self.stop_bot(bot_id.clone()).await?;
        
        // Wait a moment before restarting
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Start the bot with the same configuration
        self.start_bot(config).await?;
        
        // Update restart count
        let mut bots = self.active_bots.write().await;
        if let Some(instance) = bots.get_mut(&bot_id) {
            instance.increment_restart_count();
        }
        
        info!("Successfully restarted bot: {}", bot_id);
        Ok(())
    }

    /// Check the health of a bot
    pub async fn health_check(&self, bot_id: BotId) -> HealthStatus {
        let mut bots = self.active_bots.write().await;
        
        match bots.get_mut(&bot_id) {
            Some(instance) => {
                instance.update_health_check();
                
                match &instance.state {
                    BotState::Running => HealthStatus::Healthy,
                    BotState::Starting | BotState::Stopping => HealthStatus::Restarting,
                    BotState::Failed(reason) => HealthStatus::Unhealthy(reason.clone()),
                    BotState::Stopped => HealthStatus::NotFound,
                }
            }
            None => HealthStatus::NotFound,
        }
    }

    /// Get the count of active bots
    pub async fn get_active_bot_count(&self) -> usize {
        let bots = self.active_bots.read().await;
        bots.len()
    }

    /// List all bot IDs
    pub async fn list_bot_ids(&self) -> Vec<BotId> {
        let bots = self.active_bots.read().await;
        bots.keys().cloned().collect()
    }

    /// Shutdown all bots gracefully
    pub async fn shutdown_all(&self) -> Result<(), BotManagerError> {
        let bot_ids = self.list_bot_ids().await;
        
        for bot_id in bot_ids {
            if let Err(e) = self.stop_bot(bot_id.clone()).await {
                error!("Failed to stop bot {} during shutdown: {}", bot_id, e);
            }
        }
        
        info!("All bots have been shut down");
        Ok(())
    }

    /// Initialize a bot (private helper)
    async fn initialize_bot(&self, instance: &mut BotInstance) -> Result<(), BotManagerError> {
        // Simulate bot initialization
        debug!("Initializing bot: {}", instance.id);
        
        // In a real implementation, this would:
        // - Connect to Discord
        // - Initialize LLM connection
        // - Set up event handlers
        // - etc.
        
        instance.state = BotState::Running;
        Ok(())
    }

    /// Clean up a bot (private helper)
    async fn cleanup_bot(&self, instance: &BotInstance) -> Result<(), BotManagerError> {
        // Simulate bot cleanup
        debug!("Cleaning up bot: {}", instance.id);
        
        // In a real implementation, this would:
        // - Disconnect from Discord
        // - Close LLM connections
        // - Clean up resources
        // - etc.
        
        Ok(())
    }
}

// Re-export types for convenience
pub use crate::core::types::{BotId as BotIdType, BotConfig as BotConfigType, HealthStatus as HealthStatusType};
