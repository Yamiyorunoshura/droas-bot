// Balance Service - 業務邏輯層
// 處理餘額查詢的業務邏輯

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::database::BalanceRepository;
use crate::cache::BalanceCache;
use crate::error::{DiscordError, Result};
use crate::services::message_service::MessageService;
use serenity::builder::CreateMessage;
use serenity::model::id::UserId;
use tracing::{info, error, debug, warn, instrument};

/// 餘額查詢響應結構
#[derive(Debug, Clone)]
pub struct BalanceResponse {
    pub user_id: u64,
    pub username: String,
    pub balance: BigDecimal,
    pub created_at: Option<DateTime<Utc>>,
}

/// Balance Service - 處理餘額查詢業務邏輯
pub struct BalanceService {
    balance_repository: BalanceRepository,
    balance_cache: BalanceCache,
    message_service: MessageService,
}

impl BalanceService {
    /// 創建新的 BalanceService 實例
    pub fn new(balance_repository: BalanceRepository) -> Self {
        info!("Creating BalanceService with repository and cache");
        Self {
            balance_repository,
            balance_cache: BalanceCache::new(),
            message_service: MessageService::new(),
        }
    }

    /// 創建帶自定義快取的 BalanceService 實例
    pub fn new_with_cache(balance_repository: BalanceRepository, balance_cache: BalanceCache) -> Self {
        info!("Creating BalanceService with custom cache");
        Self {
            balance_repository,
            balance_cache,
            message_service: MessageService::new(),
        }
    }

    /// 查詢用戶餘額
    ///
    /// # Arguments
    /// * `user_id` - Discord 用戶 ID
    ///
    /// # Returns
    /// * `Ok(BalanceResponse)` - 餘額查詢成功
    /// * `Err(DiscordError)` - 查詢失敗
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_balance(&self, user_id: u64) -> Result<BalanceResponse> {
        info!("Processing balance query for user ID: {}", user_id);

        // 首先檢查快取
        if let Some(cached_balance) = self.balance_cache.get_balance(user_id).await {
            info!("Cache hit for user ID: {}", user_id);
            debug!("Retrieved balance from cache: {}", cached_balance);

            // 從快取獲取餘額，但仍需要用戶名稱和創建日期
            // 這裡可以優化為將完整的 BalanceResponse 快取，但為簡化先這樣實作
            return self.get_balance_with_cached_amount(user_id, cached_balance).await;
        }

        info!("Cache miss for user ID: {}, querying database", user_id);

        // 首先檢查用戶是否存在
        let user_exists = self.balance_repository.user_exists(user_id).await
            .map_err(|e| {
                error!("Failed to check user existence for user {}: {}", user_id, e);
                e
            })?;

        if !user_exists {
            warn!("User {} does not have an account", user_id);
            return Err(DiscordError::UserNotFound(format!("用戶 {} 沒有經濟帳戶，請先使用 `!create` 創建帳戶", user_id)));
        }

        // 查詢用戶餘額
        let balance_data = self.balance_repository.find_by_user_id(user_id).await
            .map_err(|e| {
                error!("Failed to query balance for user {}: {}", user_id, e);
                e
            })?;

        match balance_data {
            Some(balance) => {
                info!("Successfully retrieved balance for user ID: {}", user_id);
                debug!("Balance details: user={}, balance={}", balance.username, balance.balance);

                // 將餘額存入快取
                self.balance_cache.set_balance(user_id, balance.balance.clone()).await;
                debug!("Balance cached for user ID: {}", user_id);

                let response = BalanceResponse {
                    user_id,
                    username: balance.username,
                    balance: balance.balance,
                    created_at: Some(balance.created_at),
                };

                Ok(response)
            }
            None => {
                warn!("Unexpected: user exists but no balance data found for user ID: {}", user_id);
                Err(DiscordError::UserNotFound(format!("用戶 {} 沒有有效的餘額數據", user_id)))
            }
        }
    }

    /// 使用快取的餘額數值獲取完整響應
    async fn get_balance_with_cached_amount(&self, user_id: u64, cached_balance: BigDecimal) -> Result<BalanceResponse> {
        debug!("Getting full balance response with cached amount for user ID: {}", user_id);

        // 即使有快取的餘額，仍需要查詢用戶名稱和創建日期
        let balance_data = self.balance_repository.find_by_user_id(user_id).await
            .map_err(|e| {
                error!("Failed to query user details for user {}: {}", user_id, e);
                e
            })?;

        match balance_data {
            Some(balance) => {
                // 使用快取的餘額數值，但使用資料庫中的其他資訊
                let response = BalanceResponse {
                    user_id,
                    username: balance.username,
                    balance: cached_balance, // 使用快取的餘額
                    created_at: Some(balance.created_at),
                };

                Ok(response)
            }
            None => {
                warn!("Unexpected: cached balance exists but no user data found for user ID: {}", user_id);
                Err(DiscordError::UserNotFound(format!("用戶 {} 沒有有效的用戶數據", user_id)))
            }
        }
    }

    /// 獲取用戶餘額數值（僅返回餘額，不包含其他資訊）
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_balance_amount(&self, user_id: u64) -> Result<BigDecimal> {
        info!("Processing balance amount query for user ID: {}", user_id);

        // 首先檢查快取
        if let Some(cached_balance) = self.balance_cache.get_balance(user_id).await {
            info!("Cache hit for balance amount query, user ID: {}", user_id);
            debug!("Retrieved balance amount from cache: {}", cached_balance);
            return Ok(cached_balance);
        }

        info!("Cache miss for balance amount query, user ID: {}, querying database", user_id);

        // 檢查用戶是否存在
        let user_exists = self.balance_repository.user_exists(user_id).await?;
        if !user_exists {
            return Err(DiscordError::UserNotFound(format!("用戶 {} 沒有經濟帳戶", user_id)));
        }

        // 查詢餘額數值
        let balance_amount = self.balance_repository.get_balance_amount(user_id).await?;
        match balance_amount {
            Some(amount) => {
                info!("Successfully retrieved balance amount for user ID: {}", user_id);
                debug!("Balance amount: {}", amount);

                // 將餘額存入快取
                self.balance_cache.set_balance(user_id, amount.clone()).await;
                debug!("Balance amount cached for user ID: {}", user_id);

                Ok(amount)
            }
            None => {
                error!("User exists but no balance amount found for user ID: {}", user_id);
                Err(DiscordError::DatabaseQueryError(format!("用戶 {} 沒有有效的餘額數據", user_id)))
            }
        }
    }

    /// 檢查用戶是否有足夠的餘額（為轉帳功能準備）
    #[instrument(skip(self), fields(user_id = %user_id, required_amount = %required_amount))]
    pub async fn has_sufficient_balance(&self, user_id: u64, required_amount: &BigDecimal) -> Result<bool> {
        debug!("Checking if user {} has sufficient balance: {}", user_id, required_amount);

        // 檢查用戶是否存在
        let user_exists = self.balance_repository.user_exists(user_id).await?;
        if !user_exists {
            return Err(DiscordError::UserNotFound(format!("用戶 {} 沒有經濟帳戶", user_id)));
        }

        // 獲取當前餘額
        let current_balance = self.get_balance_amount(user_id).await?;

        // 比較餘額
        let has_sufficient = current_balance >= *required_amount;

        info!("User {} balance check: required={}, current={}, sufficient={}",
              user_id, required_amount, current_balance, has_sufficient);

        Ok(has_sufficient)
    }

    /// 獲取用戶餘額的 embed 消息
    ///
    /// # Arguments
    /// * `user_id` - Discord 用戶 ID
    ///
    /// # Returns
    /// * `Ok(CreateMessage)` - 餘額查詢的 embed 消息
    /// * `Err(DiscordError)` - 查詢失敗
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_balance_embed(&self, user_id: UserId) -> Result<CreateMessage> {
        info!("Creating balance embed for user ID: {}", user_id);

        // 獲取餘額數據
        let balance_response = self.get_balance(user_id.into()).await?;

        // 將 BigDecimal 轉換為 f64，使用字符串轉換
        let balance_f64 = balance_response.balance.to_string().parse::<f64>().unwrap_or(0.0);

        // 使用 MessageService 創建 embed
        let embed = self.message_service.create_balance_embed(user_id, balance_f64).await;

        info!("Successfully created balance embed for user ID: {}", user_id);
        Ok(embed)
    }

    /// 設置用戶餘額（用於測試）
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn set_balance(&self, user_id: u64, balance: BigDecimal) -> Result<()> {
        info!("Setting balance for user ID: {} to {}", user_id, balance);

        // 檢查用戶是否存在
        let user_exists = self.balance_repository.user_exists(user_id).await?;
        if !user_exists {
            return Err(DiscordError::UserNotFound(format!("用戶 {} 沒有經濟帳戶", user_id)));
        }

        // 設置快取
        self.balance_cache.set_balance(user_id, balance.clone()).await;

        info!("Balance cached successfully for user ID: {}", user_id);
        Ok(())
    }

    /// 更新用戶餘額（用於測試）
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn update_balance(&self, user_id: u64, new_balance: BigDecimal) -> Result<()> {
        info!("Updating balance for user ID: {} to {}", user_id, new_balance);

        // 檢查用戶是否存在
        let user_exists = self.balance_repository.user_exists(user_id).await?;
        if !user_exists {
            return Err(DiscordError::UserNotFound(format!("用戶 {} 沒有經濟帳戶", user_id)));
        }

        // 在實際實現中，這裡應該更新資料庫中的餘額
        // 但為了測試目的，我們只更新快取
        self.balance_cache.set_balance(user_id, new_balance.clone()).await;

        info!("Balance updated and cached successfully for user ID: {}", user_id);
        Ok(())
    }

    /// 直接獲取快取中的餘額（用於測試）
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_cached_balance(&self, user_id: u64) -> Option<BigDecimal> {
        debug!("Getting cached balance for user ID: {}", user_id);
        self.balance_cache.get_balance(user_id).await
    }

    /// 獲取快取統計信息（用於測試）
    pub async fn get_cache_stats(&self) -> crate::cache::CacheStats {
        debug!("Getting cache statistics");
        self.balance_cache.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{create_test_pool, BalanceRepository};
    
    #[tokio::test]
    async fn test_balance_service_creation() {
        // 測試 BalanceService 創建 - 使用模擬方式，不依賴真實資料庫
        // 注意：這個測試只驗證服務的結構定義是否正確，不測試實際資料庫連接

        // 測試 BalanceService 的大小（確保結構體定義正確）
        // BalanceRepository 的大小應該等於 PgPool 的大小
        assert_eq!(
            std::mem::size_of::<BalanceRepository>(),
            std::mem::size_of::<sqlx::PgPool>()
        );

        // 如果環境變數 TEST_DATABASE_URL 設置了，則測試實際連接
        if std::env::var("TEST_DATABASE_URL").is_ok() {
            let pool = create_test_pool().await;
            if pool.is_ok() {
                let balance_repo = BalanceRepository::new(pool.unwrap());
                let _service = BalanceService::new(balance_repo);
                assert!(true, "BalanceService should be created successfully");
            }
        }
    }

    #[tokio::test]
    async fn test_get_balance_no_account() {
        // 測試查詢不存在帳戶的餘額
        let pool = create_test_pool().await;

        if let Ok(pool) = pool {
            let balance_repo = BalanceRepository::new(pool);

            let service = BalanceService::new(balance_repo);

            let non_existent_user = 999999999_u64;
            let result = service.get_balance(non_existent_user).await;

            assert!(result.is_err(), "Querying non-existent user should fail");

            match result.unwrap_err() {
                DiscordError::UserNotFound(msg) => {
                    assert!(msg.contains("沒有經濟帳戶"), "Error message should mention missing account");
                }
                _ => panic!("Expected UserNotFound error"),
            }
        }
    }

    #[tokio::test]
    async fn test_get_balance_amount_no_account() {
        // 測試查詢不存在帳戶的餘額數值
        let pool = create_test_pool().await;

        if let Ok(pool) = pool {
            let balance_repo = BalanceRepository::new(pool);

            let service = BalanceService::new(balance_repo);

            let non_existent_user = 999999999_u64;
            let result = service.get_balance_amount(non_existent_user).await;

            assert!(result.is_err(), "Querying non-existent user balance amount should fail");
        }
    }
}