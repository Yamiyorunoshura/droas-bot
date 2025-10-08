use serenity::Client;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::model::gateway::GatewayIntents;
use serenity::all::Interaction;
use serenity::all::InteractionType;
use serenity::all::ComponentInteraction;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::model::channel::Message;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use crate::error::{DiscordError, Result};
use crate::logging::{log_connection_success, log_connection_error, log_command_received, log_command_processed, log_event_received};
use crate::command_router::CommandRouter;
use crate::services::ui_components::UIComponentFactory;

// Command Router modules
pub mod command_parser;
pub mod service_router;
pub mod command_registry;
pub mod router_error_handler;
pub mod router_metrics;

// Re-export main types for backward compatibility
pub use command_parser::{Command, CommandResult, CommandParser};
pub use service_router::ServiceRouter;
pub use command_registry::CommandRegistry;
pub use router_error_handler::RouterErrorHandler;
pub use router_metrics::{RouterMetrics, MetricsSnapshot, OperationTimer};

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

#[derive(Clone)]
pub struct DiscordGateway {
    client: Arc<Mutex<Option<Client>>>,
    config: Config,
    status: Arc<Mutex<ConnectionStatus>>,
    token_valid: bool,
    command_router: Arc<CommandRouter>,
    ui_factory: UIComponentFactory,
    user_account_service: Option<Arc<crate::services::UserAccountService>>,
}

struct Handler {
    status: Arc<Mutex<ConnectionStatus>>,
    ui_factory: Arc<UIComponentFactory>,
    command_router: Arc<CommandRouter>,
    user_account_service: Option<Arc<crate::services::UserAccountService>>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _ready: Ready) {
        log_connection_success();
        *self.status.lock().await = ConnectionStatus::Connected;

        // 注意：由於 Ready 事件處理器的限制，intent 驗證將在後續版本中實現
        // 這裡我們暫時記錄一個提示，因為驗證需要更多的上下文信息
        tracing::info!("🔍 GUILD_MEMBERS intent 已配置，建議在 Discord Developer Portal 驗證");
        tracing::info!("Discord Developer Portal: https://discord.com/developers/applications");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // 忽略機器人自己的訊息
        if msg.author.bot {
            return;
        }

        log_event_received("message");

        // 檢查是否為命令訊息
        if let Err(e) = self.handle_message_command(&ctx, &msg).await {
            tracing::error!("處理訊息命令時發生錯誤: {:?}", e);

            // 發送錯誤訊息
            if let Err(send_err) = msg.channel_id.say(&ctx.http, format!("❌ 命令執行失敗: {}", e)).await {
                tracing::error!("發送錯誤訊息失敗: {:?}", send_err);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log_event_received("interaction");

        match interaction.kind() {
            InteractionType::Component => {
                if let Some(component) = interaction.as_message_component() {
                    if let Err(e) = self.handle_button_interaction(ctx, component).await {
                        tracing::error!("處理按鈕交互時發生錯誤: {:?}", e);
                    }
                }
            }
            _ => {
                tracing::debug!("忽略非按鈕交互類型: {:?}", interaction.kind());
            }
        }
    }

    /// 處理 GuildMemberAdd 事件 - 新成員自動帳戶創建 (F-013)
    async fn guild_member_addition(&self, ctx: Context, new_member: serenity::model::guild::Member) {
        log_event_received("guild_member_addition");

        let user_id = new_member.user.id.get() as i64;
        let username = new_member.user.name.clone();

        tracing::info!("新成員加入群組: {} ({})", username, user_id);

        // 如果有設置 UserAccountService，則自動創建帳戶
        if let Some(user_account_service) = &self.user_account_service {
            match user_account_service.create_or_get_user_account(user_id, username.clone()).await {
                Ok(result) => {
                    if result.was_created {
                        tracing::info!("✅ 為新成員 {} 創建帳戶成功，初始餘額: {} 幣",
                            result.user.username, result.user.balance);

                        // 發送歡迎消息
                        let welcome_message = serenity::builder::CreateMessage::new()
                            .content(format!("🎉 歡迎 {}！\n\n您的經濟帳戶已自動創建，初始餘額：{} 幣\n\n使用 `!help` 查看可用命令！",
                                result.user.username, result.user.balance));

                        if let Err(e) = new_member.user.dm(&ctx.http, welcome_message).await {
                            tracing::error!("發送歡迎私訊失敗: {}", e);
                        }
                    } else {
                        tracing::info!("成員 {} 已有帳戶，跳過創建", username);
                    }
                },
                Err(e) => {
                    tracing::error!("為新成員 {} 創建帳戶失敗: {}", username, e);

                    // 發送錯誤通知
                    let error_message = serenity::builder::CreateMessage::new()
                        .content("❌ 帳戶創建失敗，請聯繫管理員。");

                    if let Err(e) = new_member.user.dm(&ctx.http, error_message).await {
                        tracing::error!("發送錯誤通知私訊失敗: {}", e);
                    }
                }
            }
        } else {
            tracing::warn!("UserAccountService 未設置，無法自動創建帳戶");
        }
    }
}

impl Handler {
    async fn handle_message_command(&self, ctx: &Context, msg: &Message) -> Result<()> {
        let content = msg.content.trim();

        // 檢查是否為命令（以 ! 開頭）
        if !content.starts_with('!') {
            return Ok(());
        }

        tracing::info!("收到命令: '{}' from user: {} ({})", content, msg.author.name, msg.author.id);

        // 解析命令
        let command_result = self.command_router.parse_command(content).await?;

        // 更新用戶資訊到 CommandResult
        let mut updated_command_result = command_result;
        updated_command_result.user_id = Some(msg.author.id.get() as i64);
        updated_command_result.username = Some(msg.author.name.clone());

        // 添加 Discord Context 和 Guild ID（如果有的話）
        if let Some(guild_id) = msg.guild_id {
            updated_command_result.guild_id = Some(guild_id.get() as i64);
        }
        updated_command_result.discord_context = Some(Arc::new(ctx.clone()));

        // 路由命令並獲得響應
        let response = self.command_router.route_command(&updated_command_result).await?;

        // 發送響應
        msg.channel_id.say(&ctx.http, response).await
            .map_err(|e| DiscordError::EventError(format!("發送響應失敗: {}", e)))?;

        Ok(())
    }

    async fn handle_button_interaction(&self, ctx: Context, component: &ComponentInteraction) -> Result<()> {
        let custom_id = &component.data.custom_id;
        let user_id = component.user.id;

        tracing::info!("處理按鈕交互: custom_id={}, user_id={}", custom_id, user_id);

        // 使用 UI 組件工廠處理按鈕交互
        match self.ui_factory.handle_button_interaction(custom_id, user_id).await {
            Ok(response) => {
                // 發送響應消息
                let create_response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(response)
                );
                if let Err(e) = component.create_response(&ctx.http, create_response).await {
                    tracing::error!("發送按鈕交互響應失敗: {:?}", e);
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("按鈕交互處理失敗: {:?}", e);

                // 發送錯誤響應
                let error_msg = format!("❌ 按鈕操作失敗: {}", e);
                let error_response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(error_msg)
                );
                if let Err(e) = component.create_response(&ctx.http, error_response).await {
                    tracing::error!("發送錯誤響應失敗: {:?}", e);
                }

                Err(e)
            }
        }
    }

    /// 驗證 GUILD_MEMBERS intent 是否可用
    ///
    /// 這個函數通過嘗試獲取成員列表來驗證 GUILD_MEMBERS intent 是否正確配置
    /// 如果 intent 未啟用，Discord 會返回錯誤
    #[allow(dead_code)]
    async fn verify_guild_members_intent(&self, ctx: &Context, ready: &Ready) -> Result<()> {
        // 獲取第一個可用伺服器來測試 intent
        if let Some(guild_id) = ready.guilds.first() {
            match ctx.http.get_guild_members(guild_id.id, Some(1), Some(1)).await {
                Ok(members) => {
                    if !members.is_empty() {
                        tracing::info!("✅ GUILD_MEMBERS intent 驗證通過");
                        tracing::info!("成功獲取 {} 個成員資訊", members.len());
                    } else {
                        tracing::warn!("⚠️  GUILD_MEMBERS intent 可能可用，但伺服器沒有成員");
                    }
                }
                Err(e) => {
                    if e.to_string().contains("Missing Intents") || e.to_string().contains("Missing Access") {
                        tracing::error!("❌ GUILD_MEMBERS intent 未啟用或無權限");
                        tracing::error!("請在 Discord Developer Portal 中為此 Bot 啟用 GUILD_MEMBERS intent");
                        tracing::error!("步驟:");
                        tracing::error!("1. 前往 https://discord.com/developers/applications");
                        tracing::error!("2. 選擇您的應用程式");
                        tracing::error!("3. 進入 'Bot' 頁面");
                        tracing::error!("4. 在 'Privileged Gateway Intents' 下啟用 'SERVER MEMBERS INTENT'");
                        return Err(DiscordError::ConfigError(
                            "缺少必要的 GUILD_MEMBERS intent。請在 Discord Developer Portal 中啟用此 intent".to_string()
                        ));
                    } else {
                        tracing::warn!("⚠️  無法驗證 GUILD_MEMBERS intent: {}", e);
                        tracing::warn!("這可能是暫時性問題或權限問題");
                    }
                }
            }
        } else {
            tracing::warn!("⚠️  Bot 未加入任何伺服器，無法驗證 GUILD_MEMBERS intent");
            tracing::warn!("請將 Bot 邀請到測試伺服器進行驗證");
        }

        Ok(())
    }
}

impl DiscordGateway {
    pub fn new() -> Self {
        let config = Config::new_with_token("test_token".to_string())
            .unwrap_or_else(|_| Config::for_test());
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            token_valid: true,
            command_router: Arc::new(CommandRouter::new()),
            ui_factory: UIComponentFactory::new(),
            user_account_service: None,
        }
    }

    pub fn new_with_config(config: Config) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            token_valid: true,
            command_router: Arc::new(CommandRouter::new()),
            ui_factory: UIComponentFactory::new(),
            user_account_service: None,
        }
    }

    /// 配置 CommandRouter 並注入服務
    pub fn configure_command_router<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: FnOnce(CommandRouter) -> CommandRouter,
    {
        // 取得現有的 CommandRouter
        let current_router = Arc::try_unwrap(Arc::clone(&self.command_router))
            .unwrap_or_else(|_| CommandRouter::new());

        // 應用配置函數
        let configured_router = config_fn(current_router);

        // 更新 command_router
        self.command_router = Arc::new(configured_router);
        self
    }

    pub async fn connect(&mut self) -> Result<()> {
        if !self.token_valid {
            *self.status.lock().await = ConnectionStatus::Error;
            log_connection_error("Invalid Discord token");
            return Err(DiscordError::InvalidToken);
        }

        *self.status.lock().await = ConnectionStatus::Connecting;
        tracing::info!("正在嘗試連接到 Discord Gateway...");

        let handler = Handler {
            status: Arc::clone(&self.status),
            ui_factory: Arc::new(self.ui_factory.clone()),
            command_router: Arc::clone(&self.command_router),
            user_account_service: self.user_account_service.clone(),
        };

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_MEMBERS;

        let client = Client::builder(&self.config.discord_token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to create Discord client: {}", e);
                log_connection_error(&error_msg);
                DiscordError::ConnectionError(error_msg)
            })?;

        // 注意：intent 驗證將在 Ready 事件中進行

        // 儲存客戶端但不立即啟動（讓 main.rs 控制啟動）
        *self.client.lock().await = Some(client);
        tracing::info!("✅ Discord 客戶端創建成功，準備啟動");
        Ok(())
    }

    /// 設置 UserAccountService 用於自動帳戶創建
    pub fn with_user_account_service(mut self, user_account_service: Arc<crate::services::UserAccountService>) -> Self {
        // 將服務存儲以便後續使用
        self.user_account_service = Some(user_account_service);
        self
    }

    /// 啟動 Discord 客戶端並開始監聽事件
    pub async fn start(&mut self) -> Result<()> {
        let mut client_guard = self.client.lock().await;
        if let Some(mut client) = client_guard.take() {
            tracing::info!("正在啟動 Discord 客戶端...");
            *self.status.lock().await = ConnectionStatus::Connecting;

            match client.start().await {
                Ok(_) => {
                    tracing::info!("✅ Discord 客戶端啟動成功");
                    *self.status.lock().await = ConnectionStatus::Connected;
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to start Discord client: {}", e);
                    log_connection_error(&error_msg);
                    *self.status.lock().await = ConnectionStatus::Error;
                    Err(DiscordError::ConnectionError(error_msg))
                }
            }
        } else {
            Err(DiscordError::ConnectionError("No Discord client available".to_string()))
        }
    }

    /// 優雅關閉 Discord 客戶端
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("正在關閉 Discord 客戶端...");
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // 清除客戶端（Serenity 會在 drop 時自動清理）
        *self.client.lock().await = None;
        tracing::info!("✅ Discord 客戶端已關閉");
        Ok(())
    }

    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.lock().await.clone()
    }

    pub fn set_invalid_token(&mut self) {
        self.token_valid = false;
        self.config.discord_token = "invalid_token".to_string();
    }

    pub async fn handle_command(&self, command: &str) -> Result<String> {
        let start_time = std::time::Instant::now();
        log_command_received(command);

        let response = if command == "!ping" {
            Ok("Pong!".to_string())
        } else {
            let command_result = self.command_router.parse_command(command).await?;
            self.command_router.route_command(&command_result).await
        };

        let elapsed = start_time.elapsed().as_millis() as u64;
        log_command_processed(command, elapsed);

        response
    }

    pub async fn simulate_message_event(&self) -> bool {
        log_event_received("message");

        match *self.status.lock().await {
            ConnectionStatus::Connected => true,
            _ => false,
        }
    }
}