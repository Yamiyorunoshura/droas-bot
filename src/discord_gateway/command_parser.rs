use crate::error::{DiscordError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use serenity::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Balance,
    Transfer,
    History,
    Help,
    AdjustBalance,
    AdminHistory,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: Command,
    pub args: Vec<String>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub guild_id: Option<i64>,
    pub discord_context: Option<Arc<Context>>,
}

pub struct CommandParser {
    command_prefix: String,
    command_handlers: HashMap<String, Command>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self::with_prefix("!".to_string())
    }

    pub fn with_prefix(prefix: String) -> Self {
        let mut command_handlers = HashMap::new();
        command_handlers.insert("balance".to_string(), Command::Balance);
        command_handlers.insert("transfer".to_string(), Command::Transfer);
        command_handlers.insert("history".to_string(), Command::History);
        command_handlers.insert("help".to_string(), Command::Help);
        command_handlers.insert("adjust_balance".to_string(), Command::AdjustBalance);
        command_handlers.insert("admin_history".to_string(), Command::AdminHistory);

        Self {
            command_prefix: prefix,
            command_handlers,
        }
    }

    pub async fn parse_command(&self, input: &str) -> Result<CommandResult> {
        if input.is_empty() {
            return Err(DiscordError::InvalidCommand("Empty command".to_string()));
        }

        if !input.starts_with(&self.command_prefix) {
            return Err(DiscordError::InvalidCommand(format!("Command must start with {}", self.command_prefix)));
        }

        let trimmed = input.trim_start_matches(&self.command_prefix).trim();
        if trimmed.is_empty() {
            return Err(DiscordError::InvalidCommand("Empty command after prefix".to_string()));
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err(DiscordError::InvalidCommand("No command found".to_string()));
        }

        let command_str = parts[0];
        let command = self.command_handlers.get(command_str)
            .ok_or_else(|| DiscordError::UnknownCommand(command_str.to_string()))?
            .clone();

        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        // 注意：在實際實作中，user_id 和 username 會從 Discord 訊息中獲取
        // 這裡暫時設為 None，在實際整合時需要從 Discord 事件中提取
        Ok(CommandResult {
            command,
            args,
            user_id: None,  // 需要從 Discord 訊息中獲取
            username: None, // 需要從 Discord 訊息中獲取
            guild_id: None, // 需要從 Discord 訊息中獲取
            discord_context: None, // 需要從 Discord 訊息中獲取
        })
    }

    pub fn get_available_commands(&self) -> Vec<String> {
        self.command_handlers.keys().cloned().collect()
    }

    pub fn is_command_supported(&self, command: &str) -> bool {
        self.command_handlers.contains_key(command)
    }

    pub fn get_prefix(&self) -> &str {
        &self.command_prefix
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}