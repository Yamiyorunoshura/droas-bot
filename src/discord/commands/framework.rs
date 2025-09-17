//! Discord 命令處理框架
//! 
//! 提供命令處理的核心基礎設施，包括權限檢查、參數解析、錯誤處理等。

use crate::config::service::GuildConfigService;
use crate::error::{DroasError, DroasResult};
use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::application::interaction::{InteractionResponseType, MessageFlags};
use serenity::model::prelude::{GuildId, UserId};
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// 命令執行結果類型別名
pub type CommandResult<T> = Result<T, CommandError>;

/// 權限級別枚舉
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    /// 所有人都可以使用
    Everyone,
    /// 需要管理伺服器權限
    ManageGuild,
    /// 需要管理員權限
    Administrator,
}

/// 命令錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("權限不足：{0}")]
    InsufficientPermissions(String),
    
    #[error("參數錯誤：{0}")]
    InvalidArguments(String),
    
    #[error("命令執行失敗：{0}")]
    ExecutionFailed(String),
    
    #[error("命令超時")]
    Timeout,
    
    #[error("內部錯誤：{0}")]
    Internal(#[from] DroasError),
    
    #[error("Discord API 錯誤：{0}")]
    DiscordApi(#[from] serenity::Error),
}

/// 命令上下文 - 包含執行命令所需的所有資訊
pub struct CommandContext {
    /// Discord 上下文
    pub discord_ctx: Context,
    /// 命令互動
    pub interaction: ApplicationCommandInteraction,
    /// 配置服務
    pub config_service: Arc<GuildConfigService>,
    /// 命令參數
    pub options: HashMap<String, CommandDataOption>,
}

impl CommandContext {
    /// 創建新的命令上下文
    pub fn new(
        discord_ctx: Context,
        interaction: ApplicationCommandInteraction,
        config_service: Arc<GuildConfigService>,
    ) -> Self {
        let options = interaction
            .data
            .options
            .iter()
            .map(|opt| (opt.name.clone(), opt.clone()))
            .collect();

        Self {
            discord_ctx,
            interaction,
            config_service,
            options,
        }
    }
    
    /// 獲取執行命令的用戶ID
    pub fn user_id(&self) -> UserId {
        self.interaction.user.id
    }
    
    /// 獲取命令執行的公會ID
    pub fn guild_id(&self) -> Option<GuildId> {
        self.interaction.guild_id
    }
    
    /// 獲取命令參數
    pub fn get_option(&self, name: &str) -> Option<&CommandDataOption> {
        self.options.get(name)
    }
    
    /// 回應命令（立即回應）
    pub async fn respond(&self, content: &str) -> CommandResult<()> {
        self.interaction
            .create_interaction_response(&self.discord_ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
            .map_err(CommandError::from)
    }
    
    /// 回應命令並附帶文件
    pub async fn respond_with_file(
        &self,
        content: &str,
        filename: &str,
        file_data: Vec<u8>,
    ) -> CommandResult<()> {
        let attachment = serenity::model::channel::AttachmentType::Bytes {
            data: std::borrow::Cow::from(file_data),
            filename: filename.to_string(),
        };
        
        self.interaction
            .create_interaction_response(&self.discord_ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(content).add_file(attachment)
                    })
            })
            .await
            .map_err(CommandError::from)
    }
    
    /// 回應錯誤訊息（只有執行者可見）
    pub async fn respond_error(&self, error: &str) -> CommandResult<()> {
        self.interaction
            .create_interaction_response(&self.discord_ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("❌ {}", error))
                            .flags(MessageFlags::EPHEMERAL)
                    })
            })
            .await
            .map_err(CommandError::from)
    }
    
    /// 延遲回應（用於長時間處理的命令）
    pub async fn defer_response(&self) -> CommandResult<()> {
        self.interaction
            .create_interaction_response(&self.discord_ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
            .map_err(CommandError::from)
    }
    
    /// 編輯延遲回應
    pub async fn edit_response(&self, content: &str) -> CommandResult<()> {
        self.interaction
            .edit_original_interaction_response(&self.discord_ctx.http, |response| {
                response.content(content)
            })
            .await
            .map_err(CommandError::from)
    }
}

/// 命令處理器 trait - 所有命令都必須實現此 trait
#[async_trait]
pub trait CommandHandler {
    /// 處理命令
    async fn handle(&self, ctx: CommandContext) -> CommandResult<()>;
    
    /// 命令名稱
    fn name(&self) -> &'static str;
    
    /// 命令描述
    fn description(&self) -> &'static str;
    
    /// 所需權限級別
    fn required_permissions(&self) -> PermissionLevel;
    
    /// 註冊命令到Discord（可選覆寫）
    fn register(&self, command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command.name(self.name()).description(self.description())
    }
}

/// 命令處理框架 - 負責命令的分發和執行
pub struct CommandFramework {
    config_service: Arc<GuildConfigService>,
    command_timeout: Duration,
}

impl CommandFramework {
    /// 創建新的命令框架
    pub fn new(config_service: Arc<GuildConfigService>) -> Self {
        Self {
            config_service,
            command_timeout: Duration::from_secs(30), // 30秒超時
        }
    }
    
    /// 設置命令超時時間
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }
    
    /// 執行命令
    pub async fn execute_command(
        &self,
        ctx: Context,
        interaction: ApplicationCommandInteraction,
        handler: &dyn CommandHandler,
    ) -> CommandResult<()> {
        let command_name = handler.name();
        let user_id = interaction.user.id;
        let guild_id = interaction.guild_id;
        
        info!(
            "執行命令 '{}' - 用戶: {}, 公會: {:?}",
            command_name, user_id, guild_id
        );
        
        // 創建命令上下文
        let command_ctx = CommandContext::new(ctx, interaction, self.config_service.clone());
        
        // 檢查權限
        if let Err(e) = self.check_permissions(&command_ctx, handler).await {
            warn!(
                "權限檢查失敗 - 命令: '{}', 用戶: {}, 錯誤: {}",
                command_name, user_id, e
            );
            command_ctx.respond_error(&e.to_string()).await?;
            return Err(e);
        }
        
        // 執行命令（帶超時保護）
        let result = timeout(self.command_timeout, handler.handle(command_ctx)).await;
        
        match result {
            Ok(Ok(())) => {
                info!(
                    "命令執行成功 - 命令: '{}', 用戶: {}",
                    command_name, user_id
                );
                Ok(())
            }
            Ok(Err(e)) => {
                error!(
                    "命令執行失敗 - 命令: '{}', 用戶: {}, 錯誤: {}",
                    command_name, user_id, e
                );
                Err(e)
            }
            Err(_) => {
                error!(
                    "命令執行超時 - 命令: '{}', 用戶: {}",
                    command_name, user_id
                );
                Err(CommandError::Timeout)
            }
        }
    }
    
    /// 檢查用戶權限
    async fn check_permissions(
        &self,
        ctx: &CommandContext,
        handler: &dyn CommandHandler,
    ) -> CommandResult<()> {
        let required_level = handler.required_permissions();
        
        // 如果命令對所有人開放，直接允許
        if required_level == PermissionLevel::Everyone {
            return Ok(());
        }
        
        // 必須在公會內執行
        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| CommandError::InsufficientPermissions("此命令只能在伺服器中使用".to_string()))?;
        
        // 獲取用戶在公會中的權限
        let member = ctx
            .discord_ctx
            .cache
            .member(guild_id, ctx.user_id())
            .ok_or_else(|| CommandError::InsufficientPermissions("無法獲取用戶權限信息".to_string()))?;
        
        let permissions = member.permissions(&ctx.discord_ctx.cache)
            .map_err(|e| CommandError::InsufficientPermissions(format!("權限檢查失敗: {}", e)))?;
        
        // 檢查所需權限
        let has_permission = match required_level {
            PermissionLevel::Everyone => true,
            PermissionLevel::ManageGuild => permissions.manage_guild(),
            PermissionLevel::Administrator => permissions.administrator(),
        };
        
        if has_permission {
            debug!(
                "權限檢查通過 - 用戶: {}, 所需權限: {:?}",
                ctx.user_id(),
                required_level
            );
            Ok(())
        } else {
            Err(CommandError::InsufficientPermissions(format!(
                "需要 {:?} 權限才能執行此命令",
                required_level
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::service::GuildConfigService;
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::NamedTempFile;
    
    async fn setup_test_config_service() -> Arc<GuildConfigService> {
        let temp_file = NamedTempFile::new().expect("創建臨時文件失敗");
        let database_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("連接測試資料庫失敗");
        
        let service = GuildConfigService::new(
            pool,
            Some(10),   // 小緩存用於測試
            Some(60),   // 1分鐘TTL
            Some(5),    // 5秒事務超時
            Some(false), // 不預加載配置
        )
        .await
        .expect("創建測試配置服務失敗");
        
        Arc::new(service)
    }
    
    #[test]
    fn test_permission_levels() {
        assert_eq!(PermissionLevel::Everyone, PermissionLevel::Everyone);
        assert_ne!(PermissionLevel::Everyone, PermissionLevel::ManageGuild);
    }
    
    #[test]
    fn test_command_error_display() {
        let error = CommandError::InsufficientPermissions("測試錯誤".to_string());
        assert!(error.to_string().contains("權限不足"));
        
        let error = CommandError::InvalidArguments("參數錯誤".to_string());
        assert!(error.to_string().contains("參數錯誤"));
    }
    
    #[tokio::test]
    async fn test_command_framework_creation() {
        let config_service = setup_test_config_service().await;
        let framework = CommandFramework::new(config_service);
        
        // 測試超時設定
        let framework_with_timeout = framework.with_timeout(Duration::from_secs(10));
        assert_eq!(framework_with_timeout.command_timeout, Duration::from_secs(10));
    }
}