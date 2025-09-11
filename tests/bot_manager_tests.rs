use droas_bot::{BotManager, BotConfig, BotId, HealthStatus, BotManagerError, LlmConfig, ProtectionLevel};
use std::sync::Arc;

#[cfg(test)]
mod bot_manager_tests {
    use super::*;

    // Test helper function to create a default BotConfig
    fn create_test_config(id: &str) -> BotConfig {
        BotConfig {
            bot_id: BotId::new(id),
            discord_token: format!("TEST_TOKEN_{}", id),
            llm_config: LlmConfig {
                base_url: "http://localhost:8080".to_string(),
                api_key: "test_api_key".to_string(),
            },
            system_prompt: "Test bot prompt".to_string(),
            protection_level: ProtectionLevel::Medium,
        }
    }

    #[tokio::test]
    async fn test_create_bot_manager() {
        // Arrange & Act
        let bot_manager = BotManager::new();
        
        // Assert
        assert_eq!(bot_manager.get_active_bot_count().await, 0);
    }

    #[tokio::test]
    async fn test_start_single_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let config = create_test_config("bot1");
        
        // Act
        let result = bot_manager.start_bot(config).await;
        
        // Assert
        assert!(result.is_ok());
        let bot_id = result.unwrap();
        assert_eq!(bot_id.as_str(), "bot1");
        assert_eq!(bot_manager.get_active_bot_count().await, 1);
    }

    #[tokio::test]
    async fn test_start_multiple_bots() {
        // Arrange
        let bot_manager = BotManager::new();
        let mut bot_ids = Vec::new();
        
        // Act - Start 5 bots
        for i in 1..=5 {
            let config = create_test_config(&format!("bot{}", i));
            let result = bot_manager.start_bot(config).await;
            assert!(result.is_ok());
            bot_ids.push(result.unwrap());
        }
        
        // Assert
        assert_eq!(bot_manager.get_active_bot_count().await, 5);
        for (i, bot_id) in bot_ids.iter().enumerate() {
            assert_eq!(bot_id.as_str(), format!("bot{}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_start_maximum_bots() {
        // Arrange
        let bot_manager = BotManager::new();
        
        // Act - Start 10 bots (maximum)
        for i in 1..=10 {
            let config = create_test_config(&format!("bot{}", i));
            let result = bot_manager.start_bot(config).await;
            assert!(result.is_ok());
        }
        
        // Assert
        assert_eq!(bot_manager.get_active_bot_count().await, 10);
        
        // Try to start 11th bot - should fail
        let config = create_test_config("bot11");
        let result = bot_manager.start_bot(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BotManagerError::MaxCapacityReached));
    }

    #[tokio::test]
    async fn test_stop_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let config = create_test_config("bot1");
        let bot_id = bot_manager.start_bot(config).await.unwrap();
        
        // Act
        let result = bot_manager.stop_bot(bot_id.clone()).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(bot_manager.get_active_bot_count().await, 0);
    }

    #[tokio::test]
    async fn test_stop_nonexistent_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let bot_id = BotId::new("nonexistent");
        
        // Act
        let result = bot_manager.stop_bot(bot_id).await;
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BotManagerError::BotNotFound(_)));
    }

    #[tokio::test]
    async fn test_restart_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let config = create_test_config("bot1");
        let bot_id = bot_manager.start_bot(config).await.unwrap();
        
        // Act
        let result = bot_manager.restart_bot(bot_id.clone()).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(bot_manager.get_active_bot_count().await, 1);
        
        // Verify the bot is actually new instance
        let health = bot_manager.health_check(bot_id).await;
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_healthy_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let config = create_test_config("bot1");
        let bot_id = bot_manager.start_bot(config).await.unwrap();
        
        // Act
        let health = bot_manager.health_check(bot_id).await;
        
        // Assert
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_nonexistent_bot() {
        // Arrange
        let bot_manager = BotManager::new();
        let bot_id = BotId::new("nonexistent");
        
        // Act
        let health = bot_manager.health_check(bot_id).await;
        
        // Assert
        assert_eq!(health, HealthStatus::NotFound);
    }

    #[tokio::test]
    async fn test_concurrent_bot_operations() {
        // Arrange
        let bot_manager = Arc::new(BotManager::new());
        let mut handles = vec![];
        
        // Act - Start 5 bots concurrently
        for i in 1..=5 {
            let manager = bot_manager.clone();
            let handle = tokio::spawn(async move {
                let config = create_test_config(&format!("bot{}", i));
                manager.start_bot(config).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // Assert
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
        assert_eq!(bot_manager.get_active_bot_count().await, 5);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        // Arrange
        let bot_manager = BotManager::new();
        
        // Start multiple bots
        for i in 1..=3 {
            let config = create_test_config(&format!("bot{}", i));
            bot_manager.start_bot(config).await.unwrap();
        }
        
        // Act
        let result = bot_manager.shutdown_all().await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(bot_manager.get_active_bot_count().await, 0);
    }

    #[tokio::test]
    async fn test_duplicate_bot_id() {
        // Arrange
        let bot_manager = BotManager::new();
        let config1 = create_test_config("bot1");
        let config2 = create_test_config("bot1"); // Same ID
        
        // Act
        let result1 = bot_manager.start_bot(config1).await;
        let result2 = bot_manager.start_bot(config2).await;
        
        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), BotManagerError::DuplicateBotId(_)));
    }

    #[tokio::test]
    async fn test_invalid_config() {
        // Arrange
        let bot_manager = BotManager::new();
        let mut config = create_test_config("bot1");
        config.discord_token = "".to_string(); // Invalid token
        
        // Act
        let result = bot_manager.start_bot(config).await;
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BotManagerError::ConfigError(_)));
    }
}
