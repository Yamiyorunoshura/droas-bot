use crate::error::{DiscordError, Result};
use crate::services::{BalanceService, TransferService, MessageService, TransactionService, HelpService, AdminService, AdminAuditService};
use super::command_parser::{Command, CommandResult};
use std::sync::Arc;
use std::str::FromStr;
use tracing::{info, error};
use bigdecimal::BigDecimal;

pub struct ServiceRouter {
    balance_service: Option<Arc<BalanceService>>,
    transfer_service: Option<Arc<TransferService>>,
    transaction_service: Option<Arc<TransactionService>>,
    message_service: Arc<MessageService>,
    help_service: Option<Arc<HelpService>>,
    admin_service: Option<Arc<AdminService>>,
    admin_audit_service: Option<Arc<AdminAuditService>>,
}

impl ServiceRouter {
    pub fn new() -> Self {
        Self {
            balance_service: None,
            transfer_service: None,
            transaction_service: None,
            message_service: Arc::new(MessageService::new()),
            help_service: None,
            admin_service: None,
            admin_audit_service: None,
        }
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
        self.message_service = message_service;
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

    pub async fn route_command(&self, command_result: &CommandResult) -> Result<String> {
        match &command_result.command {
            Command::Balance => {
                self.handle_balance_command(command_result).await
            },
            Command::Transfer => {
                self.handle_transfer_command(command_result).await
            },
            Command::Help => {
                self.handle_help_command(command_result).await
            },
            Command::History => {
                self.handle_history_command(command_result).await
            },
            Command::AdjustBalance => {
                self.handle_adjust_balance_command(command_result).await
            },
            Command::AdminHistory => {
                self.handle_admin_history_command(command_result).await
            },
        }
    }

    /// 處理餘額查詢指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 響應結果
    async fn handle_balance_command(&self, command_result: &CommandResult) -> Result<String> {
        // 檢查是否有設置餘額服務
        let balance_service = self.balance_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("餘額服務未初始化".to_string()))?;

        // 檢查是否有用戶 ID
        let user_id = command_result.user_id
            .ok_or_else(|| DiscordError::InvalidCommand("缺少用戶 ID".to_string()))?
            as u64;

        let _username = command_result.username.as_deref()
            .unwrap_or("未知用戶");

        // 調用餘額服務
        match balance_service.get_balance(user_id).await {
            Ok(balance_response) => {
                // 使用消息服務格式化響應
                let message_response = self.message_service.format_balance_response(
                    balance_response.user_id,
                    &balance_response.username,
                    &balance_response.balance,
                    balance_response.created_at,
                )?;

                // 轉換為 Discord 字符串格式
                Ok(self.message_service.to_discord_string(&message_response))
            }
            Err(e) => {
                // 格式化錯誤響應
                let error_response = self.message_service.format_error_response(&e);
                Ok(self.message_service.to_discord_string(&error_response))
            }
        }
    }

    /// 處理幫助指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 幫助響應
    async fn handle_help_command(&self, command_result: &CommandResult) -> Result<String> {
        // 如果有設置幫助服務，使用完整的幫助功能
        if let Some(help_service) = &self.help_service {
            // 檢查是否有指定特定指令的參數
            if !command_result.args.is_empty() {
                let command_name = &command_result.args[0];
                match help_service.get_command_help(command_name).await {
                    Ok(help_content) => {
                        // 使用 MessageService 格式化特定指令的幫助
                        let message_response = self.message_service.format_detailed_help_response(&help_content);
                        return Ok(self.message_service.to_discord_string(&message_response));
                    },
                    Err(_) => {
                        // 如果找不到特定指令，返回通用幫助
                        let general_help = help_service.generate_help_content().await;
                        let message_response = self.message_service.format_detailed_help_response(&general_help);
                        return Ok(self.message_service.to_discord_string(&message_response));
                    }
                }
            } else {
                // 沒有參數，返回通用幫助
                let general_help = help_service.generate_help_content().await;
                let message_response = self.message_service.format_detailed_help_response(&general_help);
                return Ok(self.message_service.to_discord_string(&message_response));
            }
        }

        // 如果幫助服務未初始化，提供基本幫助信息
        let basic_help = self.generate_basic_help().await;
        let message_response = self.message_service.format_detailed_help_response(&basic_help);
        Ok(self.message_service.to_discord_string(&message_response))
    }

    /// 生成基本幫助信息（當幫助服務未初始化時使用）
    async fn generate_basic_help(&self) -> String {
        let mut help_content = String::new();

        help_content.push_str("## 🤖 DROAS 經濟機器人幫助\n\n");
        help_content.push_str("**可用指令**:\n\n");
        help_content.push_str("• `!balance` - 查詢帳戶餘額\n");
        help_content.push_str("• `!transfer <@用戶> <金額>` - 轉帳給指定用戶\n");
        help_content.push_str("• `!history` - 查看交易歷史記錄\n");
        help_content.push_str("• `!help` - 顯示此幫助信息\n\n");
        help_content.push_str("*使用 `!help <指令名稱>` 獲取特定指令的詳細幫助*");

        help_content
    }

    /// 處理轉帳指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 響應結果
    async fn handle_transfer_command(&self, command_result: &CommandResult) -> Result<String> {
        // 檢查是否有設置轉帳服務
        let transfer_service = self.transfer_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("轉帳服務未初始化".to_string()))?;

        // 檢查是否有用戶 ID
        let from_user_id = command_result.user_id
            .ok_or_else(|| DiscordError::InvalidCommand("缺少發送者用戶 ID".to_string()))?;

        // 驗證參數數量
        self.validate_transfer_args(&command_result.args)?;

        // 解析接收者和金額
        let recipient_str = &command_result.args[0];
        let amount_str = &command_result.args[1];

        // 解析接收者 ID（移除 @ 符號如果是用戶名）
        let to_user_id = if recipient_str.starts_with('@') {
            // 如果是用戶名格式，這裡需要實作用戶名到 ID 的映射
            // 目前簡化實作，假設輸入的是數字 ID
            recipient_str.trim_start_matches('@').parse::<i64>()
                .map_err(|_| DiscordError::InvalidCommand("無效的接收者 ID 格式".to_string()))?
        } else {
            recipient_str.parse::<i64>()
                .map_err(|_| DiscordError::InvalidCommand("無效的接收者 ID 格式".to_string()))?
        };

        // 執行轉帳
        match transfer_service.execute_transfer(from_user_id, to_user_id, amount_str).await {
            Ok(transfer_result) => {
                if transfer_result.success {
                    // 使用消息服務格式化成功響應
                    let message_response = self.message_service.format_transfer_success_response(&transfer_result)?;
                    Ok(self.message_service.to_discord_string(&message_response))
                } else {
                    // 格式化錯誤響應
                    let error_response = self.message_service.format_transfer_error_response(&transfer_result);
                    Ok(self.message_service.to_discord_string(&error_response))
                }
            }
            Err(e) => {
                // 格式化錯誤響應
                let error_response = self.message_service.format_error_response(&e);
                Ok(self.message_service.to_discord_string(&error_response))
            }
        }
    }

    /// 處理歷史查詢指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 響應結果
    async fn handle_history_command(&self, command_result: &CommandResult) -> Result<String> {
        // 檢查是否有設置交易服務
        let transaction_service = self.transaction_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("交易服務未初始化".to_string()))?;

        // 檢查是否有用戶 ID
        let user_id = command_result.user_id
            .ok_or_else(|| DiscordError::InvalidCommand("缺少用戶 ID".to_string()))?;

        // 解析限制參數（如果有的話）
        let limit = if !command_result.args.is_empty() {
            match command_result.args[0].parse::<i64>() {
                Ok(limit) => {
                    if limit <= 0 || limit > 100 {
                        return Err(DiscordError::InvalidCommand("限制數量必須在 1-100 之間".to_string()));
                    }
                    Some(limit)
                }
                Err(_) => return Err(DiscordError::InvalidCommand("無效的限制數量格式".to_string())),
            }
        } else {
            Some(10) // 預設顯示 10 筆記錄
        };

        // 調用交易服務查詢歷史記錄
        match transaction_service.get_user_transaction_history(user_id, limit).await {
            Ok(transactions) => {
                // 使用消息服務格式化歷史記錄響應
                let message_response = self.message_service.format_history_response(
                    user_id,
                    &transactions,
                )?;
                Ok(self.message_service.to_discord_string(&message_response))
            }
            Err(e) => {
                // 格式化錯誤響應
                let error_response = self.message_service.format_error_response(&e);
                Ok(self.message_service.to_discord_string(&error_response))
            }
        }
    }

    fn validate_transfer_args(&self, args: &[String]) -> Result<()> {
        if args.len() != 2 {
            return Err(DiscordError::InvalidCommand("Transfer command requires 2 arguments: @user amount".to_string()));
        }
        Ok(())
    }

    /// 處理管理員餘額調整指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 響應結果
    async fn handle_adjust_balance_command(&self, command_result: &CommandResult) -> Result<String> {
        // 檢查是否有設置管理員服務和審計服務
        let admin_service = self.admin_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("管理員服務未初始化".to_string()))?;

        let admin_audit_service = self.admin_audit_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("管理員審計服務未初始化".to_string()))?;

        // 檢查是否有管理員用戶 ID
        let admin_user_id = command_result.user_id
            .ok_or_else(|| DiscordError::InvalidCommand("缺少管理員用戶 ID".to_string()))?;

        // 驗證管理員權限 - 使用 Discord 權限檢查（如果可用）
        let has_permission = if let (Some(ctx), Some(guild_id)) =
            (&command_result.discord_context, command_result.guild_id) {
            // 使用 Discord 權限檢查
            admin_service.verify_admin_permission_with_discord(
                ctx.as_ref(),
                serenity::model::id::GuildId::new(guild_id as u64),
                serenity::model::id::UserId::new(admin_user_id as u64),
            ).await.unwrap_or(false)
        } else {
            // 回退到傳統的授權列表檢查
            admin_service.verify_admin_permission(admin_user_id).await.unwrap_or(false)
        };

        if !has_permission {
            let error_response = self.message_service.format_error_response(
                &DiscordError::PermissionDenied("您沒有執行此管理員操作的權限".to_string())
            );
            return Ok(self.message_service.to_discord_string(&error_response));
        }

        // 驗證參數數量
        self.validate_adjust_balance_args(&command_result.args)?;

        // 解析參數
        let target_user_str = &command_result.args[0];
        let amount_str = &command_result.args[1];
        let reason = command_result.args.get(2).cloned().unwrap_or_else(|| "管理員調整".to_string());

        // 解析目標用戶 ID
        let target_user_id = if target_user_str.starts_with('@') {
            target_user_str.trim_start_matches('@').parse::<i64>()
                .map_err(|_| DiscordError::InvalidCommand("無效的目標用戶 ID 格式".to_string()))?
        } else {
            target_user_str.parse::<i64>()
                .map_err(|_| DiscordError::InvalidCommand("無效的目標用戶 ID 格式".to_string()))?
        };

        // 解析金額
        let amount = BigDecimal::from_str(amount_str)
            .map_err(|_| DiscordError::InvalidAmount(format!("無效的金額格式：{}", amount_str)))?;

        // 創建管理員操作記錄
        let admin_operation = crate::services::admin_service::AdminOperation {
            operation_type: crate::services::admin_service::AdminOperationType::AdjustBalance,
            admin_user_id,
            target_user_id: Some(target_user_id),
            amount: Some(amount.clone()),
            reason: reason.clone(),
            timestamp: chrono::Utc::now(),
        };

        // 創建審計記錄
        let audit_record = crate::services::admin_audit_service::AdminAuditRecord {
            id: None,
            admin_id: admin_user_id,
            operation_type: format!("{:?}", admin_operation.operation_type),
            target_user_id: admin_operation.target_user_id,
            amount: admin_operation.amount.clone(),
            reason: admin_operation.reason.clone(),
            timestamp: admin_operation.timestamp,
            ip_address: None,
            user_agent: None,
        };

        // 記錄管理員操作到審計服務
        match admin_audit_service.log_admin_operation(audit_record).await {
            Ok(recorded_audit) => {
                info!("管理員操作審計記錄創建成功，ID: {:?}", recorded_audit.id);

                // 執行餘額調整操作
                match admin_service.coordinate_admin_operation(admin_operation).await {
                    Ok(operation_result) => {
                        if operation_result.success {
                            // 格式化成功響應 - 使用現有的餘額響應格式
                            let message_response = self.message_service.format_balance_response(
                                target_user_id as u64,
                                &format!("用戶 {}", target_user_id),
                                &amount,
                                Some(chrono::Utc::now()),
                            )?;
                            Ok(self.message_service.to_discord_string(&message_response))
                        } else {
                            // 格式化錯誤響應
                            let error_response = self.message_service.format_error_response(
                                &DiscordError::InvalidCommand(operation_result.message)
                            );
                            Ok(self.message_service.to_discord_string(&error_response))
                        }
                    }
                    Err(e) => {
                        // 格式化錯誤響應
                        let error_response = self.message_service.format_error_response(&e);
                        Ok(self.message_service.to_discord_string(&error_response))
                    }
                }
            }
            Err(e) => {
                error!("記錄管理員操作審計失敗：{}", e);
                let error_response = self.message_service.format_error_response(&e);
                Ok(self.message_service.to_discord_string(&error_response))
            }
        }
    }

    /// 處理管理員歷史查詢指令
    ///
    /// # Arguments
    /// * `command_result` - 命令結果
    ///
    /// # Returns
    /// * `Result<String>` - 響應結果
    async fn handle_admin_history_command(&self, command_result: &CommandResult) -> Result<String> {
        // 檢查是否有設置管理員服務和審計服務
        let admin_service = self.admin_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("管理員服務未初始化".to_string()))?;

        let admin_audit_service = self.admin_audit_service.as_ref()
            .ok_or_else(|| DiscordError::UnimplementedCommand("管理員審計服務未初始化".to_string()))?;

        // 檢查是否有管理員用戶 ID
        let admin_user_id = command_result.user_id
            .ok_or_else(|| DiscordError::InvalidCommand("缺少管理員用戶 ID".to_string()))?;

        // 驗證管理員權限 - 使用 Discord 權限檢查（如果可用）
        let has_permission = if let (Some(ctx), Some(guild_id)) =
            (&command_result.discord_context, command_result.guild_id) {
            // 使用 Discord 權限檢查
            admin_service.verify_admin_permission_with_discord(
                ctx.as_ref(),
                serenity::model::id::GuildId::new(guild_id as u64),
                serenity::model::id::UserId::new(admin_user_id as u64),
            ).await.unwrap_or(false)
        } else {
            // 回退到傳統的授權列表檢查
            admin_service.verify_admin_permission(admin_user_id).await.unwrap_or(false)
        };

        if !has_permission {
            let error_response = self.message_service.format_error_response(
                &DiscordError::PermissionDenied("您沒有執行此管理員操作的權限".to_string())
            );
            return Ok(self.message_service.to_discord_string(&error_response));
        }

        // 解析參數
        let limit = if !command_result.args.is_empty() {
            match command_result.args[0].parse::<i64>() {
                Ok(limit) => {
                    if limit <= 0 || limit > 100 {
                        return Err(DiscordError::InvalidCommand("限制數量必須在 1-100 之間".to_string()));
                    }
                    Some(limit)
                }
                Err(_) => return Err(DiscordError::InvalidCommand("無效的限制數量格式".to_string())),
            }
        } else {
            Some(20) // 預設顯示 20 筆記錄
        };

        // 查詢管理員操作歷史
        match admin_audit_service.get_admin_history(admin_user_id, limit).await {
            Ok(operations) => {
                // 格式化歷史記錄響應 - 使用現有的歷史響應格式
                // 需要將管理員操作轉換為交易記錄格式或創建簡化響應
                let mut response_text = format!("## 管理員操作歷史 (用戶 ID: {})\n\n", admin_user_id);

                if operations.is_empty() {
                    response_text.push_str("沒有找到管理員操作記錄。");
                } else {
                    response_text.push_str("**最近的操作記錄：**\n\n");

                    for (i, operation) in operations.iter().take(limit.unwrap_or(20) as usize).enumerate() {
                        response_text.push_str(&format!(
                            "{}. **{}** - 目標: {}, 金額: {:?}, 原因: {}, 時間: {}\n",
                            i + 1,
                            match operation.operation_type.as_str() {
                                "AdjustBalance" => "餘額調整",
                                "ViewUserInfo" => "查看用戶資訊",
                                "ViewHistory" => "查看歷史記錄",
                                "SystemMaintenance" => "系統維護",
                                _ => "其他操作",
                            },
                            operation.target_user_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()),
                            operation.amount,
                            operation.reason,
                            operation.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                        ));
                    }
                }

                // 使用消息服務格式化響應
                let message_response = self.message_service.format_text_response(&response_text);
                Ok(self.message_service.to_discord_string(&message_response))
            }
            Err(e) => {
                // 格式化錯誤響應
                let error_response = self.message_service.format_error_response(&e);
                Ok(self.message_service.to_discord_string(&error_response))
            }
        }
    }

    fn validate_adjust_balance_args(&self, args: &[String]) -> Result<()> {
        if args.len() < 2 || args.len() > 3 {
            return Err(DiscordError::InvalidCommand(
                "Adjust balance command requires 2-3 arguments: @user amount [reason]".to_string()
            ));
        }
        Ok(())
    }
}

impl Default for ServiceRouter {
    fn default() -> Self {
        Self::new()
    }
}