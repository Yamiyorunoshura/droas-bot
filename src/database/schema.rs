use sqlx::{SqlitePool, Row};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::debug;

/// Guild配置數據模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuildConfig {
    /// Guild ID (Discord guild ID)
    pub guild_id: String,
    /// 歡迎頻道ID
    pub welcome_channel_id: String,
    /// 背景圖片引用
    pub background_ref: Option<String>,
    /// 更新時間
    pub updated_at: DateTime<Utc>,
}

/// 背景資源數據模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackgroundAsset {
    /// 資源ID
    pub asset_id: String,
    /// 檔案路徑
    pub file_path: String,
    /// 媒體類型
    pub media_type: String,
    /// 檔案大小（字節）
    pub file_size: i64,
    /// 創建時間
    pub created_at: DateTime<Utc>,
}

/// Guild配置服務
pub struct GuildConfigService {
    pool: SqlitePool,
}

impl GuildConfigService {
    /// 創建新的Guild配置服務
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 創建或更新Guild配置
    pub async fn upsert_guild_config(&self, config: &GuildConfig) -> Result<()> {
        debug!("更新Guild配置: guild_id={}", config.guild_id);
        
        sqlx::query(
            r#"
            INSERT INTO guild_config (guild_id, welcome_channel_id, background_ref, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(guild_id) DO UPDATE SET
                welcome_channel_id = excluded.welcome_channel_id,
                background_ref = excluded.background_ref,
                updated_at = excluded.updated_at
            "#
        )
        .bind(&config.guild_id)
        .bind(&config.welcome_channel_id)
        .bind(&config.background_ref)
        .bind(&config.updated_at)
        .execute(&self.pool)
        .await?;

        debug!("Guild配置更新成功");
        Ok(())
    }

    /// 根據Guild ID獲取配置
    pub async fn get_guild_config(&self, guild_id: &str) -> Result<Option<GuildConfig>> {
        debug!("查詢Guild配置: guild_id={}", guild_id);
        
        let row = sqlx::query(
            "SELECT guild_id, welcome_channel_id, background_ref, updated_at FROM guild_config WHERE guild_id = ?"
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        let config = if let Some(row) = row {
            Some(GuildConfig {
                guild_id: row.get("guild_id"),
                welcome_channel_id: row.get("welcome_channel_id"),
                background_ref: row.get("background_ref"),
                updated_at: row.get("updated_at"),
            })
        } else {
            None
        };

        debug!("查詢結果: {:?}", config.is_some());
        Ok(config)
    }

    /// 刪除Guild配置
    pub async fn delete_guild_config(&self, guild_id: &str) -> Result<bool> {
        debug!("刪除Guild配置: guild_id={}", guild_id);
        
        let result = sqlx::query(
            "DELETE FROM guild_config WHERE guild_id = ?"
        )
        .bind(guild_id)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected() > 0;
        debug!("刪除結果: deleted={}", deleted);
        Ok(deleted)
    }

    /// 獲取所有配置（用於管理和統計）
    pub async fn get_all_guild_configs(&self) -> Result<Vec<GuildConfig>> {
        debug!("查詢所有Guild配置");
        
        let rows = sqlx::query(
            "SELECT guild_id, welcome_channel_id, background_ref, updated_at FROM guild_config ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let configs: Vec<GuildConfig> = rows.into_iter().map(|row| GuildConfig {
            guild_id: row.get("guild_id"),
            welcome_channel_id: row.get("welcome_channel_id"),
            background_ref: row.get("background_ref"),
            updated_at: row.get("updated_at"),
        }).collect();

        debug!("查詢到 {} 個Guild配置", configs.len());
        Ok(configs)
    }
}

/// 背景資源服務
pub struct BackgroundAssetService {
    pool: SqlitePool,
}

impl BackgroundAssetService {
    /// 創建新的背景資源服務
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 創建背景資源記錄
    pub async fn create_background_asset(&self, asset: &BackgroundAsset) -> Result<()> {
        debug!("創庺背景資源: asset_id={}", asset.asset_id);
        
        sqlx::query(
            r#"
            INSERT INTO background_asset (asset_id, file_path, media_type, file_size, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&asset.asset_id)
        .bind(&asset.file_path)
        .bind(&asset.media_type)
        .bind(&asset.file_size)
        .bind(&asset.created_at)
        .execute(&self.pool)
        .await?;

        debug!("背景資源創庺成功");
        Ok(())
    }

    /// 根據資源ID獲取背景資源
    pub async fn get_background_asset(&self, asset_id: &str) -> Result<Option<BackgroundAsset>> {
        debug!("查詢背景資源: asset_id={}", asset_id);
        
        let row = sqlx::query(
            "SELECT asset_id, file_path, media_type, file_size, created_at FROM background_asset WHERE asset_id = ?"
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await?;

        let asset = if let Some(row) = row {
            Some(BackgroundAsset {
                asset_id: row.get("asset_id"),
                file_path: row.get("file_path"),
                media_type: row.get("media_type"),
                file_size: row.get("file_size"),
                created_at: row.get("created_at"),
            })
        } else {
            None
        };

        debug!("查詢結果: {:?}", asset.is_some());
        Ok(asset)
    }

    /// 刪除背景資源記錄
    pub async fn delete_background_asset(&self, asset_id: &str) -> Result<bool> {
        debug!("刪除背景資源: asset_id={}", asset_id);
        
        let result = sqlx::query(
            "DELETE FROM background_asset WHERE asset_id = ?"
        )
        .bind(asset_id)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected() > 0;
        debug!("刪除結果: deleted={}", deleted);
        Ok(deleted)
    }

    /// 獲取所有背景資源（用於清理和統計）
    pub async fn get_all_background_assets(&self) -> Result<Vec<BackgroundAsset>> {
        debug!("查詢所有背景資源");
        
        let rows = sqlx::query(
            "SELECT asset_id, file_path, media_type, file_size, created_at FROM background_asset ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let assets: Vec<BackgroundAsset> = rows.into_iter().map(|row| BackgroundAsset {
            asset_id: row.get("asset_id"),
            file_path: row.get("file_path"),
            media_type: row.get("media_type"),
            file_size: row.get("file_size"),
            created_at: row.get("created_at"),
        }).collect();

        debug!("查詢到 {} 個背景資源", assets.len());
        Ok(assets)
    }

    /// 獲取資源總大小（用於配額管理）
    pub async fn get_total_asset_size(&self) -> Result<i64> {
        debug!("查詢資源總大小");
        
        let result: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(SUM(file_size), 0) FROM background_asset"
        )
        .fetch_one(&self.pool)
        .await?;

        let total_size = result.unwrap_or(0);
        debug!("資源總大小: {} bytes", total_size);
        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    async fn create_test_services() -> (GuildConfigService, BackgroundAssetService, NamedTempFile) {
        let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let db = DatabaseManager::new(&database_url)
            .await
            .expect("無法創建測試資料庫");
        
        db.run_migrations().await.expect("無法執行遷移");
        
        let guild_service = GuildConfigService::new(db.pool().clone());
        let asset_service = BackgroundAssetService::new(db.pool().clone());
        
        (guild_service, asset_service, temp_file)
    }

    #[tokio::test]
    async fn test_guild_config_crud() {
        let (service, _asset_service, _temp_file) = create_test_services().await;
        
        let config = GuildConfig {
            guild_id: "test_guild_123".to_string(),
            welcome_channel_id: "channel_456".to_string(),
            background_ref: Some("bg_001".to_string()),
            updated_at: Utc::now(),
        };

        // 測試創建
        let result = service.upsert_guild_config(&config).await;
        assert!(result.is_ok());

        // 測試讀取
        let retrieved = service.get_guild_config(&config.guild_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.guild_id, config.guild_id);
        assert_eq!(retrieved.welcome_channel_id, config.welcome_channel_id);

        // 測試更新
        let mut updated_config = config.clone();
        updated_config.welcome_channel_id = "new_channel_789".to_string();
        updated_config.updated_at = Utc::now();
        
        let result = service.upsert_guild_config(&updated_config).await;
        assert!(result.is_ok());
        
        let retrieved = service.get_guild_config(&config.guild_id).await.unwrap().unwrap();
        assert_eq!(retrieved.welcome_channel_id, "new_channel_789");

        // 測試刪除
        let deleted = service.delete_guild_config(&config.guild_id).await.unwrap();
        assert!(deleted);
        
        let retrieved = service.get_guild_config(&config.guild_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_background_asset_crud() {
        let (_guild_service, service, _temp_file) = create_test_services().await;
        
        let asset = BackgroundAsset {
            asset_id: Uuid::new_v4().to_string(),
            file_path: "/test/path/bg.png".to_string(),
            media_type: "image/png".to_string(),
            file_size: 1024000,
            created_at: Utc::now(),
        };

        // 測試創建
        let result = service.create_background_asset(&asset).await;
        assert!(result.is_ok());

        // 測試讀取
        let retrieved = service.get_background_asset(&asset.asset_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.asset_id, asset.asset_id);
        assert_eq!(retrieved.file_path, asset.file_path);
        assert_eq!(retrieved.media_type, asset.media_type);
        assert_eq!(retrieved.file_size, asset.file_size);

        // 測試獲取總大小
        let total_size = service.get_total_asset_size().await.unwrap();
        assert_eq!(total_size, asset.file_size);

        // 測試刪除
        let deleted = service.delete_background_asset(&asset.asset_id).await.unwrap();
        assert!(deleted);
        
        let retrieved = service.get_background_asset(&asset.asset_id).await.unwrap();
        assert!(retrieved.is_none());
        
        // 確認總大小歸零
        let total_size = service.get_total_asset_size().await.unwrap();
        assert_eq!(total_size, 0);
    }

    #[tokio::test]
    async fn test_get_all_operations() {
        let (guild_service, asset_service, _temp_file) = create_test_services().await;
        
        // 創建多個Guild配置
        for i in 0..3 {
            let config = GuildConfig {
                guild_id: format!("guild_{}", i),
                welcome_channel_id: format!("channel_{}", i),
                background_ref: None,
                updated_at: Utc::now(),
            };
            guild_service.upsert_guild_config(&config).await.unwrap();
        }

        // 創建多個背景資源
        for i in 0..2 {
            let asset = BackgroundAsset {
                asset_id: format!("asset_{}", i),
                file_path: format!("/test/path/bg_{}.png", i),
                media_type: "image/png".to_string(),
                file_size: 1024 * (i + 1),
                created_at: Utc::now(),
            };
            asset_service.create_background_asset(&asset).await.unwrap();
        }

        // 測試獲取所有配置
        let all_configs = guild_service.get_all_guild_configs().await.unwrap();
        assert_eq!(all_configs.len(), 3);

        // 測試獲取所有資源
        let all_assets = asset_service.get_all_background_assets().await.unwrap();
        assert_eq!(all_assets.len(), 2);
    }
}