// User Account Service - N2 計劃安全整合版本 (GREEN 階段)
// 實現自動帳戶創建功能，包含安全驗證整合 (NFR-S-001, NFR-S-002)

use crate::database::{UserRepository};
use crate::database::user_repository::CreateUserRequest;
use crate::error::{DiscordError, Result};
use crate::services::security_service::SecurityService;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, error, debug};
use chrono::{DateTime, Utc};

/// 快取過期時間（秒）
const CACHE_EXPIRY_SECONDS: u64 = 300; // 5 分鐘

/// User Account Service
///
/// 負責管理用戶帳戶創建、驗證和相關操作
/// 自動為新 Discord 用戶創建經濟帳戶
/// 包含用戶存在檢查的快取機制，提高響應速度
/// 整合 Security Service 實現全面的安全驗證 (NFR-S-001, NFR-S-002)
pub struct UserAccountService {
    user_repository: UserRepository,
    user_exists_cache: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<i64, (bool, DateTime<Utc>)>>>,
    security_service: Arc<SecurityService>,
}

impl UserAccountService {
    /// 創建新的 User Account Service 實例
    ///
    /// # Arguments
    ///
    /// * `user_repository` - 用戶資料庫倉儲
    pub fn new(user_repository: UserRepository) -> crate::error::Result<Self> {
        let security_service = SecurityService::new(user_repository.clone())?;
        Ok(Self {
            user_repository,
            user_exists_cache: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            security_service: Arc::new(security_service),
        })
    }

    /// 創建新的 User Account Service 實例（使用現有的 Security Service）
    ///
    /// # Arguments
    ///
    /// * `user_repository` - 用戶資料庫倉儲
    /// * `security_service` - 安全驗證服務
    pub fn new_with_security(user_repository: UserRepository, security_service: SecurityService) -> Self {
        Self {
            user_repository,
            user_exists_cache: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            security_service: Arc::new(security_service),
        }
    }

    /// 為新用戶創建帳戶（整合安全驗證）
    ///
    /// 如果用戶不存在，自動創建帳戶並設置初始餘額為 1000
    /// 如果用戶已存在，返回現有用戶資訊
    /// 包含完整的安全驗證流程 (NFR-S-001, NFR-S-002)
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `username` - Discord 用戶名稱
    ///
    /// # Returns
    ///
    /// 返回 `AccountCreationResult` 包含創建結果和用戶資訊
    pub async fn create_or_get_user_account(&self, discord_user_id: i64, username: String) -> Result<AccountCreationResult> {
        debug!("檢查用戶帳戶狀態：Discord ID={}, Username={}", discord_user_id, username);

        // NFR-S-001: 驗證 Discord 用戶 ID
        self.security_service.validate_discord_user_id(discord_user_id)?;

        // NFR-S-002: 驗證和清理用戶名稱
        let sanitized_username = self.security_service.sanitize_string_input(&username, 32)?;
        self.security_service.validate_username(&sanitized_username)?;

        // 檢查用戶是否已存在
        match self.check_user_exists(discord_user_id).await {
            Ok(true) => {
                info!("用戶 {} 已存在，返回現有帳戶", discord_user_id);
                match self.user_repository.get_user_by_discord_id(discord_user_id).await {
                    Ok(Some(user)) => {
                        Ok(AccountCreationResult {
                            success: true,
                            was_created: false,
                            user,
                            message: "帳戶已存在".to_string(),
                        })
                    },
                    Ok(None) => {
                        error!("用戶存在檢查通過但無法獲取用戶資訊：{}", discord_user_id);
                        Err(DiscordError::UserNotFound(discord_user_id.to_string()))
                    },
                    Err(e) => {
                        error!("獲取用戶資訊時發生錯誤：{}", e);
                        Err(e)
                    }
                }
            },
            Ok(false) => {
                info!("新用戶 {}，開始創建帳戶", discord_user_id);
                self.create_user_account(discord_user_id, sanitized_username).await
            },
            Err(e) => {
                error!("檢查用戶是否存在時發生錯誤：{}", e);
                Err(e)
            }
        }
    }

    /// 創建新的用戶帳戶
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `username` - Discord 用戶名稱
    ///
    /// # Returns
    ///
    /// 返回 `AccountCreationResult` 包含創建結果
    async fn create_user_account(&self, discord_user_id: i64, username: String) -> Result<AccountCreationResult> {
        debug!("創建新用戶帳戶：Discord ID={}, Username={}", discord_user_id, username);

        // 設置初始餘額
        let initial_balance = self.initialize_balance().await;

        let create_request = CreateUserRequest {
            discord_user_id,
            username: username.clone(),
            initial_balance: Some(initial_balance.clone()),
        };

        match self.user_repository.create_user(create_request).await {
            Ok(user) => {
                info!("用戶帳戶創建成功：{}, 餘額：{}", user.discord_user_id, user.balance);
                Ok(AccountCreationResult {
                    success: true,
                    was_created: true,
                    user,
                    message: format!("歡迎 {}！您的帳戶已創建，初始餘額：{} 幣", username, initial_balance),
                })
            },
            Err(e) => {
                error!("創建用戶帳戶失敗：{}", e);
                Err(DiscordError::AccountCreationFailed(format!("無法創建用戶帳戶：{}", e)))
            }
        }
    }

    /// 檢查用戶是否存在（包含快取機制）
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 `Result<bool>` 表示用戶是否存在
    pub async fn check_user_exists(&self, discord_user_id: i64) -> Result<bool> {
        debug!("檢查用戶是否存在：{}", discord_user_id);

        // 檢查快取
        {
            let cache = self.user_exists_cache.lock().await;
            if let Some(&(exists, timestamp)) = cache.get(&discord_user_id) {
                let now = Utc::now();
                let age = now.signed_duration_since(timestamp);

                if age.num_seconds() < CACHE_EXPIRY_SECONDS as i64 {
                    debug!("從快取獲取用戶存在狀態：{}, 存在：{}", discord_user_id, exists);
                    return Ok(exists);
                }
            }
        }

        // 快取未命中或已過期，查詢資料庫
        let exists = self.user_repository.user_exists(discord_user_id).await
            .map_err(|e| {
                error!("檢查用戶是否存在時發生錯誤：{}", e);
                DiscordError::DatabaseQueryError(e.to_string())
            })?;

        // 更新快取
        {
            let mut cache = self.user_exists_cache.lock().await;
            cache.insert(discord_user_id, (exists, Utc::now()));

            // 清理過期的快取項目
            let now = Utc::now();
            cache.retain(|_, (_, timestamp)| {
                let age = now.signed_duration_since(*timestamp);
                age.num_seconds() < CACHE_EXPIRY_SECONDS as i64
            });
        }

        debug!("用戶存在檢查完成：{}, 存在：{}", discord_user_id, exists);
        Ok(exists)
    }

    /// 清除用戶快取（用於測試或資料更新）
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID，如果為 None 則清除所有快取
    pub async fn clear_cache(&self, discord_user_id: Option<i64>) {
        let mut cache = self.user_exists_cache.lock().await;

        match discord_user_id {
            Some(user_id) => {
                cache.remove(&user_id);
                debug!("清除用戶 {} 的快取", user_id);
            },
            None => {
                cache.clear();
                debug!("清除所有用戶快取");
            }
        }
    }

    /// 初始化用戶餘額
    ///
    /// # Returns
    ///
    /// 返回初始餘額（預設 1000.00）
    async fn initialize_balance(&self) -> BigDecimal {
        debug!("設置初始餘額");

        BigDecimal::from_str("1000.00")
            .unwrap_or_else(|_| BigDecimal::from_str("1000").unwrap())
    }

    /// 獲取用戶當前餘額
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    ///
    /// # Returns
    ///
    /// 返回用戶餘額，如果用戶不存在則返回 None
    pub async fn get_user_balance(&self, discord_user_id: i64) -> Result<Option<BigDecimal>> {
        debug!("獲取用戶餘額：{}", discord_user_id);

        self.user_repository.get_balance(discord_user_id).await
            .map_err(|e| {
                error!("獲取用戶餘額時發生錯誤：{}", e);
                DiscordError::DatabaseQueryError(e.to_string())
            })
    }
}

/// 帳戶創建結果
///
/// 包含帳戶創建操作的所有相關資訊
#[derive(Debug, Clone)]
pub struct AccountCreationResult {
    /// 操作是否成功
    pub success: bool,
    /// 是否為新創建的帳戶
    pub was_created: bool,
    /// 用戶資訊
    pub user: crate::database::user_repository::User,
    /// 操作結果訊息（用於 Discord 回覆）
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_user_pool;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_user_account_service_creation() {
        // 測試 UserAccountService 創建
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = UserRepository::new(pool);
            let _service = UserAccountService::new(user_repo);

            // 服務創建應該成功
            assert!(true, "UserAccountService 創建成功");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }

    #[tokio::test]
    async fn test_initialize_balance() {
        // 測試初始餘額設置
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = UserRepository::new(pool);
            let service = UserAccountService::new(user_repo).expect("Failed to create UserAccountService");

            let balance = service.initialize_balance().await;
            let expected = BigDecimal::from_str("1000.00").unwrap();

            assert_eq!(balance, expected, "初始餘額應該為 1000.00");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }
}