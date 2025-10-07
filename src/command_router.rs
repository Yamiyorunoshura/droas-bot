use crate::error::{DiscordError, Result};
use crate::discord_gateway::{
    CommandParser, ServiceRouter, CommandRegistry,
    RouterErrorHandler, RouterMetrics, OperationTimer,
    Command, CommandResult
};
use crate::services::{UserAccountService, BalanceService, MessageService, TransferService, TransactionService, HelpService, AdminService, AdminAuditService};
use std::sync::Arc;
use tokio::sync::Mutex;

// Re-export for backward compatibility
pub use crate::discord_gateway::Command as PublicCommand;
pub use crate::discord_gateway::CommandResult as PublicCommandResult;

pub struct CommandRouter {
    parser: CommandParser,
    command_registry: CommandRegistry,
    error_handler: RouterErrorHandler,
    metrics: Arc<Mutex<RouterMetrics>>,
    user_account_service: Option<Arc<UserAccountService>>,
    balance_service: Option<Arc<BalanceService>>,
    transfer_service: Option<Arc<TransferService>>,
    transaction_service: Option<Arc<TransactionService>>,
    message_service: Option<Arc<MessageService>>,
    help_service: Option<Arc<HelpService>>,
    admin_service: Option<Arc<AdminService>>,
    admin_audit_service: Option<Arc<AdminAuditService>>,
}

impl CommandRouter {
    pub fn new() -> Self {
        Self::with_prefix("!".to_string())
    }

    pub fn with_prefix(prefix: String) -> Self {
        Self {
            parser: CommandParser::with_prefix(prefix),
            command_registry: CommandRegistry::new(),
            error_handler: RouterErrorHandler::new(),
            metrics: Arc::new(Mutex::new(RouterMetrics::new())),
            user_account_service: None,
            balance_service: None,
            transfer_service: None,
            transaction_service: None,
            message_service: None,
            help_service: None,
            admin_service: None,
            admin_audit_service: None,
        }
    }

    /// 設置用戶帳戶服務
    ///
    /// # Arguments
    ///
    /// * `user_account_service` - 用戶帳戶服務實例
    pub fn with_user_account_service(mut self, user_account_service: Arc<UserAccountService>) -> Self {
        self.user_account_service = Some(user_account_service);
        self
    }

    /// 設置餘額服務
    ///
    /// # Arguments
    ///
    /// * `balance_service` - 餘額服務實例
    pub fn with_balance_service(mut self, balance_service: Arc<BalanceService>) -> Self {
        self.balance_service = Some(balance_service);
        self
    }

    /// 設置轉帳服務
    ///
    /// # Arguments
    ///
    /// * `transfer_service` - 轉帳服務實例
    pub fn with_transfer_service(mut self, transfer_service: Arc<TransferService>) -> Self {
        self.transfer_service = Some(transfer_service);
        self
    }

    /// 設置交易服務
    ///
    /// # Arguments
    ///
    /// * `transaction_service` - 交易服務實例
    pub fn with_transaction_service(mut self, transaction_service: Arc<TransactionService>) -> Self {
        self.transaction_service = Some(transaction_service);
        self
    }

    /// 設置消息服務
    ///
    /// # Arguments
    ///
    /// * `message_service` - 消息服務實例
    pub fn with_message_service(mut self, message_service: Arc<MessageService>) -> Self {
        self.message_service = Some(message_service);
        self
    }

    /// 設置幫助服務
    ///
    /// # Arguments
    ///
    /// * `help_service` - 幫助服務實例
    pub fn with_help_service(mut self, help_service: Arc<HelpService>) -> Self {
        self.help_service = Some(help_service);
        self
    }

    /// 設置管理員服務
    ///
    /// # Arguments
    ///
    /// * `admin_service` - 管理員服務實例
    pub fn with_admin_service(mut self, admin_service: Arc<AdminService>) -> Self {
        self.admin_service = Some(admin_service);
        self
    }

    /// 設置管理員審計服務
    ///
    /// # Arguments
    ///
    /// * `admin_audit_service` - 管理員審計服務實例
    pub fn with_admin_audit_service(mut self, admin_audit_service: Arc<AdminAuditService>) -> Self {
        self.admin_audit_service = Some(admin_audit_service);
        self
    }

    /// 構建最終的 ServiceRouter（配置所有服務）
    fn build_service_router(&self) -> ServiceRouter {
        let mut service_router = ServiceRouter::new();

        // 設置餘額服務
        if let Some(ref balance_service) = self.balance_service {
            service_router = service_router.with_balance_service(Arc::clone(balance_service));
        }

        // 設置轉帳服務
        if let Some(ref transfer_service) = self.transfer_service {
            service_router = service_router.with_transfer_service(Arc::clone(transfer_service));
        }

        // 設置交易服務
        if let Some(ref transaction_service) = self.transaction_service {
            service_router = service_router.with_transaction_service(Arc::clone(transaction_service));
        }

        // 設置消息服務
        if let Some(ref message_service) = self.message_service {
            service_router = service_router.with_message_service(Arc::clone(message_service));
        }

        // 設置幫助服務
        if let Some(ref help_service) = self.help_service {
            service_router = service_router.with_help_service(Arc::clone(help_service));
        }

        // 設置管理員服務
        if let Some(ref admin_service) = self.admin_service {
            service_router = service_router.with_admin_service(Arc::clone(admin_service));
        }

        // 設置管理員審計服務
        if let Some(ref admin_audit_service) = self.admin_audit_service {
            service_router = service_router.with_admin_audit_service(Arc::clone(admin_audit_service));
        }

        service_router
    }

    pub async fn parse_command(&self, input: &str) -> Result<CommandResult> {
        let timer = OperationTimer::new();
        let result = self.parser.parse_command(input).await;

        let is_error = result.is_err();
        let elapsed = timer.elapsed();

        // Record metrics (extract command name from result or use "unknown" for errors)
        let command_name = if let Ok(ref cmd_result) = result {
            format!("{:?}", cmd_result.command)
        } else {
            "unknown".to_string()
        };

        if let Ok(mut metrics) = self.metrics.try_lock() {
            metrics.record_command_execution(&command_name, elapsed, is_error);
        }

        result
    }

    pub async fn route_command(&self, command_result: &CommandResult) -> Result<String> {
        let timer = OperationTimer::new();
        let command_name = format!("{:?}", command_result.command);

        // 自動帳戶創建檢查 - 如果有用戶帳戶服務且命令需要用戶帳戶
        if let Some(ref user_service) = self.user_account_service {
            if self.requires_user_account(&command_result.command) {
                if let (Some(discord_user_id), Some(username)) = (command_result.user_id, &command_result.username) {
                    match user_service.create_or_get_user_account(discord_user_id, username.clone()).await {
                        Ok(account_result) => {
                            // 如果是新創建的帳戶，可以記錄日誌或發送歡迎訊息
                            if account_result.was_created {
                                tracing::info!("為用戶 {} 自動創建帳戶成功", discord_user_id);
                            }
                        },
                        Err(e) => {
                            tracing::error!("自動帳戶創建失敗：{}", e);
                            // 根據錯誤類型決定是否繼續執行命令
                            return Err(e);
                        }
                    }
                }
            }
        }

        let service_router = self.build_service_router();
        let result = service_router.route_command(command_result).await;

        let is_error = result.is_err();
        let elapsed = timer.elapsed();

        // Record metrics
        if let Ok(mut metrics) = self.metrics.try_lock() {
            metrics.record_command_execution(&command_name, elapsed, is_error);
        }

        // If there's an error, format it using the error handler
        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                // Use error handler to format the error message but preserve the original error type
                let formatted_error = self.error_handler.handle_error(&e);
                match e {
                    DiscordError::UnimplementedCommand(_) => Err(DiscordError::UnimplementedCommand(formatted_error)),
                    DiscordError::UnknownCommand(_) => Err(DiscordError::UnknownCommand(formatted_error)),
                    _ => Err(DiscordError::InvalidCommand(formatted_error)),
                }
            }
        }
    }

    /// 檢查命令是否需要用戶帳戶
    ///
    /// # Arguments
    ///
    /// * `command` - 要檢查的命令
    ///
    /// # Returns
    ///
    /// 返回 true 如果命令需要用戶帳戶
    fn requires_user_account(&self, command: &Command) -> bool {
        match command {
            Command::Balance => true,
            Command::Transfer => true,
            Command::History => true,
            Command::AdjustBalance => true,  // 管理員命令也需要帳戶驗證
            Command::AdminHistory => true,   // 管理員命令也需要帳戶驗證
            // 可以根據需要添加更多命令
            _ => false,
        }
    }

    pub fn get_available_commands(&self) -> Vec<String> {
        self.parser.get_available_commands()
    }

    pub fn is_command_supported(&self, command: &str) -> bool {
        self.parser.is_command_supported(command)
    }

    pub fn get_prefix(&self) -> &str {
        self.parser.get_prefix()
    }

    pub async fn get_metrics_snapshot(&self) -> crate::discord_gateway::MetricsSnapshot {
        if let Ok(metrics) = self.metrics.try_lock() {
            metrics.get_metrics_snapshot()
        } else {
            // Return empty metrics if lock can't be acquired
            crate::discord_gateway::MetricsSnapshot {
                command_counts: std::collections::HashMap::new(),
                average_response_times: std::collections::HashMap::new(),
                total_requests: 0,
                error_rate: 0.0,
                uptime: std::time::Duration::from_secs(0),
            }
        }
    }

    /// 獲取命令註冊表的幫助文本
    pub fn get_help_text(&self) -> String {
        self.command_registry.get_help_text()
    }

    /// 獲取特定命令的描述
    pub fn get_command_description(&self, command: &str) -> Option<String> {
        self.command_registry.get_description(command).cloned()
    }

    pub async fn is_within_sla(&self, command: &str, max_response_time: std::time::Duration) -> bool {
        if let Ok(metrics) = self.metrics.try_lock() {
            metrics.is_within_sla(command, max_response_time)
        } else {
            true // Assume within SLA if we can't check
        }
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}