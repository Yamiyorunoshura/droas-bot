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
}

struct Handler {
    status: Arc<Mutex<ConnectionStatus>>,
    ui_factory: Arc<UIComponentFactory>,
    command_router: Arc<CommandRouter>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, _ready: Ready) {
        log_connection_success();
        *self.status.lock().await = ConnectionStatus::Connected;
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
        };

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::DIRECT_MESSAGES;

        let client = Client::builder(&self.config.discord_token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to create Discord client: {}", e);
                log_connection_error(&error_msg);
                DiscordError::ConnectionError(error_msg)
            })?;

        // 儲存客戶端但不立即啟動（讓 main.rs 控制啟動）
        *self.client.lock().await = Some(client);
        tracing::info!("✅ Discord 客戶端創建成功，準備啟動");
        Ok(())
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