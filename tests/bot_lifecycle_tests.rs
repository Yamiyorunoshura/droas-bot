use droas_bot::bot_manager::lifecycle::LifecycleManager;
use droas_bot::bot_manager::health::{HealthMonitor, HealthStatus};
use droas_bot::bot_manager::restart_policy::{AutoRestartSupervisor, RestartPolicy, RestartReporter};
use droas_bot::core::types::{BotId, BotConfig, BotConfigBuilder, LlmConfig, ProtectionLevel};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

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
async fn test_complete_lifecycle_flow() {
    // Initialize components
    let manager = Arc::new(LifecycleManager::new(10));
    let monitor = Arc::new(HealthMonitor::new(manager.clone()));
    let policy = RestartPolicy::default();
    let supervisor = Arc::new(AutoRestartSupervisor::new(
        manager.clone(),
        monitor.clone(),
        policy,
    ));

    // Create multiple bots
    let bot1_config = create_test_config("lifecycle_bot_1");
    let bot2_config = create_test_config("lifecycle_bot_2");
    
    let bot1_id = manager.create_bot(bot1_config).await.unwrap();
    let bot2_id = manager.create_bot(bot2_config).await.unwrap();
    
    // Start bots
    manager.start_bot(&bot1_id).await.unwrap();
    manager.start_bot(&bot2_id).await.unwrap();
    
    // Check health
    let health1 = monitor.check_bot_health(&bot1_id).await;
    let health2 = monitor.check_bot_health(&bot2_id).await;
    
    assert!(matches!(health1, HealthStatus::Healthy));
    assert!(matches!(health2, HealthStatus::Healthy));
    
    // Get status
    let status1 = monitor.get_bot_status(&bot1_id).await.unwrap();
    let status2 = monitor.get_bot_status(&bot2_id).await.unwrap();
    
    assert_eq!(status1.bot_id, bot1_id);
    assert_eq!(status2.bot_id, bot2_id);
    
    // Stop one bot
    manager.stop_bot(&bot1_id).await.unwrap();
    
    // Check health again
    let health1_after = monitor.check_bot_health(&bot1_id).await;
    assert!(matches!(health1_after, HealthStatus::Unhealthy(_)));
    
    // Restart the stopped bot
    manager.restart_bot(&bot1_id).await.unwrap();
    
    // Verify it's running again
    let health1_restarted = monitor.check_bot_health(&bot1_id).await;
    assert!(matches!(health1_restarted, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_concurrent_bot_operations() {
    let manager = Arc::new(LifecycleManager::new(10));
    
    // Create multiple bots concurrently
    let mut tasks = vec![];
    
    for i in 1..=5 {
        let manager_clone = manager.clone();
        let config = create_test_config(&format!("concurrent_bot_{}", i));
        
        let task = tokio::spawn(async move {
            let bot_id = manager_clone.create_bot(config).await.unwrap();
            manager_clone.start_bot(&bot_id).await.unwrap();
            bot_id
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let bot_ids: Vec<BotId> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Verify all bots were created
    assert_eq!(bot_ids.len(), 5);
    assert_eq!(manager.get_bot_count().await, 5);
    
    // Stop all bots concurrently
    let mut stop_tasks = vec![];
    
    for bot_id in bot_ids {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            manager_clone.stop_bot(&bot_id).await.unwrap();
        });
        stop_tasks.push(task);
    }
    
    futures::future::join_all(stop_tasks).await;
    
    // All bots should still be registered but stopped
    assert_eq!(manager.get_bot_count().await, 5);
}

#[tokio::test]
async fn test_auto_restart_on_failure() {
    // Create components with quick restart policy for testing
    let policy = RestartPolicy {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        reset_window: Duration::from_secs(3600),
    };
    
    let manager = Arc::new(LifecycleManager::new(10));
    let monitor = Arc::new(HealthMonitor::new(manager.clone()));
    let supervisor = Arc::new(AutoRestartSupervisor::new(
        manager.clone(),
        monitor.clone(),
        policy,
    ));
    let reporter = Arc::new(RestartReporter::new());
    
    // Create and start a bot
    let config = create_test_config("auto_restart_bot");
    let bot_id = manager.create_bot(config).await.unwrap();
    manager.start_bot(&bot_id).await.unwrap();
    
    // Simulate failure and trigger restart
    supervisor.handle_unhealthy_bot(&bot_id, "Simulated failure").await;
    
    // Wait for restart to happen
    sleep(Duration::from_millis(200)).await;
    
    // Check restart info
    let restart_info = supervisor.get_restart_info(&bot_id).await;
    assert!(restart_info.is_some());
    let (count, _) = restart_info.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_capacity_limits() {
    let manager = Arc::new(LifecycleManager::new(3)); // Small capacity for testing
    
    // Create bots up to capacity
    for i in 1..=3 {
        let config = create_test_config(&format!("capacity_bot_{}", i));
        let result = manager.create_bot(config).await;
        assert!(result.is_ok());
    }
    
    // Try to create one more beyond capacity
    let config = create_test_config("overflow_bot");
    let result = manager.create_bot(config).await;
    assert!(result.is_err());
    
    // Verify the error is MaxCapacityReached
    match result.unwrap_err() {
        droas_bot::core::types::BotManagerError::MaxCapacityReached => {},
        _ => panic!("Expected MaxCapacityReached error"),
    }
}

#[tokio::test]
async fn test_health_monitoring_integration() {
    let manager = Arc::new(LifecycleManager::new(10));
    let monitor = Arc::new(HealthMonitor::new(manager.clone()));
    
    // Create multiple bots with different states
    let healthy_config = create_test_config("healthy_bot");
    let unhealthy_config = create_test_config("unhealthy_bot");
    
    let healthy_id = manager.create_bot(healthy_config).await.unwrap();
    let unhealthy_id = manager.create_bot(unhealthy_config).await.unwrap();
    
    // Start only the healthy bot
    manager.start_bot(&healthy_id).await.unwrap();
    // unhealthy_bot is created but not started
    
    // Get all statuses
    let all_statuses = monitor.get_all_statuses().await;
    assert_eq!(all_statuses.len(), 2);
    
    // Verify health states
    for status in all_statuses {
        if status.bot_id == healthy_id {
            assert!(matches!(status.health, HealthStatus::Healthy | HealthStatus::Unknown));
        } else if status.bot_id == unhealthy_id {
            assert!(matches!(status.health, HealthStatus::Unhealthy(_) | HealthStatus::Unknown));
        }
    }
}
