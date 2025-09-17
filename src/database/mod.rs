pub mod schema;
pub mod migrations;

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use anyhow::Result;
use tracing::{info, debug, error};
use std::time::Duration;

/// 資料庫管理器，負責連接池管理和基本操作
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    /// 創建新的資料庫管理器實例
    /// 
    /// # Arguments
    /// * `database_url` - SQLite 資料庫連接字符串
    /// 
    /// # Returns
    /// * `Result<Self>` - 成功返回 DatabaseManager 實例，失敗返回錯誤
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("正在初始化資料庫連接池: {}", database_url);

        // 配置連接池選項
        let pool = SqlitePoolOptions::new()
            .max_connections(20)  // 最大連接數
            .min_connections(1)   // 最小連接數
            .acquire_timeout(Duration::from_secs(8))  // 獲取連接超時
            .idle_timeout(Duration::from_secs(300))   // 空閒連接超時
            .test_before_acquire(true)  // 獲取前測試連接健康
            .connect(database_url)
            .await?;

        debug!("資料庫連接池創建成功");
        
        Ok(Self { pool })
    }

    /// 獲取資料庫連接池的引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// 檢查資料庫連接健康狀態
    pub async fn health_check(&self) -> Result<()> {
        debug!("執行資料庫健康檢查");
        
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
            
        if row.0 == 1 {
            debug!("資料庫健康檢查通過");
            Ok(())
        } else {
            error!("資料庫健康檢查失敗");
            Err(anyhow::anyhow!("資料庫健康檢查失敗"))
        }
    }

    /// 關閉資料庫連接池
    pub async fn close(self) {
        info!("關閉資料庫連接池");
        self.pool.close().await;
    }

    /// 執行資料庫遷移
    pub async fn run_migrations(&self) -> Result<()> {
        info!("開始執行資料庫遷移");
        
        // 創建 migrations 表來跟蹤遷移狀態
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // 執行初始遷移
        migrations::apply_migrations(&self.pool).await?;
        
        info!("資料庫遷移完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> (DatabaseManager, NamedTempFile) {
        let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let db = DatabaseManager::new(&database_url)
            .await
            .expect("無法創建測試資料庫");
        (db, temp_file)
    }

    #[tokio::test]
    async fn test_database_manager_creation() {
        let (db, _temp_file) = create_test_database().await;
        
        // 驗證資料庫管理器創建成功
        assert!(!db.pool().is_closed());
    }

    #[tokio::test]
    async fn test_health_check() {
        let (db, _temp_file) = create_test_database().await;
        
        // 測試健康檢查
        let result = db.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_database_close() {
        let (db, _temp_file) = create_test_database().await;
        
        // 測試關閉資料庫
        db.close().await;
        // 注意：關閉後無法再測試連接狀態，這是預期行為
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let (db, _temp_file) = create_test_database().await;
        
        // 測試執行遷移
        let result = db.run_migrations().await;
        assert!(result.is_ok());
        
        // 驗證 migrations 表被創建
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'"
        )
        .fetch_one(db.pool())
        .await
        .expect("無法查詢 migrations 表");
        
        assert_eq!(count.0, 1);
    }
}