use droas_bot::config::{
    FileWatcher, EventBus, HotReloadService, ConfigEvent, ConfigService
};
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use std::fs;

// ========== FileWatcher Tests ==========

#[cfg(test)]
mod file_watcher_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_file_watcher_detect_changes() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("watch_config.yaml");
        
        // Create initial file
        let initial_content = r#"
bot_config:
  discord_token: "initial_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "initial_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "Initial prompt"
  protection_level: "low"
  enabled: true
"#;
        fs::write(&file_path, initial_content).unwrap();
        
        // Create watcher
        let watcher = FileWatcher::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        
        // Start watching
        watcher.watch(&file_path, tx.clone()).await.unwrap();
        
        // Modify file
        let updated_content = r#"
bot_config:
  discord_token: "updated_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "updated_key"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "Updated prompt"
  protection_level: "medium"
  enabled: true
"#;
        
        // Wait a bit to ensure watcher is ready
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&file_path, updated_content).unwrap();
        
        // Wait for change event
        let result = timeout(Duration::from_secs(5), rx.recv()).await;
        assert!(result.is_ok());
        
        let event = result.unwrap().unwrap();
        assert_eq!(event, file_path);
    }
    
    #[tokio::test]
    async fn test_file_watcher_multiple_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("config1.yaml");
        let file2 = dir.path().join("config2.yaml");
        
        // Create files
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        
        let watcher = FileWatcher::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        
        // Watch both files
        watcher.watch(&file1, tx.clone()).await.unwrap();
        watcher.watch(&file2, tx.clone()).await.unwrap();
        
        // Wait for watcher setup
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Modify both files
        fs::write(&file1, "updated1").unwrap();
        fs::write(&file2, "updated2").unwrap();
        
        // Should receive two events
        let mut events = Vec::new();
        for _ in 0..2 {
            let result = timeout(Duration::from_secs(5), rx.recv()).await;
            if let Ok(Some(path)) = result {
                events.push(path);
            }
        }
        
        assert_eq!(events.len(), 2);
    }
    
    #[tokio::test]
    async fn test_file_watcher_stop() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("stop_test.yaml");
        fs::write(&file_path, "content").unwrap();
        
        let watcher = FileWatcher::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        
        // Start watching
        let watch_handle = watcher.watch(&file_path, tx.clone()).await.unwrap();
        
        // Stop watching
        watcher.stop_watch(watch_handle).await.unwrap();
        
        // Modify file after stopping
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&file_path, "modified").unwrap();
        
        // Should not receive event
        let result = timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(result.is_err()); // Timeout expected
    }
}

// ========== EventBus Tests ==========

#[cfg(test)]
mod event_bus_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_bus_single_subscriber() {
        let event_bus = EventBus::<ConfigEvent>::new(100);
        let mut subscriber = event_bus.subscribe().await;
        
        let event = ConfigEvent::ConfigUpdated {
            path: "/test/config.yaml".to_string(),
            success: true,
        };
        
        event_bus.publish(event.clone()).await.unwrap();
        
        let received = timeout(Duration::from_secs(1), subscriber.recv()).await;
        assert!(received.is_ok());
        
        let received_event = received.unwrap().unwrap();
        assert_eq!(received_event, event);
    }
    
    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let event_bus = EventBus::<ConfigEvent>::new(100);
        
        // Create multiple subscribers
        let mut sub1 = event_bus.subscribe().await;
        let mut sub2 = event_bus.subscribe().await;
        let mut sub3 = event_bus.subscribe().await;
        
        let event = ConfigEvent::ConfigValidationFailed {
            path: "/test/config.yaml".to_string(),
            error: "Invalid format".to_string(),
        };
        
        event_bus.publish(event.clone()).await.unwrap();
        
        // All subscribers should receive the event
        let r1 = timeout(Duration::from_secs(1), sub1.recv()).await;
        let r2 = timeout(Duration::from_secs(1), sub2.recv()).await;
        let r3 = timeout(Duration::from_secs(1), sub3.recv()).await;
        
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
        
        assert_eq!(r1.unwrap().unwrap(), event);
        assert_eq!(r2.unwrap().unwrap(), event);
        assert_eq!(r3.unwrap().unwrap(), event);
    }
    
    #[tokio::test]
    async fn test_event_bus_subscriber_count() {
        let event_bus = EventBus::<ConfigEvent>::new(100);
        
        assert_eq!(event_bus.subscriber_count().await, 0);
        
        let _sub1 = event_bus.subscribe().await;
        assert_eq!(event_bus.subscriber_count().await, 1);
        
        let _sub2 = event_bus.subscribe().await;
        assert_eq!(event_bus.subscriber_count().await, 2);
        
        // Dropping subscriber should decrease count
        drop(_sub1);
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(event_bus.subscriber_count().await, 1);
    }
}

// ========== HotReload Integration Tests ==========

#[cfg(test)]
mod hot_reload_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hot_reload_config_update() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("hot_reload.yaml");
        
        // Create initial config
        let initial_config = r#"
bot_config:
  discord_token: "initial_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "initial_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "Initial prompt"
  protection_level: "low"
  enabled: true
"#;
        fs::write(&config_path, initial_config).unwrap();
        
        // Create hot reload service
        let hot_reload = HotReloadService::new();
        let config_service = Arc::new(ConfigService::new());
        
        // Load initial config
        config_service.load_config(&config_path).await.unwrap();
        
        // Start hot reload
        hot_reload.start(&config_path, config_service.clone()).await.unwrap();
        
        // Subscribe to config events
        let mut event_subscriber = hot_reload.subscribe_events().await;
        
        // Update config file
        let updated_config = r#"
bot_config:
  discord_token: "updated_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "updated_key"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "Updated prompt"
  protection_level: "high"
  enabled: true
"#;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&config_path, updated_config).unwrap();
        
        // Wait for reload event
        let result = timeout(Duration::from_secs(10), event_subscriber.recv()).await;
        assert!(result.is_ok());
        
        let event = result.unwrap().unwrap();
        if let ConfigEvent::ConfigUpdated { success, .. } = event {
            assert!(success);
        } else {
            panic!("Expected ConfigUpdated event");
        }
        
        // Verify config was updated
        let config = config_service.get_config().await.unwrap();
        assert_eq!(config.bot_config.discord_token, "updated_token");
        assert_eq!(config.bot_config.protection_level, "high");
    }
    
    #[tokio::test]
    async fn test_hot_reload_validation_failure_rollback() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rollback_test.yaml");
        
        // Create valid initial config
        let valid_config = r#"
bot_config:
  discord_token: "valid_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "valid_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "Valid prompt"
  protection_level: "medium"
  enabled: true
"#;
        fs::write(&config_path, valid_config).unwrap();
        
        let hot_reload = HotReloadService::new();
        let config_service = Arc::new(ConfigService::new());
        
        // Load initial config
        config_service.load_config(&config_path).await.unwrap();
        let initial_token = config_service.get_config().await.unwrap()
            .bot_config.discord_token.clone();
        
        // Start hot reload
        hot_reload.start(&config_path, config_service.clone()).await.unwrap();
        
        let mut event_subscriber = hot_reload.subscribe_events().await;
        
        // Write invalid config (invalid protection_level after validation)
        let invalid_config = r#"
bot_config:
  discord_token: "new_token"
  llm_config:
    base_url: "invalid_url"
    api_key: "new_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "New prompt"
  protection_level: "medium"
  enabled: true
"#;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&config_path, invalid_config).unwrap();
        
        // Wait for validation failure event
        let result = timeout(Duration::from_secs(10), event_subscriber.recv()).await;
        assert!(result.is_ok());
        
        let event = result.unwrap().unwrap();
        if let ConfigEvent::ConfigValidationFailed { error, .. } = event {
            assert!(error.contains("Invalid"));
        } else {
            panic!("Expected ConfigValidationFailed event");
        }
        
        // Verify config was rolled back
        let config = config_service.get_config().await.unwrap();
        assert_eq!(config.bot_config.discord_token, initial_token);
    }
    
    #[tokio::test]
    async fn test_hot_reload_performance() {
        use std::time::Instant;
        
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("perf_test.yaml");
        
        let config = r#"
bot_config:
  discord_token: "perf_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "perf_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "Performance test"
  protection_level: "low"
  enabled: true
"#;
        fs::write(&config_path, config).unwrap();
        
        let hot_reload = HotReloadService::new();
        let config_service = Arc::new(ConfigService::new());
        
        config_service.load_config(&config_path).await.unwrap();
        hot_reload.start(&config_path, config_service.clone()).await.unwrap();
        
        let mut event_subscriber = hot_reload.subscribe_events().await;
        
        // Measure reload time
        let updated_config = config.replace("perf_token", "updated_perf_token");
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let start = Instant::now();
        fs::write(&config_path, updated_config).unwrap();
        
        // Wait for reload completion
        let result = timeout(Duration::from_secs(15), event_subscriber.recv()).await;
        let reload_time = start.elapsed();
        
        assert!(result.is_ok());
        
        // Verify reload time is within SLA (< 10 seconds)
        assert!(reload_time < Duration::from_secs(10), 
                "Reload took {:?}, expected < 10s", reload_time);
    }
}
