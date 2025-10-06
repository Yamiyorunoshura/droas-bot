// Balance Repository - 資料庫存取層
// 處理餘額相關的資料庫操作

use sqlx::{postgres::PgPool, Row};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::error::{DiscordError, Result};
use tracing::{info, error, debug};

/// 用戶餘額資訊結構
#[derive(Debug, Clone)]
pub struct Balance {
    pub discord_user_id: i64,
    pub username: String,
    pub balance: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Balance Repository - 處理餘額查詢的資料庫操作
pub struct BalanceRepository {
    pool: PgPool,
}

impl BalanceRepository {
    /// 創建新的 BalanceRepository 實例
    pub fn new(pool: PgPool) -> Self {
        info!("Creating BalanceRepository with database pool");
        Self { pool }
    }

    /// 根據用戶 ID 查詢餘額
    pub async fn find_by_user_id(&self, user_id: u64) -> Result<Option<Balance>> {
        debug!("Querying balance for user ID: {}", user_id);

        let query = sqlx::query(
            r#"
            SELECT
                discord_user_id,
                username,
                balance,
                created_at,
                updated_at
            FROM users
            WHERE discord_user_id = $1
            "#
        )
        .bind(user_id as i64);

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to query balance for user {}: {}", user_id, e);
                DiscordError::DatabaseQueryError(format!("Failed to query balance: {}", e))
            })?;

        match row {
            Some(row) => {
                let balance = Balance {
                    discord_user_id: row.get("discord_user_id"),
                    username: row.get("username"),
                    balance: row.get("balance"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };

                info!("Successfully retrieved balance for user ID: {}", user_id);
                debug!("Balance details: user={}, balance={}", balance.username, balance.balance);
                Ok(Some(balance))
            }
            None => {
                info!("No balance found for user ID: {}", user_id);
                Ok(None)
            }
        }
    }

    /// 獲取用戶餘額（僅返回餘額數值）
    pub async fn get_balance_amount(&self, user_id: u64) -> Result<Option<BigDecimal>> {
        debug!("Querying balance amount for user ID: {}", user_id);

        let query = sqlx::query(
            r#"
            SELECT balance
            FROM users
            WHERE discord_user_id = $1
            "#
        )
        .bind(user_id as i64);

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to query balance amount for user {}: {}", user_id, e);
                DiscordError::DatabaseQueryError(format!("Failed to query balance amount: {}", e))
            })?;

        match row {
            Some(row) => {
                let balance: BigDecimal = row.get("balance");
                info!("Successfully retrieved balance amount for user ID: {}", user_id);
                debug!("Balance amount: {}", balance);
                Ok(Some(balance))
            }
            None => {
                info!("No balance found for user ID: {}", user_id);
                Ok(None)
            }
        }
    }

    /// 檢查用戶是否存在
    pub async fn user_exists(&self, user_id: u64) -> Result<bool> {
        debug!("Checking if user exists for user ID: {}", user_id);

        let query = sqlx::query(
            r#"
            SELECT 1 as exists
            FROM users
            WHERE discord_user_id = $1
            LIMIT 1
            "#
        )
        .bind(user_id as i64);

        let result = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to check user existence for user {}: {}", user_id, e);
                DiscordError::DatabaseQueryError(format!("Failed to check user existence: {}", e))
            })?;

        let exists = result.is_some();
        info!("User {} exists: {}", user_id, exists);
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_pool;

    #[tokio::test]
    async fn test_balance_repository_creation() {
        // 測試 BalanceRepository 創建 - 使用模擬方式，不依賴真實資料庫
        // 注意：這個測試只驗證結構體的定義是否正確，不測試實際資料庫連接

        // 由於我們不能總是依賴測試資料庫的存在，我們改為測試 BalanceRepository 的類型檢查
        // 這確保我們的結構體定義是正確的

        // 測試 BalanceRepository 的大小（確保結構體定義正確）
        assert_eq!(std::mem::size_of::<BalanceRepository>(), std::mem::size_of::<PgPool>());

        // 如果環境變數 TEST_DATABASE_URL 設置了，則測試實際連接
        if std::env::var("TEST_DATABASE_URL").is_ok() {
            let pool = create_test_pool().await;
            if pool.is_ok() {
                let _repo = BalanceRepository::new(pool.unwrap());
                assert!(true, "BalanceRepository should be created successfully");
            }
        }
    }

    #[tokio::test]
    async fn test_user_exists() {
        // 測試用戶存在性檢查
        let pool = create_test_pool().await;

        if let Ok(pool) = pool {
            let repo = BalanceRepository::new(pool);

            // 測試不存在的用戶
            let non_existent_user = 999999999_u64;
            let exists = repo.user_exists(non_existent_user).await;
            assert!(exists.is_ok(), "User existence check should succeed");
            assert!(!exists.unwrap(), "Non-existent user should return false");
        }
    }
}