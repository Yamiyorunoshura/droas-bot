use crate::core::types::BotId;
use crate::bot_manager::lifecycle::LifecycleManager;
use crate::bot_manager::health::{HealthMonitor, HealthStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};

/// Restart policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    /// Maximum number of restart attempts
    pub max_attempts: u32,
    /// Initial delay between restarts
    pub initial_delay: Duration,
    /// Maximum delay between restarts
    pub max_delay: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Time window for resetting restart count
    pub reset_window: Duration,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes
            backoff_multiplier: 2.0,
            reset_window: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Restart attempt tracking
#[derive(Debug, Clone)]
struct RestartAttempt {
    count: u32,
    last_attempt: Instant,
    next_delay: Duration,
}

/// Auto-restart supervisor with exponential backoff
pub struct AutoRestartSupervisor {
    manager: Arc<LifecycleManager>,
    monitor: Arc<HealthMonitor>,
    policy: RestartPolicy,
    restart_attempts: Arc<RwLock<HashMap<BotId, RestartAttempt>>>,
}

impl AutoRestartSupervisor {
    /// Create a new auto-restart supervisor
    pub fn new(
        manager: Arc<LifecycleManager>,
        monitor: Arc<HealthMonitor>,
        policy: RestartPolicy,
    ) -> Self {
        Self {
            manager,
            monitor,
            policy,
            restart_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle an unhealthy bot with auto-restart
    pub async fn handle_unhealthy_bot(&self, bot_id: &BotId, failure_reason: &str) {
        let mut attempts = self.restart_attempts.write().await;
        
        let now = Instant::now();
        let attempt = attempts.entry(bot_id.clone()).or_insert_with(|| {
            RestartAttempt {
                count: 0,
                last_attempt: now,
                next_delay: self.policy.initial_delay,
            }
        });

        // Check if we should reset the counter (outside the reset window)
        if now.duration_since(attempt.last_attempt) > self.policy.reset_window {
            info!("Resetting restart counter for bot {} (outside reset window)", bot_id);
            attempt.count = 0;
            attempt.next_delay = self.policy.initial_delay;
        }

        // Check if we've exceeded max attempts
        if attempt.count >= self.policy.max_attempts {
            error!(
                "Bot {} has exceeded maximum restart attempts ({}/{}). Manual intervention required.",
                bot_id, attempt.count, self.policy.max_attempts
            );
            return;
        }

        // Calculate next delay with exponential backoff
        let delay = attempt.next_delay;
        attempt.count += 1;
        attempt.last_attempt = now;
        attempt.next_delay = self.calculate_next_delay(attempt.next_delay);

        info!(
            "Scheduling restart for bot {} (attempt {}/{}) after {:?} delay. Reason: {}",
            bot_id, attempt.count, self.policy.max_attempts, delay, failure_reason
        );

        // Clone values we need for the async block
        let bot_id = bot_id.clone();
        let manager = self.manager.clone();
        
        // Schedule the restart after delay
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            
            match manager.restart_bot(&bot_id).await {
                Ok(_) => {
                    info!("Successfully restarted bot {}", bot_id);
                }
                Err(e) => {
                    error!("Failed to restart bot {}: {}", bot_id, e);
                }
            }
        });
    }

    /// Calculate next delay with exponential backoff
    fn calculate_next_delay(&self, current_delay: Duration) -> Duration {
        let next = current_delay.mul_f64(self.policy.backoff_multiplier);
        if next > self.policy.max_delay {
            self.policy.max_delay
        } else {
            next
        }
    }

    /// Get restart attempt information for a bot
    pub async fn get_restart_info(&self, bot_id: &BotId) -> Option<(u32, Duration)> {
        let attempts = self.restart_attempts.read().await;
        attempts.get(bot_id).map(|a| (a.count, a.next_delay))
    }

    /// Clear restart attempts for a bot
    pub async fn clear_restart_attempts(&self, bot_id: &BotId) {
        let mut attempts = self.restart_attempts.write().await;
        attempts.remove(bot_id);
    }

    /// Monitor and auto-restart unhealthy bots
    pub async fn start_monitoring(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                self.check_and_restart_bots().await;
            }
        });
    }

    /// Check all bots and restart unhealthy ones
    async fn check_and_restart_bots(&self) {
        let bot_ids = self.manager.list_bot_ids().await;
        
        for bot_id in bot_ids {
            let health = self.monitor.check_bot_health(&bot_id).await;
            
            if let HealthStatus::Unhealthy(reason) = health {
                self.handle_unhealthy_bot(&bot_id, &reason).await;
            }
        }
    }
}

/// Restart event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartEvent {
    pub bot_id: BotId,
    pub timestamp: String,
    pub reason: String,
    pub attempt_number: u32,
    pub next_delay_seconds: u64,
}

/// Restart event reporter
pub struct RestartReporter {
    events: Arc<RwLock<Vec<RestartEvent>>>,
}

impl RestartReporter {
    /// Create a new restart reporter
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Report a restart event
    pub async fn report_restart(
        &self,
        bot_id: BotId,
        reason: String,
        attempt_number: u32,
        next_delay: Duration,
    ) {
        let event = RestartEvent {
            bot_id: bot_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            reason,
            attempt_number,
            next_delay_seconds: next_delay.as_secs(),
        };

        let mut events = self.events.write().await;
        events.push(event.clone());
        
        // Log the event
        info!(
            "Restart event reported - Bot: {}, Attempt: {}, Reason: {}",
            bot_id, attempt_number, event.reason
        );
        
        // TODO: Send notification to parent bot
        // TODO: Send metrics to monitoring system
    }

    /// Get all restart events
    pub async fn get_events(&self) -> Vec<RestartEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Get events for a specific bot
    pub async fn get_bot_events(&self, bot_id: &BotId) -> Vec<RestartEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.bot_id == *bot_id)
            .cloned()
            .collect()
    }

    /// Clear all events
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
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
    async fn test_exponential_backoff() {
        let policy = RestartPolicy {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            reset_window: Duration::from_secs(3600),
        };

        let manager = Arc::new(LifecycleManager::new(10));
        let monitor = Arc::new(HealthMonitor::new(manager.clone()));
        let supervisor = AutoRestartSupervisor::new(manager, monitor, policy);

        // Test backoff calculation
        let delay1 = supervisor.calculate_next_delay(Duration::from_secs(1));
        assert_eq!(delay1, Duration::from_secs(2));

        let delay2 = supervisor.calculate_next_delay(delay1);
        assert_eq!(delay2, Duration::from_secs(4));

        let delay3 = supervisor.calculate_next_delay(Duration::from_secs(8));
        assert_eq!(delay3, Duration::from_secs(10)); // Capped at max_delay
    }

    #[tokio::test]
    async fn test_restart_attempt_tracking() {
        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_restart_tracking");
        let bot_id = manager.create_bot(config).await.unwrap();
        
        let monitor = Arc::new(HealthMonitor::new(manager.clone()));
        let supervisor = AutoRestartSupervisor::new(
            manager,
            monitor,
            RestartPolicy::default(),
        );

        // Initially no restart info
        assert!(supervisor.get_restart_info(&bot_id).await.is_none());

        // Simulate unhealthy bot
        supervisor.handle_unhealthy_bot(&bot_id, "Test failure").await;

        // Should have restart info now
        let info = supervisor.get_restart_info(&bot_id).await;
        assert!(info.is_some());
        let (count, _) = info.unwrap();
        assert_eq!(count, 1);

        // Clear attempts
        supervisor.clear_restart_attempts(&bot_id).await;
        assert!(supervisor.get_restart_info(&bot_id).await.is_none());
    }

    #[tokio::test]
    async fn test_max_restart_attempts() {
        let policy = RestartPolicy {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            reset_window: Duration::from_secs(3600),
        };

        let manager = Arc::new(LifecycleManager::new(10));
        let config = create_test_config("test_max_attempts");
        let bot_id = manager.create_bot(config).await.unwrap();
        
        let monitor = Arc::new(HealthMonitor::new(manager.clone()));
        let supervisor = AutoRestartSupervisor::new(manager, monitor, policy);

        // First attempt
        supervisor.handle_unhealthy_bot(&bot_id, "Failure 1").await;
        let info = supervisor.get_restart_info(&bot_id).await.unwrap();
        assert_eq!(info.0, 1);

        // Second attempt
        supervisor.handle_unhealthy_bot(&bot_id, "Failure 2").await;
        let info = supervisor.get_restart_info(&bot_id).await.unwrap();
        assert_eq!(info.0, 2);

        // Third attempt should be blocked (exceeds max_attempts)
        supervisor.handle_unhealthy_bot(&bot_id, "Failure 3").await;
        let info = supervisor.get_restart_info(&bot_id).await.unwrap();
        assert_eq!(info.0, 2); // Count should not increase
    }

    #[tokio::test]
    async fn test_restart_reporter() {
        let reporter = RestartReporter::new();
        let bot_id = BotId::new("test_reporter");

        // Report some events
        reporter.report_restart(
            bot_id.clone(),
            "Test failure".to_string(),
            1,
            Duration::from_secs(1),
        ).await;

        reporter.report_restart(
            bot_id.clone(),
            "Another failure".to_string(),
            2,
            Duration::from_secs(2),
        ).await;

        // Check events
        let all_events = reporter.get_events().await;
        assert_eq!(all_events.len(), 2);

        let bot_events = reporter.get_bot_events(&bot_id).await;
        assert_eq!(bot_events.len(), 2);
        assert_eq!(bot_events[0].attempt_number, 1);
        assert_eq!(bot_events[1].attempt_number, 2);

        // Clear events
        reporter.clear_events().await;
        let events = reporter.get_events().await;
        assert_eq!(events.len(), 0);
    }
}
