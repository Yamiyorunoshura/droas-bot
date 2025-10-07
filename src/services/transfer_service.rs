// Transfer Service - N2 計劃安全整合版本 (GREEN 階段)
// 實現點對點轉帳功能，包含全面的安全驗證 (NFR-S-001, NFR-S-002)

use crate::database::{
    user_repository::{User, UserRepositoryTrait},
    transaction_repository::{CreateTransactionRequest, TransactionRepositoryTrait},
};
use crate::error::{DiscordError, Result};
use crate::services::{security_service::SecurityService, validation_pattern::{ValidatorFactory, TransferInput, Validator}, transfer_validation_service::TransferValidationService, message_service::MessageService};
use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::sync::Arc;
use serenity::builder::CreateMessage;
use serenity::model::id::UserId;
use tracing::{info, error, debug, instrument};

/// 轉帳結果
#[derive(Debug, Clone)]
pub struct TransferResult {
    /// 轉帳是否成功
    pub success: bool,
    /// 交易 ID（如果成功）
    pub transaction_id: Option<String>,
    /// 發送方用戶資訊
    pub from_user: User,
    /// 接收方用戶資訊
    pub to_user: User,
    /// 轉帳金額
    pub amount: BigDecimal,
    /// 操作結果訊息（用於 Discord 回覆）
    pub message: String,
}

/// Transfer Service
///
/// 負責處理點對點轉帳交易
/// 整合 Security Service、Transfer Validation Service 和統一驗證模式實現全面的安全驗證 (NFR-S-001, NFR-S-002)
pub struct TransferService {
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
    transaction_repository: Arc<dyn TransactionRepositoryTrait + Send + Sync>,
    security_service: Arc<SecurityService>,
    validator_factory: Arc<ValidatorFactory>,
    transfer_validation_service: Arc<TransferValidationService>,
    message_service: MessageService,
}

impl TransferService {
    /// 創建新的 Transfer Service 實例
    ///
    /// # Arguments
    ///
    /// * `user_repository` - 用戶資料庫倉儲
    /// * `transaction_repository` - 交易資料庫倉儲
    /// * `security_service` - 安全驗證服務
    pub fn new<T, U>(
        user_repository: T,
        transaction_repository: U,
        security_service: SecurityService,
    ) -> crate::error::Result<Self>
    where
        T: UserRepositoryTrait + Send + Sync + 'static,
        U: TransactionRepositoryTrait + Send + Sync + 'static,
    {
        let security_service = Arc::new(security_service);
        let validator_factory = Arc::new(ValidatorFactory::new(security_service.clone()));
        let transfer_validation_service = Arc::new(TransferValidationService::new());
        let message_service = MessageService::new();

        Ok(Self {
            user_repository: Arc::new(user_repository),
            transaction_repository: Arc::new(transaction_repository),
            security_service,
            validator_factory,
            transfer_validation_service,
            message_service,
        })
    }

    /// 執行轉帳交易（包含完整的安全驗證）
    ///
    /// # Arguments
    ///
    /// * `from_user_id` - 發送方 Discord 用戶 ID
    /// * `to_user_id` - 接收方 Discord 用戶 ID
    /// * `amount_str` - 轉帳金額字符串
    ///
    /// # Returns
    ///
    /// 返回 `TransferResult` 包含轉帳結果
    pub async fn execute_transfer(
        &self,
        from_user_id: i64,
        to_user_id: i64,
        amount_str: &str,
    ) -> Result<TransferResult> {
        debug!("開始處理轉帳請求：{} -> {}, 金額：{}", from_user_id, to_user_id, amount_str);

        // 獲取用戶資訊（需要在驗證之前獲取，以便在錯誤情況下返回）
        let from_user = match self.security_service.authenticate_user(from_user_id).await {
            Ok(user) => user,
            Err(error) => {
                return Ok(TransferResult {
                    success: false,
                    transaction_id: None,
                    from_user: User { // 創建一個基本的用戶對象
                        discord_user_id: from_user_id,
                        username: "Unknown".to_string(),
                        balance: BigDecimal::from(0),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    to_user: User {
                        discord_user_id: to_user_id,
                        username: "Unknown".to_string(),
                        balance: BigDecimal::from(0),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    amount: BigDecimal::from(0),
                    message: format!("發送方用戶驗證失敗：{}", error),
                });
            }
        };

        let to_user = match self.security_service.authenticate_user(to_user_id).await {
            Ok(user) => user,
            Err(_) => {
                return Ok(TransferResult {
                    success: false,
                    transaction_id: None,
                    from_user,
                    to_user: User {
                        discord_user_id: to_user_id,
                        username: "Unknown".to_string(),
                        balance: BigDecimal::from(0),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    amount: BigDecimal::from(0),
                    message: "接收方用戶不存在".to_string(),
                });
            }
        };

        // 驗證和清理金額輸入
        let amount = match self.security_service.validate_amount(amount_str) {
            Ok(amount_f64) => BigDecimal::from_str(&amount_f64.to_string()).unwrap(),
            Err(error) => {
                return Ok(TransferResult {
                    success: false,
                    transaction_id: None,
                    from_user,
                    to_user,
                    amount: BigDecimal::from(0),
                    message: format!("金額驗證失敗：{}", error),
                });
            }
        };

        // 使用統一的驗證模式進行輸入驗證
        let transfer_input = TransferInput {
            from_user_id,
            to_user_id,
            amount: amount_str.to_string(),
        };

        if let Err(error) = self.validator_factory.create_transfer_validator().validate(&transfer_input) {
            return Ok(TransferResult {
                success: false,
                transaction_id: None,
                from_user,
                to_user,
                amount,
                message: format!("輸入驗證失敗：{}", error),
            });
        }

        // 使用 Transfer Validation Service 進行全面的轉帳驗證
        if let Err(validation_error) = self.transfer_validation_service.validate_transfer(&from_user, &to_user, &amount) {
            let message = match validation_error {
                crate::services::transfer_validation_service::ValidationError::InsufficientBalance { message } => message,
                crate::services::transfer_validation_service::ValidationError::SelfTransfer { message } => message,
                crate::services::transfer_validation_service::ValidationError::InvalidAmount { message } => message,
                crate::services::transfer_validation_service::ValidationError::AmountExceedsLimit { message, .. } => message,
            };

            return Ok(TransferResult {
                success: false,
                transaction_id: None,
                from_user,
                to_user,
                amount,
                message,
            });
        }

        // 執行轉帳交易，捕獲錯誤並轉換為 TransferResult
        match self.execute_transfer_transaction(from_user.clone(), to_user.clone(), amount.clone()).await {
            Ok(result) => Ok(result),
            Err(error) => {
                let error_message = match &error {
                    DiscordError::ValidationError(msg) => msg.clone(),
                    DiscordError::InsufficientBalance(user_id) => format!("用戶 {} 餘額不足", user_id),
                    DiscordError::UserNotFound(msg) => msg.clone(),
                    DiscordError::InvalidAmount(msg) => msg.clone(),
                    _ => format!("轉帳失敗：{}", error),
                };

                Ok(TransferResult {
                    success: false,
                    transaction_id: None,
                    from_user,
                    to_user,
                    amount,
                    message: error_message,
                })
            }
        }
    }

    /// 執行實際的轉帳交易
    ///
    /// # Arguments
    ///
    /// * `from_user` - 發送方用戶
    /// * `to_user` - 接收方用戶
    /// * `amount` - 轉帳金額
    ///
    /// # Returns
    ///
    /// 返回 `TransferResult` 包含轉帳結果
    async fn execute_transfer_transaction(
        &self,
        from_user: User,
        to_user: User,
        amount: BigDecimal,
    ) -> Result<TransferResult> {
        debug!("執行轉帳交易：{} -> {}, 金額：{}", from_user.discord_user_id, to_user.discord_user_id, amount);

        // 更新發送方餘額
        let new_from_balance = &from_user.balance - &amount;
        self.user_repository.update_balance(from_user.discord_user_id, &new_from_balance).await
            .map_err(|e| {
                error!("更新發送方餘額失敗：{}", e);
                DiscordError::DatabaseQueryError(format!("更新餘額失敗：{}", e))
            })?;

        // 更新接收方餘額
        let new_to_balance = &to_user.balance + &amount;
        self.user_repository.update_balance(to_user.discord_user_id, &new_to_balance).await
            .map_err(|e| {
                error!("更新接收方餘額失敗：{}", e);
                DiscordError::DatabaseQueryError(format!("更新餘額失敗：{}", e))
            })?;

        // 創建交易記錄
        let transaction_request = CreateTransactionRequest {
            from_user_id: Some(from_user.discord_user_id),
            to_user_id: Some(to_user.discord_user_id),
            amount: amount.clone(),
            transaction_type: "transfer".to_string(),
            metadata: None,
        };

        let transaction = self.transaction_repository.create_transaction(transaction_request).await
            .map_err(|e| {
                error!("創建交易記錄失敗：{}", e);
                DiscordError::DatabaseQueryError(format!("創建交易記錄失敗：{}", e))
            })?;

        info!("轉帳交易成功：{} -> {}, 金額：{}, 交易ID：{}",
              from_user.discord_user_id, to_user.discord_user_id, amount, transaction.id);

        // 更新用戶資訊中的餘額
        let mut updated_from_user = from_user.clone();
        updated_from_user.balance = new_from_balance;

        let mut updated_to_user = to_user.clone();
        updated_to_user.balance = new_to_balance;

        let amount_str = amount.to_string();
        Ok(TransferResult {
            success: true,
            transaction_id: Some(transaction.id.to_string()),
            from_user: updated_from_user,
            to_user: updated_to_user,
            amount,
            message: format!("成功轉帳 {} 幣給 {}！交易ID：{}", amount_str, to_user.username, transaction.id),
        })
    }

    /// 查詢用戶轉帳歷史
    ///
    /// # Arguments
    ///
    /// * `user_id` - Discord 用戶 ID
    /// * `limit` - 查詢記錄數量限制
    ///
    /// # Returns
    ///
    /// 返回交易記錄列表
    pub async fn get_transfer_history(&self, user_id: i64, limit: Option<i64>) -> crate::error::Result<Vec<crate::database::transaction_repository::Transaction>> {
        debug!("查詢用戶 {} 的轉帳歷史，限制：{}", user_id, limit.unwrap_or(10));

        // NFR-S-001: 驗證用戶 ID
        self.security_service.validate_discord_user_id(user_id)?;

        // 驗證用戶存在
        let _user = self.security_service.authenticate_user(user_id).await?;

        // 查詢交易記錄
        self.transaction_repository.get_user_transactions(user_id, limit, Some(0)).await
            .map_err(|e| {
                error!("查詢轉帳歷史失敗：{}", e);
                DiscordError::DatabaseQueryError(format!("查詢轉帳歷史失敗：{}", e))
            })
    }

    /// 驗證轉帳請求參數
    ///
    /// # Arguments
    ///
    /// * `from_user_id` - 發送方 Discord 用戶 ID
    /// * `to_user_id` - 接收方 Discord 用戶 ID
    /// * `amount_str` - 轉帳金額字符串
    ///
    /// # Returns
    ///
    /// 返回驗證結果和解析後的金額
    pub async fn validate_transfer_request(
        &self,
        from_user_id: i64,
        to_user_id: i64,
        amount_str: &str,
    ) -> crate::error::Result<(User, User, f64)> {
        debug!("驗證轉帳請求：{} -> {}, 金額：{}", from_user_id, to_user_id, amount_str);

        // 使用統一的驗證模式進行輸入驗證 (REFACTOR 階段優化)
        let transfer_input = TransferInput {
            from_user_id,
            to_user_id,
            amount: amount_str.to_string(),
        };

        let transfer_validator = self.validator_factory.create_transfer_validator();
        transfer_validator.validate(&transfer_input)?;

        // NFR-S-002: 驗證和清理金額輸入
        let amount = self.security_service.validate_amount(amount_str)?;

        // 驗證用戶存在
        let from_user = self.security_service.authenticate_user(from_user_id).await?;
        let to_user = self.security_service.authenticate_user(to_user_id).await
            .map_err(|_| DiscordError::UserNotFound("接收方用戶不存在".to_string()))?;

        debug!("轉帳請求驗證通過");
        Ok((from_user, to_user, amount))
    }

    /// 創建轉帳確認的 embed 消息
    ///
    /// # Arguments
    /// * `from_user_id` - 發送方 Discord 用戶 ID
    /// * `to_user_id` - 接收方 Discord 用戶 ID
    /// * `amount` - 轉帳金額
    ///
    /// # Returns
    /// * `Ok(CreateMessage)` - 轉帳確認的 embed 消息
    /// * `Err(DiscordError)` - 創建失敗
    #[instrument(skip(self), fields(from_user_id = %from_user_id, to_user_id = %to_user_id, amount = %amount))]
    pub async fn create_transfer_confirmation_embed(
        &self,
        from_user_id: UserId,
        to_user_id: UserId,
        amount: f64,
    ) -> Result<CreateMessage> {
        info!("Creating transfer confirmation embed: {} -> {}, amount: {}", from_user_id, to_user_id, amount);

        // 使用 MessageService 創建轉帳 embed，包含按鈕
        let embed = self.message_service.create_transfer_embed(from_user_id, to_user_id, amount).await;

        info!("Successfully created transfer confirmation embed");
        Ok(embed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{create_user_pool, UserRepository, TransactionRepository};
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_transfer_service_creation() {
        // 測試 TransferService 創建
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = UserRepository::new(pool.clone());
            let transaction_repo = TransactionRepository::new(pool);
            let security_service = SecurityService::new(user_repo.clone()).unwrap();

            let service = TransferService::new(user_repo, transaction_repo, security_service);
            assert!(service.is_ok(), "TransferService 創建應該成功");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }

    #[tokio::test]
    async fn test_validate_transfer_request() {
        // 測試轉帳請求驗證
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = UserRepository::new(pool.clone());
            let transaction_repo = TransactionRepository::new(pool);
            let security_service = SecurityService::new(user_repo.clone()).unwrap();

            let service = TransferService::new(user_repo, transaction_repo, security_service).unwrap();

            // 測試有效的轉帳請求
            let result = service.validate_transfer_request(123, 456, "100.50").await;

            // 預期會失敗，因為用戶不存在，但安全驗證應該通過
            match result {
                Err(DiscordError::UserNotFound(_)) => {
                    // 這是預期的行為
                    assert!(true, "安全驗證通過，但用戶不存在，這是預期的");
                }
                Err(_) => {
                    panic!("安全驗證失敗，但應該通過");
                }
                Ok(_) => {
                    // 如果成功，說明用戶存在，這也是可能的
                    assert!(true, "轉帳請求驗證成功");
                }
            }
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }
}