use crate::discord_gateway::{DiscordGateway, ConnectionStatus};
use crate::error::Result;
use sqlx::PgPool;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub discord_connected: bool,
    pub database_connected: bool,
    pub last_check: Instant,
    pub uptime: Duration,
}

pub struct HealthChecker {
    start_time: Instant,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// 檢查系統健康狀態
    ///
    /// 檢查 Discord API 連接和資料庫連接狀態。
    ///
    /// # Arguments
    ///
    /// * `gateway` - Discord Gateway 連接實例
    /// * `database_pool` - 資料庫連接池
    ///
    /// # Returns
    ///
    /// 返回包含所有健康狀態檢查結果的 `HealthStatus`
    pub async fn check_health(&self, gateway: &DiscordGateway, database_pool: &PgPool) -> HealthStatus {
        let discord_connected = gateway.get_status().await == ConnectionStatus::Connected;

        // 檢查資料庫連接
        let database_connected = self.check_database_connection(database_pool).await
            .unwrap_or_else(|_| false);

        HealthStatus {
            discord_connected,
            database_connected,
            last_check: Instant::now(),
            uptime: self.start_time.elapsed(),
        }
    }

    /// 檢查資料庫連接狀態
    ///
    /// 執行簡單的查詢來驗證資料庫連接是否正常。
    ///
    /// # Arguments
    ///
    /// * `pool` - 資料庫連接池
    ///
    /// # Returns
    ///
    /// 返回 `Ok(true)` 如果連接正常，否則返回錯誤或 `Ok(false)`
    async fn check_database_connection(&self, pool: &PgPool) -> Result<bool> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(pool)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 判斷系統是否健康
    ///
    /// 系統健康需要 Discord API 和資料庫都正常連接。
    ///
    /// # Arguments
    ///
    /// * `status` - 健康狀態檢查結果
    ///
    /// # Returns
    ///
    /// 返回 `true` 如果所有關鍵服務都正常，否則返回 `false`
    pub fn is_healthy(&self, status: &HealthStatus) -> bool {
        status.discord_connected && status.database_connected
    }
}