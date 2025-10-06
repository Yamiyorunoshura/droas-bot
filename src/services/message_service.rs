// Message Service - 用戶界面服務
// 提供用戶友好的響應格式和 Discord embed 創建

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::error::{DiscordError, Result};
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::id::UserId;
use crate::styles::embed_themes::EmbedTheme;
use crate::services::ui_components::UIComponentFactory;

/// 響應格式結構
#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub content: String,
    pub is_embed: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<MessageField>,
    pub color: Option<u32>,
}

/// 消息欄位結構
#[derive(Debug, Clone)]
pub struct MessageField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

/// Message Service - 處理用戶界面響應格式
pub struct MessageService {
    default_color: u32,
    ui_factory: UIComponentFactory,
}

impl MessageService {
    /// 創建新的 MessageService 實例
    pub fn new() -> Self {
        Self {
            default_color: 0x00FF00, // 綠色
            ui_factory: UIComponentFactory::new(),
        }
    }

    /// 創建帶自定義顏色的 MessageService 實例
    pub fn with_color(color: u32) -> Self {
        Self {
            default_color: color,
            ui_factory: UIComponentFactory::new(),
        }
    }

    /// 格式化餘額查詢響應
    ///
    /// # Arguments
    /// * `user_id` - Discord 用戶 ID
    /// * `username` - 用戶名稱
    /// * `balance` - 帳戶餘額
    /// * `created_at` - 帳戶創建時間
    ///
    /// # Returns
    /// * `Ok(MessageResponse)` - 格式化的響應
    /// * `Err(DiscordError)` - 格式化失敗
    pub fn format_balance_response(
        &self,
        user_id: u64,
        username: &str,
        balance: &BigDecimal,
        created_at: Option<DateTime<Utc>>,
    ) -> Result<MessageResponse> {
        let created_str = match created_at {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            None => "未知".to_string(),
        };

        let fields = vec![
            MessageField {
                name: "用戶 ID".to_string(),
                value: user_id.to_string(),
                inline: true,
            },
            MessageField {
                name: "用戶名稱".to_string(),
                value: username.to_string(),
                inline: true,
            },
            MessageField {
                name: "帳戶餘額".to_string(),
                value: format!("{} 幣", balance),
                inline: false,
            },
            MessageField {
                name: "創建時間".to_string(),
                value: created_str,
                inline: true,
            },
        ];

        Ok(MessageResponse {
            content: String::new(), // Discord 嵌入消息不需要主要內容
            is_embed: true,
            title: Some("💰 帳戶餘額查詢".to_string()),
            description: Some(format!("{} 的帳戶資訊", username)),
            fields,
            color: Some(self.default_color),
        })
    }

    /// 格式化錯誤響應
    ///
    /// # Arguments
    /// * `error` - 錯誤類型
    ///
    /// # Returns
    /// * `MessageResponse` - 格式化的錯誤響應
    pub fn format_error_response(&self, error: &DiscordError) -> MessageResponse {
        let (title, description, color) = match error {
            DiscordError::UserNotFound(msg) => (
                "❌ 帳戶錯誤".to_string(),
                msg.clone(),
                0xFF0000, // 紅色
            ),
            DiscordError::InvalidCommand(msg) => (
                "⚠️ 指令錯誤".to_string(),
                format!("指令格式不正確：{}", msg),
                0xFFFF00, // 黃色
            ),
            DiscordError::UnknownCommand(cmd) => (
                "❓ 未知指令".to_string(),
                format!("未知指令：{}，請使用 `!help` 查看可用指令", cmd),
                0xFFFF00, // 黃色
            ),
            DiscordError::DatabaseQueryError(msg) => (
                "💾 資料庫錯誤".to_string(),
                format!("資料庫查詢失敗：{}", msg),
                0xFF0000, // 紅色
            ),
            _ => (
                "⚠️ 系統錯誤".to_string(),
                format!("發生錯誤：{}", error),
                0xFF0000, // 紅色
            ),
        };

        MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some(title),
            description: Some(description),
            fields: vec![],
            color: Some(color),
        }
    }

    /// 格式化簡單文本響應
    ///
    /// # Arguments
    /// * `message` - 消息內容
    ///
    /// # Returns
    /// * `MessageResponse` - 文本響應
    pub fn format_text_response(&self, message: &str) -> MessageResponse {
        MessageResponse {
            content: message.to_string(),
            is_embed: false,
            title: None,
            description: None,
            fields: vec![],
            color: None,
        }
    }

    /// 格式化幫助響應
    ///
    /// # Arguments
    /// * `commands` - 可用指令列表
    ///
    /// # Returns
    /// * `MessageResponse` - 幫助響應
    pub fn format_help_response(&self, commands: &[String]) -> MessageResponse {
        let command_list = commands
            .iter()
            .map(|cmd| format!("• `{}`", cmd))
            .collect::<Vec<_>>()
            .join("\n");

        let fields = vec![
            MessageField {
                name: "可用指令".to_string(),
                value: command_list,
                inline: false,
            },
            MessageField {
                name: "使用說明".to_string(),
                value: "使用 `!指令名稱` 來執行相應功能".to_string(),
                inline: false,
            },
        ];

        MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some("🤖 DROAS Bot 幫助".to_string()),
            description: Some("以下是可用的指令列表".to_string()),
            fields,
            color: Some(0x0099FF), // 藍色
        }
    }

    /// 格式化詳細幫助響應（與 HelpService 整合）
    ///
    /// # Arguments
    ///
    /// * `help_content` - HelpService 生成的幫助內容
    ///
    /// # Returns
    ///
    /// * `MessageResponse` - 詳細幫助響應
    pub fn format_detailed_help_response(&self, help_content: &str) -> MessageResponse {
        MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some("🤖 DROAS 經濟機器人幫助".to_string()),
            description: Some("完整的指令使用指南".to_string()),
            fields: vec![
                MessageField {
                    name: "幫助內容".to_string(),
                    value: help_content.to_string(),
                    inline: false,
                },
                MessageField {
                    name: "提示".to_string(),
                    value: "使用 `!help <指令名稱>` 獲取特定指令的詳細幫助".to_string(),
                    inline: false,
                },
            ],
            color: Some(0x0099FF), // 藍色
        }
    }

    /// 格式化轉帳成功響應
    ///
    /// # Arguments
    /// * `transfer_result` - 轉帳結果
    ///
    /// # Returns
    /// * `Ok(MessageResponse)` - 格式化的成功響應
    /// * `Err(DiscordError)` - 格式化失敗
    pub fn format_transfer_success_response(
        &self,
        transfer_result: &crate::services::transfer_service::TransferResult,
    ) -> Result<MessageResponse> {
        let fields = vec![
            MessageField {
                name: "發送者".to_string(),
                value: format!("{} ({})", transfer_result.from_user.username, transfer_result.from_user.discord_user_id),
                inline: true,
            },
            MessageField {
                name: "接收者".to_string(),
                value: format!("{} ({})", transfer_result.to_user.username, transfer_result.to_user.discord_user_id),
                inline: true,
            },
            MessageField {
                name: "轉帳金額".to_string(),
                value: format!("{} 幣", transfer_result.amount),
                inline: false,
            },
            MessageField {
                name: "交易 ID".to_string(),
                value: transfer_result.transaction_id.as_ref().unwrap_or(&"未知".to_string()).clone(),
                inline: true,
            },
            MessageField {
                name: "發送者餘額".to_string(),
                value: format!("{} 幣", transfer_result.from_user.balance),
                inline: true,
            },
            MessageField {
                name: "接收者餘額".to_string(),
                value: format!("{} 幣", transfer_result.to_user.balance),
                inline: true,
            },
        ];

        Ok(MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some("✅ 轉帳成功".to_string()),
            description: Some(transfer_result.message.clone()),
            fields,
            color: Some(0x00FF00), // 綠色
        })
    }

    /// 格式化轉帳錯誤響應
    ///
    /// # Arguments
    /// * `transfer_result` - 轉帳結果
    ///
    /// # Returns
    /// * `MessageResponse` - 格式化的錯誤響應
    pub fn format_transfer_error_response(
        &self,
        transfer_result: &crate::services::transfer_service::TransferResult,
    ) -> MessageResponse {
        let fields = vec![
            MessageField {
                name: "發送者".to_string(),
                value: format!("{} ({})", transfer_result.from_user.username, transfer_result.from_user.discord_user_id),
                inline: true,
            },
            MessageField {
                name: "轉帳金額".to_string(),
                value: format!("{} 幣", transfer_result.amount),
                inline: true,
            },
            MessageField {
                name: "當前餘額".to_string(),
                value: format!("{} 幣", transfer_result.from_user.balance),
                inline: true,
            },
        ];

        MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some("❌ 轉帳失敗".to_string()),
            description: Some(transfer_result.message.clone()),
            fields,
            color: Some(0xFF0000), // 紅色
        }
    }

    /// 格式化歷史查詢響應
    ///
    /// # Arguments
    ///
    /// * `user_id` - Discord 用戶 ID
    /// * `transactions` - 交易記錄列表
    ///
    /// # Returns
    /// * `Ok(MessageResponse)` - 格式化的歷史記錄響應
    pub fn format_history_response(
        &self,
        user_id: i64,
        transactions: &[crate::database::transaction_repository::Transaction],
    ) -> Result<MessageResponse> {
        if transactions.is_empty() {
            // 沒有交易記錄
            return Ok(MessageResponse {
                content: String::new(),
                is_embed: true,
                title: Some("📋 交易歷史".to_string()),
                description: Some(format!("用戶 <@{}> 暫無交易記錄", user_id)),
                fields: Vec::new(),
                color: Some(0xFFA500), // 橙色
            });
        }

        // 創建交易記錄欄位
        let mut fields = Vec::new();
        for (i, transaction) in transactions.iter().take(10).enumerate() { // 最多顯示 10 筆
            let transaction_type = match transaction.transaction_type.as_str() {
                "transfer" => "💸 轉帳",
                _ => "📝 交易",
            };

            let amount = &transaction.amount;
            let created_at = transaction.created_at.format("%Y-%m-%d %H:%M");

            // 確定交易方向
            let direction = if transaction.from_user_id == Some(user_id) {
                "發送"
            } else if transaction.to_user_id == Some(user_id) {
                "接收"
            } else {
                "未知"
            };

            let counterparty = if transaction.from_user_id == Some(user_id) {
                transaction.to_user_id.map(|id| format!("<@{}>", id))
            } else if transaction.to_user_id == Some(user_id) {
                transaction.from_user_id.map(|id| format!("<@{}>", id))
            } else {
                None
            };

            let field_value = if let Some(counterparty) = counterparty {
                format!("{} {} {} - {}\n時間: {}",
                    if direction == "發送" { "➖" } else { "➕" },
                    transaction_type,
                    amount,
                    counterparty,
                    created_at
                )
            } else {
                format!("{} {} {}\n時間: {}",
                    if direction == "發送" { "➖" } else { "➕" },
                    transaction_type,
                    amount,
                    created_at
                )
            };

            fields.push(MessageField {
                name: format!("交易 #{}", i + 1),
                value: field_value,
                inline: false,
            });
        }

        Ok(MessageResponse {
            content: String::new(),
            is_embed: true,
            title: Some("📋 交易歷史".to_string()),
            description: Some(format!("用戶 <@{}> 的最近 {} 筆交易記錄", user_id, transactions.len().min(10))),
            fields,
            color: Some(0x0099FF), // 藍色
        })
    }

    /// 將 MessageResponse 轉換為 Discord 字符串（簡化版）
    ///
    /// 在實際 Discord 整合中，這會轉換為 Discord Embed 結構
    /// 目前返回格式化的文本字符串供測試使用
    ///
    /// # Arguments
    /// * `response` - MessageResponse 實例
    ///
    /// # Returns
    /// * `String` - Discord 字符串格式
    pub fn to_discord_string(&self, response: &MessageResponse) -> String {
        if response.is_embed {
            let mut result = String::new();

            if let Some(title) = &response.title {
                result.push_str(&format!("**{}**\n", title));
            }

            if let Some(description) = &response.description {
                result.push_str(&format!("{}\n", description));
            }

            for field in &response.fields {
                result.push_str(&format!("**{}**: {}\n", field.name, field.value));
            }

            result
        } else {
            response.content.clone()
        }
    }

    // ===== Discord Embed 創建方法 =====

    /// 創建餘額查詢的 embed 消息
    pub async fn create_balance_embed(&self, _user_id: UserId, balance: f64) -> CreateMessage {
        let embed = CreateEmbed::default()
            .title("💰 餘額查詢")
            .description(format!("您的當前餘額：$ {:.2}", balance))
            .color(EmbedTheme::Success.color());

        CreateMessage::default().add_embed(embed)
    }

    /// 創建轉帳相關的 embed 消息
    pub async fn create_transfer_embed(&self, from_user: UserId, to_user: UserId, amount: f64) -> CreateMessage {
        let embed = CreateEmbed::default()
            .title("💸 轉帳操作")
            .description(format!(
                "轉帳金額：$ {:.2}\n從用戶 <@{}> 到用戶 <@{}>",
                amount, from_user, to_user
            ))
            .color(EmbedTheme::Info.color());

        let buttons = self.ui_factory.create_action_buttons(
            &format!("transfer_{}_{}_{}", from_user, to_user, amount as u64)
        );

        let mut message = CreateMessage::default().add_embed(embed);

        for button in buttons {
            message = message.button(
                serenity::builder::CreateButton::new(button.custom_id.unwrap_or_default())
                    .style(button.style)
                    .label(button.label.unwrap_or_default())
            );
        }

        message
    }

    /// 創建交易歷史的 embed 消息
    pub async fn create_history_embed<T>(&self, _user_id: UserId, transactions: Vec<T>) -> CreateMessage {
        let description = if transactions.is_empty() {
            "暫無交易記錄".to_string()
        } else {
            format!("共有 {} 筆交易記錄", transactions.len())
        };

        let embed = CreateEmbed::default()
            .title("📜 交易歷史")
            .description(description)
            .color(EmbedTheme::Info.color());

        CreateMessage::default().add_embed(embed)
    }

    /// 創建成功消息的 embed
    pub async fn create_success_embed(&self, message: &str) -> CreateMessage {
        self.create_embed_with_theme(message, EmbedTheme::Success).await
    }

    /// 創建信息消息的 embed
    pub async fn create_info_embed(&self, message: &str) -> CreateMessage {
        self.create_embed_with_theme(message, EmbedTheme::Info).await
    }

    /// 創建警告消息的 embed
    pub async fn create_warning_embed(&self, message: &str) -> CreateMessage {
        self.create_embed_with_theme(message, EmbedTheme::Warning).await
    }

    /// 創建錯誤消息的 embed
    pub async fn create_error_embed(&self, message: &str) -> CreateMessage {
        self.create_embed_with_theme(message, EmbedTheme::Error).await
    }

    /// 根據主題創建 embed 的私有方法
    async fn create_embed_with_theme(&self, message: &str, theme: EmbedTheme) -> CreateMessage {
        let embed = CreateEmbed::default()
            .title(self.get_title_for_theme(&theme))
            .description(message)
            .color(theme.color());

        CreateMessage::default().add_embed(embed)
    }

    /// 根據主題獲取對應的標題
    fn get_title_for_theme(&self, theme: &EmbedTheme) -> &str {
        match theme {
            EmbedTheme::Success => "✅ 成功",
            EmbedTheme::Info => "ℹ️ 信息",
            EmbedTheme::Warning => "⚠️ 警告",
            EmbedTheme::Error => "❌ 錯誤",
        }
    }
}

impl Default for MessageService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn test_message_service_creation() {
        let service = MessageService::new();
        assert_eq!(service.default_color, 0x00FF00);
    }

    #[test]
    fn test_balance_response_formatting() {
        let service = MessageService::new();
        let balance = BigDecimal::from_str("1000.50").unwrap();
        let created_at = Utc::now();

        let response = service.format_balance_response(
            12345,
            "TestUser",
            &balance,
            Some(created_at),
        );

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.is_embed);
        assert_eq!(response.title, Some("💰 帳戶餘額查詢".to_string()));
        assert_eq!(response.fields.len(), 4);
        assert_eq!(response.fields[2].name, "帳戶餘額");
        assert!(response.fields[2].value.contains("1000.50"));
    }

    #[test]
    fn test_error_response_formatting() {
        let service = MessageService::new();
        let error = DiscordError::UserNotFound("用戶不存在".to_string());

        let response = service.format_error_response(&error);

        assert!(response.is_embed);
        assert_eq!(response.title, Some("❌ 帳戶錯誤".to_string()));
        assert_eq!(response.description, Some("用戶不存在".to_string()));
        assert_eq!(response.color, Some(0xFF0000));
    }

    #[test]
    fn test_text_response_formatting() {
        let service = MessageService::new();
        let response = service.format_text_response("Simple message");

        assert!(!response.is_embed);
        assert_eq!(response.content, "Simple message");
        assert!(response.title.is_none());
    }

    #[test]
    fn test_help_response_formatting() {
        let service = MessageService::new();
        let commands = vec!["balance".to_string(), "transfer".to_string()];

        let response = service.format_help_response(&commands);

        assert!(response.is_embed);
        assert_eq!(response.title, Some("🤖 DROAS Bot 幫助".to_string()));
        assert_eq!(response.fields.len(), 2);
        assert!(response.fields[0].value.contains("balance"));
        assert!(response.fields[0].value.contains("transfer"));
    }

    #[test]
    fn test_to_discord_string() {
        let service = MessageService::new();
        let balance = BigDecimal::from_str("500").unwrap();

        let response = service.format_balance_response(
            12345,
            "TestUser",
            &balance,
            None,
        ).unwrap();

        let discord_str = service.to_discord_string(&response);

        assert!(discord_str.contains("帳戶餘額查詢"));
        assert!(discord_str.contains("TestUser"));
        assert!(discord_str.contains("500"));
    }
}