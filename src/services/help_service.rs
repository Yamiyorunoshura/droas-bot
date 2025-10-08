/// Help Service
/// 提供指令幫助系統功能

use std::collections::HashMap;
use tracing::{info, warn, debug, instrument};
use crate::error::{DiscordError, Result};

/// 指令資訊結構體
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// 指令名稱
    pub name: String,
    /// 指令描述
    pub description: String,
    /// 使用語法
    pub usage: String,
    /// 使用範例
    pub example: String,
    /// 指令分類
    pub category: CommandCategory,
}

/// 指令分類
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    /// 帳戶管理
    Account,
    /// 交易功能
    Transaction,
    /// 查詢功能
    Query,
    /// 系統幫助
    System,
    /// 管理員功能
    Admin,
}

/// Help Service 結構體
/// 負責管理和生成指令幫助內容
pub struct HelpService {
    /// 可用指令映射
    commands: HashMap<String, CommandInfo>,
}

impl HelpService {
    /// 創建新的 HelpService 實例
    pub fn new() -> Self {
        let mut service = Self {
            commands: HashMap::new(),
        };

        // 初始化基本指令
        service.initialize_commands();
        service
    }

    /// 初始化所有可用指令
    fn initialize_commands(&mut self) {
        // 帳戶管理指令
        self.register_command(CommandInfo {
            name: "balance".to_string(),
            description: "查詢帳戶餘額".to_string(),
            usage: "!balance".to_string(),
            example: "!balance".to_string(),
            category: CommandCategory::Account,
        });

        // 交易功能指令
        self.register_command(CommandInfo {
            name: "transfer".to_string(),
            description: "轉帳給指定用戶".to_string(),
            usage: "!transfer <@用戶> <金額>".to_string(),
            example: "!transfer @user 500".to_string(),
            category: CommandCategory::Transaction,
        });

        // 查詢功能指令
        self.register_command(CommandInfo {
            name: "history".to_string(),
            description: "查看交易歷史記錄".to_string(),
            usage: "!history".to_string(),
            example: "!history".to_string(),
            category: CommandCategory::Query,
        });

        // 管理員功能指令
        self.register_command(CommandInfo {
            name: "adjust_balance".to_string(),
            description: "管理員專用：調整用戶帳戶餘額".to_string(),
            usage: "!adjust_balance <@用戶> <金額> <原因>".to_string(),
            example: "!adjust_balance @user +1000 獎勵發放".to_string(),
            category: CommandCategory::Admin,
        });

        self.register_command(CommandInfo {
            name: "admin_history".to_string(),
            description: "管理員專用：查看系統操作歷史記錄".to_string(),
            usage: "!admin_history [用戶ID] [天數]".to_string(),
            example: "!admin_history @user 7".to_string(),
            category: CommandCategory::Admin,
        });

        self.register_command(CommandInfo {
            name: "sync_members".to_string(),
            description: "管理員專用：同步群組成員帳戶".to_string(),
            usage: "!sync_members".to_string(),
            example: "!sync_members".to_string(),
            category: CommandCategory::Admin,
        });

        // 系統幫助指令
        self.register_command(CommandInfo {
            name: "help".to_string(),
            description: "顯示幫助信息".to_string(),
            usage: "!help [指令名稱]".to_string(),
            example: "!help".to_string(),
            category: CommandCategory::System,
        });
    }

    /// 註冊新指令
    pub fn register_command(&mut self, command_info: CommandInfo) {
        info!("註冊指令: {}", command_info.name);
        self.commands.insert(command_info.name.clone(), command_info);
    }

    /// 生成完整的幫助內容
    #[instrument(skip(self))]
    pub async fn generate_help_content(&self) -> String {
        info!("生成幫助內容");

        let mut content = String::new();

        // 添加標題（移除 Markdown 格式，因為會在 Discord embed 中顯示異常）
        content.push_str("🤖 DROAS 經濟機器人幫助\n\n");

        // 按分類組織指令
        let mut categories = HashMap::new();
        for command_info in self.commands.values() {
            categories.entry(command_info.category.clone())
                .or_insert_with(Vec::new)
                .push(command_info.clone());
        }

        // 按分類顯示指令
        for (category, commands) in categories {
            let category_title = match category {
                CommandCategory::Account => "💰 帳戶管理",
                CommandCategory::Transaction => "💸 交易功能",
                CommandCategory::Query => "📊 查詢功能",
                CommandCategory::Admin => "🔧 管理員功能",
                CommandCategory::System => "⚙️ 系統幫助",
            };

            content.push_str(&format!("### {}\n\n", category_title));

            for command in commands {
                content.push_str(&format!(
                    "**{}** {}\n\n",
                    command.usage,
                    command.description
                ));
                content.push_str(&format!("📝 範例: `{}`\n\n", command.example));
            }
        }

    
        debug!("幫助內容生成完成，長度: {}", content.len());
        content
    }

    /// 獲取特定指令的幫助信息
    #[instrument(skip(self), fields(command_name))]
    pub async fn get_command_help(&self, command_name: &str) -> Result<String> {
        debug!("獲取指令幫助: {}", command_name);

        if let Some(command_info) = self.commands.get(command_name) {
            let help_content = format!(
                "## 指令幫助: {}\n\n**描述**: {}\n\n**用法**: `{}`\n\n**範例**: `{}`\n\n",
                command_info.name,
                command_info.description,
                command_info.usage,
                command_info.example
            );

            info!("成功獲取指令幫助: {}", command_name);
            Ok(help_content)
        } else {
            warn!("找不到指令: {}", command_name);

            // 返回友好的錯誤消息和可用指令列表
            let available_commands = self.commands.keys()
                .map(|name| format!("!{}", name))
                .collect::<Vec<_>>()
                .join(", ");

            let error_message = format!(
                "❌ 找不到指令: `{}`\n\n**可用指令**:\n{}\n\n使用 `!help` 查看所有可用指令。",
                command_name,
                available_commands
            );

            Err(DiscordError::UnknownCommand(error_message))
        }
    }

    /// 獲取指令資訊
    pub async fn get_command_info(&self, command_name: &str) -> Result<CommandInfo> {
        self.commands.get(command_name)
            .cloned()
            .ok_or_else(|| DiscordError::UnknownCommand(format!("找不到指令: {}", command_name)))
    }

    /// 獲取所有可用指令列表
    pub async fn get_available_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// 按分類獲取指令
    pub async fn get_commands_by_category(&self, category: &CommandCategory) -> Vec<CommandInfo> {
        self.commands.values()
            .filter(|cmd| &cmd.category == category)
            .cloned()
            .collect()
    }

    /// 檢查指令是否存在
    pub async fn is_command_available(&self, command_name: &str) -> bool {
        self.commands.contains_key(command_name)
    }

    /// 搜索相關指令
    pub async fn search_commands(&self, query: &str) -> Vec<CommandInfo> {
        let query_lower = query.to_lowercase();
        self.commands.values()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower) ||
                cmd.description.to_lowercase().contains(&query_lower) ||
                cmd.category.to_string().to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

impl Default for HelpService {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandCategory::Account => write!(f, "帳戶管理"),
            CommandCategory::Transaction => write!(f, "交易功能"),
            CommandCategory::Query => write!(f, "查詢功能"),
            CommandCategory::Admin => write!(f, "管理員功能"),
            CommandCategory::System => write!(f, "系統幫助"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_service_creation() {
        let help_service = HelpService::new();
        let commands = help_service.commands;

        // 驗證基本指令都已註冊
        assert!(commands.contains_key("balance"));
        assert!(commands.contains_key("transfer"));
        assert!(commands.contains_key("history"));
        assert!(commands.contains_key("help"));
    }

    #[test]
    fn test_command_info_structure() {
        let command_info = CommandInfo {
            name: "test".to_string(),
            description: "測試指令".to_string(),
            usage: "!test".to_string(),
            example: "!test".to_string(),
            category: CommandCategory::System,
        };

        assert_eq!(command_info.name, "test");
        assert_eq!(command_info.description, "測試指令");
        assert_eq!(command_info.category, CommandCategory::System);
    }

    #[tokio::test]
    async fn test_get_available_commands() {
        let help_service = HelpService::new();
        let commands = help_service.get_available_commands().await;

        assert!(!commands.is_empty());
        assert!(commands.contains(&"balance".to_string()));
        assert!(commands.contains(&"help".to_string()));
    }

    #[tokio::test]
    async fn test_is_command_available() {
        let help_service = HelpService::new();

        assert!(help_service.is_command_available("balance").await);
        assert!(help_service.is_command_available("help").await);
        assert!(!help_service.is_command_available("nonexistent").await);
    }

    #[tokio::test]
    async fn test_get_command_help_existing() {
        let help_service = HelpService::new();
        let result = help_service.get_command_help("balance").await;

        assert!(result.is_ok());
        let help_text = result.unwrap();
        assert!(help_text.contains("balance"));
        assert!(help_text.contains("描述"));
    }

    #[tokio::test]
    async fn test_get_command_help_nonexistent() {
        let help_service = HelpService::new();
        let result = help_service.get_command_help("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_help_content() {
        let help_service = HelpService::new();
        let content = help_service.generate_help_content().await;

        assert!(!content.is_empty());
        assert!(content.contains("DROAS"));
        assert!(content.contains("balance"));
        assert!(content.contains("transfer"));
    }

    #[tokio::test]
    async fn test_register_command() {
        let mut help_service = HelpService::new();
        let new_command = CommandInfo {
            name: "test".to_string(),
            description: "測試指令".to_string(),
            usage: "!test".to_string(),
            example: "!test".to_string(),
            category: CommandCategory::System,
        };

        help_service.register_command(new_command);
        assert!(help_service.is_command_available("test").await);
    }

    #[tokio::test]
    async fn test_get_commands_by_category() {
        let help_service = HelpService::new();
        let account_commands = help_service.get_commands_by_category(&CommandCategory::Account).await;

        assert!(!account_commands.is_empty());
        assert!(account_commands.iter().any(|cmd| cmd.name == "balance"));
    }

    #[tokio::test]
    async fn test_search_commands() {
        let help_service = HelpService::new();
        let results = help_service.search_commands("餘額").await;

        // 應該找到 balance 指令（因為描述中包含相關詞彙）
        assert!(!results.is_empty());
    }
}