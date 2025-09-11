use droas_bot::config::{BotConfig, BotConfigInner, LlmConfig, ConfigService, ConfigError, ValidationEngine};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

// ========== Schema Tests ==========

#[cfg(test)]
mod schema_tests {
    use super::*;
    
    #[test]
    fn test_bot_config_deserialize_valid_yaml() {
        let yaml_content = r#"
bot_config:
  discord_token: "test_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "test_api_key"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "You are a helpful assistant"
  protection_level: "medium"
  enabled: true
"#;
        
        let config: Result<BotConfig, _> = serde_yaml::from_str(yaml_content);
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.bot_config.discord_token, "test_token");
        assert_eq!(config.bot_config.llm_config.base_url, "http://localhost:8080");
        assert_eq!(config.bot_config.protection_level, "medium");
    }
    
    #[test]
    fn test_bot_config_missing_required_fields() {
        let yaml_content = r#"
bot_config:
  discord_token: "test_token"
  # Missing llm_config
  system_prompt: "You are a helpful assistant"
"#;
        
        let config: Result<BotConfig, _> = serde_yaml::from_str(yaml_content);
        assert!(config.is_err());
    }
    
    #[test]
    fn test_environment_variable_injection() {
        std::env::set_var("TEST_BOT_TOKEN", "injected_token");
        std::env::set_var("TEST_API_KEY", "injected_key");
        
        let yaml_content = r#"
bot_config:
  discord_token: "${TEST_BOT_TOKEN}"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "${TEST_API_KEY}"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "Test prompt"
  protection_level: "low"
  enabled: true
"#;
        
        let config = BotConfig::from_yaml_with_env(yaml_content).unwrap();
        assert_eq!(config.bot_config.discord_token, "injected_token");
        assert_eq!(config.bot_config.llm_config.api_key, "injected_key");
        
        // Clean up
        std::env::remove_var("TEST_BOT_TOKEN");
        std::env::remove_var("TEST_API_KEY");
    }
    
    #[test]
    fn test_invalid_protection_level() {
        let yaml_content = r#"
bot_config:
  discord_token: "test_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "test_key"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "Test prompt"
  protection_level: "invalid_level"
  enabled: true
"#;
        
        // Parse should succeed, but validation should fail
        let config = BotConfig::from_yaml_str(yaml_content);
        assert!(config.is_ok());
        
        let config = config.unwrap();
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        if let Err(e) = validation_result {
            assert!(e.to_string().contains("protection_level"));
        }
    }
}

// ========== ConfigService Tests ==========

#[cfg(test)]
mod config_service_tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_config_service_load_valid_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.yaml");
        
        let yaml_content = r#"
bot_config:
  discord_token: "test_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "test_api_key"
    model: "gpt-4"
    max_tokens: 2000
  system_prompt: "You are a helpful assistant"
  protection_level: "medium"
  enabled: true
"#;
        
        fs::write(&file_path, yaml_content).unwrap();
        
        let service = ConfigService::new();
        let result = service.load_config(&file_path).await;
        
        assert!(result.is_ok());
        
        let config = service.get_config().await.unwrap();
        assert_eq!(config.bot_config.discord_token, "test_token");
    }
    
    #[tokio::test]
    async fn test_config_service_load_non_existent_file() {
        let service = ConfigService::new();
        let result = service.load_config(Path::new("/non/existent/path.yaml")).await;
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ConfigError::FileNotFound(_)));
        }
    }
    
    #[tokio::test]
    async fn test_config_service_validate() {
        let service = ConfigService::new();
        
        let valid_config = BotConfig {
            bot_config: BotConfigInner {
                discord_token: "valid_token".to_string(),
                llm_config: LlmConfig {
                    base_url: "http://localhost:8080".to_string(),
                    api_key: "valid_key".to_string(),
                    model: "gpt-4".to_string(),
                    max_tokens: 2000,
                    temperature: 0.7,
                    timeout_seconds: 30,
                },
                system_prompt: "Test prompt".to_string(),
                protection_level: "medium".to_string(),
                enabled: true,
                custom_params: std::collections::HashMap::new(),
            }
        };
        
        let validation_result = service.validate_config(&valid_config).await;
        assert!(validation_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_service_thread_safety() {
        use std::sync::Arc;
        use tokio::task;
        
        let service = Arc::new(ConfigService::new());
        let mut handles = vec![];
        
        // Spawn multiple tasks that read/write config concurrently
        for i in 0..10 {
            let service_clone = Arc::clone(&service);
            let handle = task::spawn(async move {
                let config = BotConfig {
                    bot_config: BotConfigInner {
                        discord_token: format!("token_{}", i),
                        llm_config: LlmConfig {
                            base_url: "http://localhost:8080".to_string(),
                            api_key: format!("key_{}", i),
                            model: "gpt-4".to_string(),
                            max_tokens: 2000,
                            temperature: 0.7,
                            timeout_seconds: 30,
                        },
                        system_prompt: "Test prompt".to_string(),
                        protection_level: "medium".to_string(),
                        enabled: true,
                        custom_params: std::collections::HashMap::new(),
                    }
                };
                
                service_clone.set_config(config).await.unwrap();
                let _ = service_clone.get_config().await;
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}

// ========== ValidationEngine Tests ==========

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[test]
    fn test_validate_discord_token() {
        let engine = ValidationEngine::new();
        
        // Valid token format
        assert!(engine.validate_discord_token("valid_token_123").is_ok());
        
        // Empty token
        assert!(engine.validate_discord_token("").is_err());
        
        // Token with spaces
        assert!(engine.validate_discord_token("invalid token").is_err());
    }
    
    #[test]
    fn test_validate_url() {
        let engine = ValidationEngine::new();
        
        // Valid URLs
        assert!(engine.validate_url("http://localhost:8080").is_ok());
        assert!(engine.validate_url("https://api.openai.com").is_ok());
        
        // Invalid URLs
        assert!(engine.validate_url("not_a_url").is_err());
        assert!(engine.validate_url("").is_err());
    }
    
    #[test]
    fn test_validate_protection_level() {
        let engine = ValidationEngine::new();
        
        // Valid levels
        assert!(engine.validate_protection_level("low").is_ok());
        assert!(engine.validate_protection_level("medium").is_ok());
        assert!(engine.validate_protection_level("high").is_ok());
        
        // Invalid levels
        assert!(engine.validate_protection_level("invalid").is_err());
        assert!(engine.validate_protection_level("").is_err());
    }
    
    #[test]
    fn test_validate_max_tokens() {
        let engine = ValidationEngine::new();
        
        // Valid range
        assert!(engine.validate_max_tokens(100).is_ok());
        assert!(engine.validate_max_tokens(2000).is_ok());
        assert!(engine.validate_max_tokens(8000).is_ok());
        
        // Invalid range
        assert!(engine.validate_max_tokens(0).is_err());
        assert!(engine.validate_max_tokens(-100).is_err());
        assert!(engine.validate_max_tokens(100_000).is_err());
    }
}

// ========== Performance Tests ==========

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_config_load_performance() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("perf_config.yaml");
        
        // Create a large config file
        let mut yaml_content = String::from("bot_config:\n");
        yaml_content.push_str("  discord_token: \"test_token\"\n");
        yaml_content.push_str("  llm_config:\n");
        yaml_content.push_str("    base_url: \"http://localhost:8080\"\n");
        yaml_content.push_str("    api_key: \"test_key\"\n");
        yaml_content.push_str("    model: \"gpt-4\"\n");
        yaml_content.push_str("    max_tokens: 2000\n");
        yaml_content.push_str("  system_prompt: \"");
        
        // Add a large prompt to test performance
        for _ in 0..1000 {
            yaml_content.push_str("This is a test prompt. ");
        }
        yaml_content.push_str("\"\n");
        yaml_content.push_str("  protection_level: \"medium\"\n");
        yaml_content.push_str("  enabled: true\n");
        
        fs::write(&file_path, yaml_content).unwrap();
        
        let service = ConfigService::new();
        
        let start = Instant::now();
        let result = service.load_config(&file_path).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        // Assert load time is less than 100ms
        assert!(duration < Duration::from_millis(100), 
                "Config load took {:?}, expected < 100ms", duration);
    }
}
