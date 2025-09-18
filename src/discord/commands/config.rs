//! /config 命令實現
//!
//! 提供公會配置的查看、重置等管理功能。

use crate::config::models::GuildConfig;
use crate::discord::commands::framework::{
    BoxFuture, CommandContext, CommandError, CommandHandler, CommandResult, PermissionLevel,
};
use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use tracing::{debug, error, info, warn};

/// /config 命令處理器
pub struct ConfigHandler;

impl ConfigHandler {
    /// 創建新的 /config 命令處理器
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConfigHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for ConfigHandler {
    fn name(&self) -> &'static str {
        "config"
    }

    fn description(&self) -> &'static str {
        "管理公會配置"
    }

    fn required_permissions(&self) -> PermissionLevel {
        PermissionLevel::ManageGuild
    }

    fn handle(&self, ctx: CommandContext) -> BoxFuture<'_, CommandResult<()>> {
        let this = self.clone();
        Box::pin(async move { this.handle_impl(ctx).await })
    }
}

impl ConfigHandler {
    async fn handle_impl(&self, ctx: CommandContext) -> CommandResult<()> {
        debug!("開始處理 /config 命令");

        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| CommandError::ExecutionFailed("此命令只能在伺服器中使用".to_string()))?;

        // 解析子命令
        let subcommand = ctx
            .get_option("action")
            .ok_or_else(|| CommandError::InvalidArguments("請指定操作類型".to_string()))?;

        match subcommand.resolved {
            CommandDataOptionValue::String(ref action) => match action.as_str() {
                "view" => self.handle_view_config(&ctx, guild_id.0 as i64).await,
                "reset" => self.handle_reset_config(&ctx, guild_id.0 as i64).await,
                _ => Err(CommandError::InvalidArguments(
                    "不支持的操作。可用操作：view、reset".to_string(),
                )),
            },
            _ => Err(CommandError::InvalidArguments(
                "操作類型必須是字符串".to_string(),
            )),
        }
    }
}

impl ConfigHandler {
    /// 處理查看配置命令
    async fn handle_view_config(&self, ctx: &CommandContext, guild_id: i64) -> CommandResult<()> {
        debug!("查看公會 {} 的配置", guild_id);

        // 獲取配置
        let config = ctx
            .config_service
            .get_config(guild_id)
            .await
            .map_err(CommandError::Internal)?;

        let response = match config {
            Some(config) => self.format_config_display(&config).await?,
            None => {
                "📋 **伺服器配置**\n\n❌ 此伺服器尚未進行任何配置。\n\n💡 **提示**：\n• 使用 `/set-background` 設置歡迎背景圖片\n• 配置會自動保存到資料庫".to_string()
            }
        };

        ctx.respond(&response).await?;

        info!("成功顯示公會 {} 的配置", guild_id);
        Ok(())
    }

    /// 處理重置配置命令
    async fn handle_reset_config(&self, ctx: &CommandContext, guild_id: i64) -> CommandResult<()> {
        debug!("重置公會 {} 的配置", guild_id);

        // 檢查是否有現有配置
        let existing_config = ctx
            .config_service
            .get_config(guild_id)
            .await
            .map_err(CommandError::Internal)?;

        if existing_config.is_none() {
            ctx.respond("⚠️ 此伺服器沒有需要重置的配置。").await?;
            return Ok(());
        }

        // 詢問確認
        let confirmation_message = "⚠️ **配置重置確認**\n\n您確定要重置此伺服器的所有配置嗎？此操作將：\n• 清除歡迎背景圖片設定\n• 清除所有自定義配置\n• **此操作無法撤銷**\n\n如果確認要重置，請使用 `/config action:reset` 並附加 `confirm:true` 參數。";

        // 檢查是否有確認參數
        let confirmed = ctx
            .get_option("confirm")
            .and_then(|opt| match &opt.resolved {
                CommandDataOptionValue::Boolean(confirmed) => Some(*confirmed),
                _ => None,
            })
            .unwrap_or(false);

        if !confirmed {
            ctx.respond(confirmation_message).await?;
            return Ok(());
        }

        // 執行重置
        self.execute_config_reset(ctx, guild_id).await
    }

    /// 執行配置重置
    async fn execute_config_reset(&self, ctx: &CommandContext, guild_id: i64) -> CommandResult<()> {
        debug!("執行公會 {} 的配置重置", guild_id);

        // 刪除配置
        let deleted = ctx
            .config_service
            .delete_config(guild_id)
            .await
            .map_err(CommandError::Internal)?;

        if deleted {
            let success_message = "✅ **配置重置成功**\n\n📋 此伺服器的所有配置已被清除。\n\n💡 **下一步**：\n• 使用 `/set-background` 重新設置背景圖片\n• 所有新配置將自動保存";

            ctx.respond(success_message).await?;

            info!("成功重置公會 {} 的配置", guild_id);
        } else {
            warn!("嘗試重置公會 {} 配置時未找到配置記錄", guild_id);
            ctx.respond("⚠️ 沒有找到需要重置的配置。").await?;
        }

        Ok(())
    }

    /// 格式化配置顯示
    async fn format_config_display(&self, config: &GuildConfig) -> CommandResult<String> {
        let mut display = String::new();
        display.push_str("📋 **伺服器配置**\n\n");

        // 基本信息
        display.push_str(&format!("🆔 **公會ID**：`{}`\n", config.guild_id));
        display.push_str(&format!(
            "📅 **創建時間**：<t:{}:F>\n",
            config.created_at.timestamp()
        ));
        display.push_str(&format!(
            "🔄 **最後更新**：<t:{}:R>\n\n",
            config.updated_at.timestamp()
        ));

        // 歡迎頻道配置
        display.push_str("📢 **歡迎頻道**：");
        match config.welcome_channel_id {
            Some(channel_id) => {
                display.push_str(&format!("<#{}>", channel_id));
            }
            None => {
                display.push_str("❌ 未設置");
            }
        }
        display.push('\n');

        // 背景圖片配置
        display.push_str("🖼️ **背景圖片**：");
        match &config.background_ref {
            Some(bg_ref) => {
                display.push_str(&format!("✅ `{}`", bg_ref));

                // 檢查文件是否存在（簡單驗證）
                let bg_info = self.get_background_info(bg_ref).await;
                match bg_info {
                    Ok(Some(info)) => {
                        display.push_str(&format!("\n  └── 📁 檔案大小：{} KB", info.size_kb));
                        display.push_str(&format!("\n  └── 🔗 格式：{}", info.format));
                    }
                    Ok(None) => {
                        display.push_str("\n  └── ⚠️ 檔案可能不存在");
                    }
                    Err(_) => {
                        display.push_str("\n  └── ❌ 檔案狀態未知");
                    }
                }
            }
            None => {
                display.push_str("❌ 未設置");
            }
        }
        display.push('\n');

        // 添加操作提示
        display.push_str("\n💡 **可用操作**：\n");
        display.push_str("• `/set-background` - 設置背景圖片\n");
        display.push_str("• `/preview` - 預覽歡迎圖片\n");
        display.push_str("• `/config action:reset` - 重置所有配置\n");

        Ok(display)
    }

    /// 獲取背景圖片信息
    async fn get_background_info(&self, bg_ref: &str) -> CommandResult<Option<BackgroundInfo>> {
        // 注意：這是一個簡化的實現
        // 在實際項目中，這裡應該：
        // 1. 從配置服務或資產管理器獲取詳細信息
        // 2. 檢查實際文件存在性
        // 3. 獲取文件元數據

        // 為了演示，這裡返回一個模擬的信息
        if bg_ref.starts_with("bg_") {
            Ok(Some(BackgroundInfo {
                size_kb: 1024, // 模擬 1MB
                format: if bg_ref.contains("png") || bg_ref.ends_with("png") {
                    "PNG".to_string()
                } else {
                    "JPEG".to_string()
                },
            }))
        } else {
            Ok(None)
        }
    }
}

/// 背景圖片信息
struct BackgroundInfo {
    size_kb: u64,
    format: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_handler_properties() {
        let handler = ConfigHandler::new();

        assert_eq!(handler.name(), "config");
        assert_eq!(handler.required_permissions(), PermissionLevel::ManageGuild);
        assert!(!handler.description().is_empty());
    }

    #[tokio::test]
    async fn test_format_config_display() {
        let handler = ConfigHandler::new();
        let now = Utc::now();

        let config = GuildConfig {
            guild_id: 123456789,
            welcome_channel_id: Some(987654321),
            background_ref: Some("bg_test_12345".to_string()),
            created_at: now,
            updated_at: now,
        };

        let display = handler.format_config_display(&config).await.unwrap();

        assert!(display.contains("123456789"));
        assert!(display.contains("<#987654321>"));
        assert!(display.contains("bg_test_12345"));
        assert!(display.contains("📋 **伺服器配置**"));
    }

    #[tokio::test]
    async fn test_format_empty_config_display() {
        let handler = ConfigHandler::new();
        let now = Utc::now();

        let config = GuildConfig {
            guild_id: 123456789,
            welcome_channel_id: None,
            background_ref: None,
            created_at: now,
            updated_at: now,
        };

        let display = handler.format_config_display(&config).await.unwrap();

        assert!(display.contains("❌ 未設置"));
        assert!(display.contains("123456789"));
    }

    #[tokio::test]
    async fn test_get_background_info() {
        let handler = ConfigHandler::new();

        // 測試有效的背景引用
        let info = handler.get_background_info("bg_12345_png").await.unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.format, "PNG");

        // 測試JPEG格式
        let info = handler.get_background_info("bg_12345_jpg").await.unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.format, "JPEG");

        // 測試無效引用
        let info = handler.get_background_info("invalid_ref").await.unwrap();
        assert!(info.is_none());
    }

    #[test]
    fn test_default_implementation() {
        let handler1 = ConfigHandler::new();
        let handler2 = ConfigHandler::default();

        assert_eq!(handler1.name(), handler2.name());
        assert_eq!(
            handler1.required_permissions(),
            handler2.required_permissions()
        );
    }
}
