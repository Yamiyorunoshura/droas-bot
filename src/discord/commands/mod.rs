//! Discord 斜線命令處理模組
//! 
//! 提供統一的命令註冊、處理和響應框架，支持權限驗證、參數解析和錯誤處理。

pub mod constants;
pub mod framework;
pub mod services;
pub mod set_background;
pub mod preview;
pub mod config;

pub use framework::{
    CommandFramework, CommandHandler, CommandContext, CommandResult,
    CommandError, PermissionLevel,
};

use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

/// 命令註冊器 - 負責將命令註冊到Discord
pub struct CommandRegistry {
    handlers: HashMap<String, Arc<dyn CommandHandler + Send + Sync>>,
}

impl CommandRegistry {
    /// 創建新的命令註冊器
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// 註冊命令處理器
    pub fn register<H>(&mut self, name: &str, handler: H) -> &mut Self
    where
        H: CommandHandler + Send + Sync + 'static,
    {
        self.handlers.insert(name.to_string(), Arc::new(handler));
        self
    }
    
    /// 獲取已註冊的處理器
    pub fn get_handler(&self, name: &str) -> Option<&Arc<dyn CommandHandler + Send + Sync>> {
        self.handlers.get(name)
    }
    
    /// 列出所有已註冊的命令名稱
    pub fn list_commands(&self) -> Vec<&String> {
        self.handlers.keys().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discord::commands::framework::{CommandContext, CommandResult};
    
    struct TestCommandHandler;
    
    #[async_trait]
    impl CommandHandler for TestCommandHandler {
        async fn handle(&self, ctx: CommandContext) -> CommandResult<()> {
            Ok(())
        }
        
        fn name(&self) -> &'static str {
            "test"
        }
        
        fn description(&self) -> &'static str {
            "Test command"
        }
        
        fn required_permissions(&self) -> PermissionLevel {
            PermissionLevel::Everyone
        }
    }
    
    #[test]
    fn test_command_registry() {
        let mut registry = CommandRegistry::new();
        let handler = TestCommandHandler;
        
        registry.register("test", handler);
        
        assert!(registry.get_handler("test").is_some());
        assert!(registry.get_handler("nonexistent").is_none());
        assert_eq!(registry.list_commands().len(), 1);
    }
}