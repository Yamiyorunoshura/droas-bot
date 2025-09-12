use crate::core::types::{BotId, BotManagerError};
use crate::bot_manager::lifecycle::{LifecycleManager, BotProcess};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, interval};
use tracing::{info, warn, error, debug};

/// Health status for bots with detailed information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Bot is operating normally
    Healthy,
    /// Bot is experiencing issues but still functioning
    Degraded(String),
    /// Bot is not functioning properly
    Unhealthy(String),
    /// Health status is unknown (no recent health check)
    Unknown,
}

/// Detailed bot status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    pub bot_id: BotId,
    pub health: HealthStatus,
    pub state: String,
    pub uptime_seconds: u64,
    pub restart_count: u32,
    pub message_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub last_health_check: Option<String>,
    pub process_id: Option<u32>,
    pub config_version: String,
}

/// Health monitoring service
pub struct HealthMonitor {
    manager: Arc<LifecycleManager>,
    health_cache: Arc<RwLock<HashMap<BotId, HealthStatus>>>,
    check_interval: Duration,
    unhealthy_threshold: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(manager: Arc<LifecycleManager>) -> Self {
        Self {
            manager,
            health_cache: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(30),
            unhealthy_threshold: Duration::from_secs(60),
        }
    }

    /// Check the health of a specific bot
    pub async fn check_bot_health(&self, bot_id: &BotId) -> HealthStatus {
        let mut bots = self.manager.bots.write().await;
        
        match bots.get_mut(bot_id) {
            Some(bot) => {
                // Update health check timestamp
                bot.update_health_check();
                
                // Check if process is alive
                let is_alive = bot.is_alive().await;
                
                let health = if !is_alive {
                    HealthStatus::Unhealthy("Process not running".to_string())
                } else if bot.error_count > 10 {
                    HealthStatus::Degraded(format!("High error count: {}", bot.error_count))
                } else if let Some(last_check) = bot.last_health_check {
                    if last_check.elapsed() > self.unhealthy_threshold {
                        HealthStatus::Unknown
                    } else {
                        HealthStatus::Healthy
                    }
                } else {
                    HealthStatus::Unknown
                };
                
                // Update cache
                let mut cache = self.health_cache.write().await;
                cache.insert(bot_id.clone(), health.clone());
                
                health
            }
            None => HealthStatus::Unhealthy("Bot not found".to_string()),
        }
    }

    /// Get detailed status for a bot
    pub async fn get_bot_status(&self, bot_id: &BotId) -> Result<BotStatus, BotManagerError> {
        let bots = self.manager.bots.read().await;
        
        match bots.get(bot_id) {
            Some(bot) => {
                let health = self.get_cached_health(bot_id).await
                    .unwrap_or(HealthStatus::Unknown);
                
                Ok(BotStatus {
                    bot_id: bot_id.clone(),
                    health,
                    state: format!("{:?}", bot.state),
                    uptime_seconds: bot.uptime().as_secs(),
                    restart_count: bot.restart_count,
                    message_count: bot.message_count,
                    error_count: bot.error_count,
                    last_error: bot.last_error.clone(),
                    last_health_check: bot.last_health_check
                        .map(|t| format!("{:.2}s ago", t.elapsed().as_secs_f64())),
                    process_id: bot.process_id,
                    config_version: "1.0.0".to_string(), // TODO: Get from actual config
                })
            }
            None => Err(BotManagerError::BotNotFound(bot_id.clone())),
        }
    }

    /// Get all bot statuses
    pub async fn get_all_statuses(&self) -> Vec<BotStatus> {
        let bot_ids = self.manager.list_bot_ids().await;
        let mut statuses = Vec::new();
        
        for bot_id in bot_ids {
            if let Ok(status) = self.get_bot_status(&bot_id).await {
                statuses.push(status);
            }
        }
        
        statuses
    }

    /// Start periodic health monitoring
    pub async fn start_monitoring(self: Arc<Self>) {
        let mut check_interval = interval(self.check_interval);
        
        tokio::spawn(async move {
            loop {
                check_interval.tick().await;
                self.check_all_bots().await;
            }
        });
    }

    /// Check health of all bots
    async fn check_all_bots(&self) {
        let bot_ids = self.manager.list_bot_ids().await;
        
        for bot_id in bot_ids {
            let health = self.check_bot_health(&bot_id).await;
            
            match health {
                HealthStatus::Unhealthy(reason) => {
                    warn!("Bot {} is unhealthy: {}", bot_id, reason);
                    // TODO: Trigger auto-restart if configured
                }
                HealthStatus::Degraded(reason) => {
                    warn!("Bot {} is degraded: {}", bot_id, reason);
                }
                HealthStatus::Unknown => {
                    debug!("Bot {} health is unknown", bot_id);
                }
                HealthStatus::Healthy => {
                    debug!("Bot {} is healthy", bot_id);
                }
            }
        }
    }

    /// Get cached health status
    async fn get_cached_health(&self, bot_id: &BotId) -> Option<HealthStatus> {
        let cache = self.health_cache.read().await;
        cache.get(bot_id).cloned()
    }

    /// Clear health cache
    pub async fn clear_cache(&self) {
        let mut cache = self.health_cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{BotConfig, LlmConfig, ProtectionLevel, BotConfigBuilder};

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
    async fn test_health_check_healthy_bot() {
        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_health_1");
        
        let bot_id = manager.create_bot(config).await.unwrap();
        manager.start_bot(&bot_id).await.unwrap();
        
        let monitor = HealthMonitor::new(manager.clone());
        let health = monitor.check_bot_health(&bot_id).await;
        
        assert!(matches!(health, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_health_check_stopped_bot() {
        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_health_2");
        
        let bot_id = manager.create_bot(config).await.unwrap();
        // Bot created but not started
        
        let monitor = HealthMonitor::new(manager.clone());
        let health = monitor.check_bot_health(&bot_id).await;
        
        // Bot process is not running, should be unhealthy
        assert!(matches!(health, HealthStatus::Unhealthy(_)));
    }

    #[tokio::test]
    async fn test_get_bot_status() {
        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_status_1");
        
        let bot_id = manager.create_bot(config).await.unwrap();
        manager.start_bot(&bot_id).await.unwrap();
        
        let monitor = HealthMonitor::new(manager.clone());
        let status = monitor.get_bot_status(&bot_id).await.unwrap();
        
        assert_eq!(status.bot_id, bot_id);
        assert_eq!(status.restart_count, 0);
        assert_eq!(status.message_count, 0);
        assert_eq!(status.error_count, 0);
        assert!(status.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_get_bot_status_not_found() {
        let manager = Arc::new(LifecycleManager::new(10));
        let monitor = HealthMonitor::new(manager.clone());
        
        let bot_id = BotId::new("non_existent");
        let result = monitor.get_bot_status(&bot_id).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            BotManagerError::BotNotFound(_) => {},
            _ => panic!("Expected BotNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_all_statuses() {
        let manager = Arc::new(LifecycleManager::new(10));
        
        // Create multiple bots
        for i in 1..=3 {
            let config = create_test_config(&format!("test_all_{}", i));
            let bot_id = manager.create_bot(config).await.unwrap();
            manager.start_bot(&bot_id).await.unwrap();
        }
        
        let monitor = HealthMonitor::new(manager.clone());
        let statuses = monitor.get_all_statuses().await;
        
        assert_eq!(statuses.len(), 3);
        for status in statuses {
            assert!(status.bot_id.as_str().starts_with("test_all_"));
        }
    }

    #[tokio::test]
    async fn test_health_cache() {
        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_cache_1");
        
        let bot_id = manager.create_bot(config).await.unwrap();
        manager.start_bot(&bot_id).await.unwrap();
        
        let monitor = HealthMonitor::new(manager.clone());
        
        // Check health (should update cache)
        monitor.check_bot_health(&bot_id).await;
        
        // Verify cache was updated
        let cached = monitor.get_cached_health(&bot_id).await;
        assert!(cached.is_some());
        
        // Clear cache
        monitor.clear_cache().await;
        
        // Verify cache is empty
        let cached = monitor.get_cached_health(&bot_id).await;
        assert!(cached.is_none());
    }
}
