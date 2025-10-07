// Balance Service - 業務邏輯層
// 處理餘額查詢的業務邏輯

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::database::BalanceRepository;
use crate::cache::BalanceCache;
use crate::error::{DiscordError, Result};
use crate::services::message_service::MessageService;
use crate::services::security_service::SecurityService;
use crate::services::admin_audit_service::AdminAuditService;
use std::sync::Arc;
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
    security_service: Option<Arc<SecurityService>>,
    admin_audit_service: Option<Arc<AdminAuditService>>,
}

impl BalanceService {
    /// 創建新的 BalanceService 實例
    pub fn new(balance_repository: BalanceRepository) -> Self {
        info!("Creating BalanceService with repository and cache");
        Self {
            balance_repository,
            balance_cache: BalanceCache::new(),
            message_service: MessageService::new(),
            security_service: None,
            admin_audit_service: None,
        }
    }

    /// 創建帶自定義快取的 BalanceService 實例
    pub fn new_with_cache(balance_repository: BalanceRepository, balance_cache: BalanceCache) -> Self {
        info!("Creating BalanceService with custom cache");
        Self {
            balance_repository,
            balance_cache,
            message_service: MessageService::new(),
            security_service: None,
            admin_audit_service: None,
        }
    }

    /// 創建帶安全服務的 BalanceService 實例（用於管理員功能）
    pub fn new_with_admin_services(
        balance_repository: BalanceRepository,
        security_service: Arc<SecurityService>,
        admin_audit_service: Arc<AdminAuditService>
    ) -> Self {
        info!("Creating BalanceService with admin services");
        Self {
            balance_repository,
            balance_cache: BalanceCache::new(),
            message_service: MessageService::new(),
            security_service: Some(security_service),
            admin_audit_service: Some(admin_audit_service),
        }
    }

    /// 設置安全服務（用於現有實例）
    pub fn with_security_service(mut self, security_service: Arc<SecurityService>) -> Self {
        self.security_service = Some(security_service);
        self
    }

    /// 設置審計服務（用於現有實例）
    pub fn with_admin_audit_service(mut self, admin_audit_service: Arc<AdminAuditService>) -> Self {
        self.admin_audit_service = Some(admin_audit_service);
        self
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

    /// 管理員調整用戶餘額（GREEN 階段實現）
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 執行調整的管理員 Discord 用戶 ID
    /// * `admin_users` - 授權的管理員用戶 ID 列表
    /// * `target_user_id` - 目標用戶 Discord 用戶 ID
    /// * `amount` - 調整金額（正數為增加，負數為減少）
    /// * `reason` - 調整原因
    ///
    /// # Returns
    ///
    /// 返回 Result<BalanceResponse>，成功時包含調整後的餘額信息
    #[instrument(skip(self), fields(admin_id = %admin_user_id, target_id = %target_user_id, amount = %amount))]
    pub async fn adjust_balance_by_admin(
        &self,
        admin_user_id: i64,
        admin_users: &[i64],
        target_user_id: i64,
        amount: BigDecimal,
        reason: String,
    ) -> Result<BalanceResponse> {
        info!("管理員 {} 調整用戶 {} 餘額：{}，原因：{}", admin_user_id, target_user_id, amount, reason);

        // 驗證管理員權限
        let security_service = self.security_service.as_ref()
            .ok_or_else(|| DiscordError::PermissionDenied("安全服務未初始化".to_string()))?;

        security_service.verify_admin_permission(admin_user_id, admin_users).await
            .map_err(|e| {
                warn!("管理員權限驗證失敗：{}", e);
                e
            })?;

        // 驗證目標用戶存在
        let target_user_exists = self.balance_repository.user_exists(target_user_id as u64).await
            .map_err(|e| {
                error!("檢查目標用戶 {} 存在性失敗：{}", target_user_id, e);
                DiscordError::DatabaseQueryError(format!("檢查用戶失敗：{}", e))
            })?;

        if !target_user_exists {
            warn!("目標用戶 {} 不存在", target_user_id);
            return Err(DiscordError::UserNotFound(format!("目標用戶 {} 不存在", target_user_id)));
        }

        // 獲取當前餘額
        let current_balance = self.get_balance_amount(target_user_id as u64).await
            .map_err(|e| {
                error!("獲取用戶 {} 當前餘額失敗：{}", target_user_id, e);
                e
            })?;

        // 計算新餘額
        let new_balance = current_balance.clone() + amount.clone();

        // 驗證新餘額是否合理
        if new_balance < BigDecimal::from(-1000000) {
            warn!("調整後餘額過低：{}", new_balance);
            return Err(DiscordError::InvalidAmount("調整後餘額不能低於 -1,000,000".to_string()));
        }

        if new_balance > BigDecimal::from(100000000) {
            warn!("調整後餘額過高：{}", new_balance);
            return Err(DiscordError::InvalidAmount("調整後餘額不能高於 100,000,000".to_string()));
        }

        // 更新資料庫中的餘額
        self.balance_repository.update_balance(target_user_id as u64, &new_balance).await
            .map_err(|e| {
                error!("更新用戶 {} 餘額失敗：{}", target_user_id, e);
                DiscordError::DatabaseQueryError(format!("更新餘額失敗：{}", e))
            })?;

        // 更新快取
        self.balance_cache.set_balance(target_user_id as u64, new_balance.clone()).await;

        // 記錄審計信息
        if let Some(audit_service) = &self.admin_audit_service {
            let audit_record = crate::services::admin_audit_service::AdminAuditRecord {
                id: None,
                admin_id: admin_user_id,
                operation_type: if amount > BigDecimal::from(0) {
                    "ADJUST_BALANCE_ADD".to_string()
                } else {
                    "ADJUST_BALANCE_SUBTRACT".to_string()
                },
                target_user_id: Some(target_user_id),
                amount: Some(amount.clone()),
                reason: reason.clone(),
                timestamp: Utc::now(),
                ip_address: None,
                user_agent: None,
            };

            if let Err(e) = audit_service.log_admin_operation(audit_record).await {
                error!("記錄管理員審計失敗：{}", e);
                // 不影響主要功能，只記錄錯誤
            }
        } else {
            warn!("審計服務未初始化，無法記錄管理員操作");
        }

        // 獲取用戶名稱以構建響應
        let balance_data = self.balance_repository.find_by_user_id(target_user_id as u64).await
            .map_err(|e| {
                error!("獲取用戶 {} 資訊失敗：{}", target_user_id, e);
                DiscordError::DatabaseQueryError(format!("獲取用戶資訊失敗：{}", e))
            })?;

        let user_info = balance_data.ok_or_else(|| {
            DiscordError::UserNotFound("無法獲取用戶資訊".to_string())
        })?;

        info!("管理員餘額調整完成：{} -> {} (+{})", current_balance, new_balance, amount);

        Ok(BalanceResponse {
            user_id: target_user_id as u64,
            username: user_info.username,
            balance: new_balance,
            created_at: Some(user_info.created_at),
        })
    }

    /// 管理員設置用戶餘額（覆蓋現有餘額）
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 執行設置的管理員 Discord 用戶 ID
    /// * `admin_users` - 授權的管理員用戶 ID 列表
    /// * `target_user_id` - 目標用戶 Discord 用戶 ID
    /// * `new_balance` - 新的餘額
    /// * `reason` - 設置原因
    ///
    /// # Returns
    ///
    /// 返回 Result<BalanceResponse>，成功時包含設置後的餘額信息
    #[instrument(skip(self), fields(admin_id = %admin_user_id, target_id = %target_user_id, new_balance = %new_balance))]
    pub async fn set_balance_by_admin(
        &self,
        admin_user_id: i64,
        admin_users: &[i64],
        target_user_id: i64,
        new_balance: BigDecimal,
        reason: String,
    ) -> Result<BalanceResponse> {
        info!("管理員 {} 設置用戶 {} 餘額為：{}，原因：{}", admin_user_id, target_user_id, new_balance, reason);

        // 驗證管理員權限
        let security_service = self.security_service.as_ref()
            .ok_or_else(|| DiscordError::PermissionDenied("安全服務未初始化".to_string()))?;

        security_service.verify_admin_permission(admin_user_id, admin_users).await?;

        // 驗證目標用戶存在
        let target_user_exists = self.balance_repository.user_exists(target_user_id as u64).await?;
        if !target_user_exists {
            return Err(DiscordError::UserNotFound(format!("目標用戶 {} 不存在", target_user_id)));
        }

        // 驗證新餘額是否合理
        if new_balance < BigDecimal::from(-1000000) {
            return Err(DiscordError::InvalidAmount("餘額不能低於 -1,000,000".to_string()));
        }

        if new_balance > BigDecimal::from(100000000) {
            return Err(DiscordError::InvalidAmount("餘額不能高於 100,000,000".to_string()));
        }

        // 更新資料庫中的餘額
        self.balance_repository.update_balance(target_user_id as u64, &new_balance).await?;

        // 更新快取
        self.balance_cache.set_balance(target_user_id as u64, new_balance.clone()).await;

        // 記錄審計信息
        if let Some(audit_service) = &self.admin_audit_service {
            let audit_record = crate::services::admin_audit_service::AdminAuditRecord {
                id: None,
                admin_id: admin_user_id,
                operation_type: "SET_BALANCE".to_string(),
                target_user_id: Some(target_user_id),
                amount: Some(new_balance.clone()),
                reason: reason.clone(),
                timestamp: Utc::now(),
                ip_address: None,
                user_agent: None,
            };

            if let Err(e) = audit_service.log_admin_operation(audit_record).await {
                error!("記錄管理員審計失敗：{}", e);
            }
        }

        // 獲取用戶名稱以構建響應
        let balance_data = self.balance_repository.find_by_user_id(target_user_id as u64).await?;
        let user_info = balance_data.ok_or_else(|| {
            DiscordError::UserNotFound("無法獲取用戶資訊".to_string())
        })?;

        info!("管理員餘額設置完成：用戶 {} 餘額設置為 {}", target_user_id, new_balance);

        Ok(BalanceResponse {
            user_id: target_user_id as u64,
            username: user_info.username,
            balance: new_balance,
            created_at: Some(user_info.created_at),
        })
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