//! 公會配置資料庫操作 Repository
//!
//! 此模組提供公會配置管理的資料存取層，使用 Repository Pattern
//! 抽象資料庫操作，提高可測試性和維護性。

use crate::config::models::{BackgroundAsset, GuildConfig};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 公會配置 Repository
///
/// 提供對公會配置和背景圖片資源的完整 CRUD 操作，
/// 封裝所有資料庫存取邏輯，確保資料一致性。
#[derive(Clone)]
pub struct GuildConfigRepository {
    pool: Pool<Sqlite>,
}

impl GuildConfigRepository {
    /// 創建新的 Repository 實例
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite 連接池
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// 執行資料庫遷移
    ///
    /// 從遷移文件創建必要的表和索引
    pub async fn migrate(&self) -> Result<()> {
        let migration_sql = include_str!("../../migrations/001_create_config_tables.sql");

        info!("執行公會配置資料庫遷移...");
        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .context("執行資料庫遷移失敗")?;

        info!("公會配置資料庫遷移完成");
        Ok(())
    }

    /// 根據公會 ID 獲取配置
    ///
    /// # Arguments
    ///
    /// * `guild_id` - Discord 公會 ID
    ///
    /// # Returns
    ///
    /// 返回 Option<GuildConfig>，如果配置不存在則為 None
    pub async fn get_config(&self, guild_id: i64) -> Result<Option<GuildConfig>> {
        debug!("查詢公會配置: guild_id={}", guild_id);

        let config = sqlx::query_as::<_, GuildConfig>(
            "SELECT guild_id, welcome_channel_id, background_ref, updated_at, created_at 
             FROM guild_configs 
             WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("查詢公會配置失敗: guild_id={}", guild_id))?;

        if config.is_some() {
            debug!("找到公會配置: guild_id={}", guild_id);
        } else {
            debug!("未找到公會配置: guild_id={}", guild_id);
        }

        Ok(config)
    }

    /// 獲取所有公會配置（用於預加載）
    ///
    /// # Returns
    ///
    /// 返回所有公會配置的 HashMap，鍵為 guild_id
    pub async fn get_all_configs(&self) -> Result<HashMap<i64, GuildConfig>> {
        info!("加載所有公會配置...");

        let rows = sqlx::query(
            "SELECT guild_id, welcome_channel_id, background_ref, updated_at, created_at 
             FROM guild_configs",
        )
        .fetch_all(&self.pool)
        .await
        .context("查詢所有公會配置失敗")?;

        let mut configs = HashMap::new();
        for row in rows {
            let guild_id = row.get::<i64, _>("guild_id");
            let config = GuildConfig {
                guild_id,
                welcome_channel_id: row.get("welcome_channel_id"),
                background_ref: row.get("background_ref"),
                updated_at: row.get("updated_at"),
                created_at: row.get("created_at"),
            };
            configs.insert(guild_id, config);
        }

        info!("成功加載 {} 個公會配置", configs.len());
        Ok(configs)
    }

    /// 創建或更新公會配置
    ///
    /// 使用 UPSERT 操作確保原子性
    ///
    /// # Arguments
    ///
    /// * `config` - 要保存的配置
    pub async fn upsert_config(&self, config: &GuildConfig) -> Result<()> {
        debug!("保存公會配置: guild_id={}", config.guild_id);

        let now = Utc::now();

        sqlx::query(
            "INSERT INTO guild_configs (guild_id, welcome_channel_id, background_ref, updated_at, created_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(guild_id) DO UPDATE SET
                 welcome_channel_id = excluded.welcome_channel_id,
                 background_ref = excluded.background_ref,
                 updated_at = excluded.updated_at"
        )
        .bind(config.guild_id)
        .bind(config.welcome_channel_id)
        .bind(&config.background_ref)
        .bind(now)
        .bind(config.created_at)
        .execute(&self.pool)
        .await
        .with_context(|| format!("保存公會配置失敗: guild_id={}", config.guild_id))?;

        info!("成功保存公會配置: guild_id={}", config.guild_id);
        Ok(())
    }

    /// 刪除公會配置
    ///
    /// # Arguments
    ///
    /// * `guild_id` - 要刪除配置的公會 ID
    pub async fn delete_config(&self, guild_id: i64) -> Result<bool> {
        debug!("刪除公會配置: guild_id={}", guild_id);

        let result = sqlx::query("DELETE FROM guild_configs WHERE guild_id = ?")
            .bind(guild_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("刪除公會配置失敗: guild_id={}", guild_id))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("成功刪除公會配置: guild_id={}", guild_id);
        } else {
            warn!("嘗試刪除不存在的公會配置: guild_id={}", guild_id);
        }

        Ok(deleted)
    }

    /// 創建背景圖片資源
    ///
    /// # Arguments
    ///
    /// * `asset` - 要創建的背景圖片資源
    pub async fn create_background_asset(&self, asset: &BackgroundAsset) -> Result<()> {
        debug!("創建背景圖片資源: id={}", asset.id);

        sqlx::query(
            "INSERT INTO background_assets (id, file_path, media_type, file_size, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&asset.id)
        .bind(&asset.file_path)
        .bind(&asset.media_type)
        .bind(asset.file_size)
        .bind(asset.created_at)
        .execute(&self.pool)
        .await
        .with_context(|| format!("創建背景圖片資源失敗: id={}", asset.id))?;

        info!("成功創建背景圖片資源: id={}", asset.id);
        Ok(())
    }

    /// 根據 ID 獲取背景圖片資源
    ///
    /// # Arguments
    ///
    /// * `asset_id` - 背景圖片資源 ID
    pub async fn get_background_asset(&self, asset_id: &str) -> Result<Option<BackgroundAsset>> {
        debug!("查詢背景圖片資源: id={}", asset_id);

        let asset = sqlx::query_as::<_, BackgroundAsset>(
            "SELECT id, file_path, media_type, file_size, created_at
             FROM background_assets
             WHERE id = ?",
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("查詢背景圖片資源失敗: id={}", asset_id))?;

        if asset.is_some() {
            debug!("找到背景圖片資源: id={}", asset_id);
        } else {
            debug!("未找到背景圖片資源: id={}", asset_id);
        }

        Ok(asset)
    }

    /// 刪除背景圖片資源
    ///
    /// 注意：由於外鍵約束，如果資源正在被使用，會自動將相關配置的 background_ref 設為 NULL
    ///
    /// # Arguments
    ///
    /// * `asset_id` - 要刪除的背景圖片資源 ID
    pub async fn delete_background_asset(&self, asset_id: &str) -> Result<bool> {
        debug!("刪除背景圖片資源: id={}", asset_id);

        let result = sqlx::query("DELETE FROM background_assets WHERE id = ?")
            .bind(asset_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("刪除背景圖片資源失敗: id={}", asset_id))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("成功刪除背景圖片資源: id={}", asset_id);
        } else {
            warn!("嘗試刪除不存在的背景圖片資源: id={}", asset_id);
        }

        Ok(deleted)
    }

    /// 獲取配置統計信息
    ///
    /// # Returns
    ///
    /// 返回包含配置統計的元組 (總配置數, 有歡迎頻道的配置數, 有背景圖片的配置數)
    pub async fn get_config_statistics(&self) -> Result<(i64, i64, i64)> {
        let row = sqlx::query(
            "SELECT 
                COUNT(*) as total_configs,
                COUNT(welcome_channel_id) as configs_with_channel,
                COUNT(background_ref) as configs_with_background
             FROM guild_configs",
        )
        .fetch_one(&self.pool)
        .await
        .context("查詢配置統計失敗")?;

        let total_configs = row.get::<i64, _>("total_configs");
        let configs_with_channel = row.get::<i64, _>("configs_with_channel");
        let configs_with_background = row.get::<i64, _>("configs_with_background");

        Ok((total_configs, configs_with_channel, configs_with_background))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use tempfile::NamedTempFile;

    async fn setup_test_db() -> Pool<Sqlite> {
        let temp_file = NamedTempFile::new().expect("創建臨時文件失敗");
        let database_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());

        SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("連接測試資料庫失敗")
    }

    #[tokio::test]
    async fn test_migrate() {
        let pool = setup_test_db().await;
        let repo = GuildConfigRepository::new(pool);

        let result = repo.migrate().await;
        assert!(result.is_ok(), "資料庫遷移應該成功");
    }

    #[tokio::test]
    async fn test_config_crud_operations() {
        let pool = setup_test_db().await;
        let repo = GuildConfigRepository::new(pool);

        // 執行遷移
        repo.migrate().await.expect("遷移失敗");

        let guild_id = 12345i64;
        let channel_id = 67890i64;

        // 測試查詢不存在的配置
        let result = repo.get_config(guild_id).await.expect("查詢失敗");
        assert!(result.is_none(), "不存在的配置應該返回 None");

        // 測試創建配置
        let config = GuildConfig::new(guild_id, Some(channel_id), None);
        repo.upsert_config(&config).await.expect("創建配置失敗");

        // 測試查詢存在的配置
        let retrieved_config = repo
            .get_config(guild_id)
            .await
            .expect("查詢失敗")
            .expect("配置應該存在");
        assert_eq!(retrieved_config.guild_id, guild_id);
        assert_eq!(retrieved_config.welcome_channel_id, Some(channel_id));
        assert_eq!(retrieved_config.background_ref, None);

        // 測試更新配置
        let mut updated_config = retrieved_config.clone();
        updated_config.update_background(Some("bg_001".to_string()));
        repo.upsert_config(&updated_config)
            .await
            .expect("更新配置失敗");

        let retrieved_updated = repo
            .get_config(guild_id)
            .await
            .expect("查詢失敗")
            .expect("配置應該存在");
        assert_eq!(retrieved_updated.background_ref, Some("bg_001".to_string()));
        assert!(retrieved_updated.updated_at > retrieved_config.updated_at);

        // 測試刪除配置
        let deleted = repo.delete_config(guild_id).await.expect("刪除失敗");
        assert!(deleted, "刪除應該成功");

        let result = repo.get_config(guild_id).await.expect("查詢失敗");
        assert!(result.is_none(), "刪除後配置應該不存在");
    }

    #[tokio::test]
    async fn test_background_asset_operations() {
        let pool = setup_test_db().await;
        let repo = GuildConfigRepository::new(pool);

        // 執行遷移
        repo.migrate().await.expect("遷移失敗");

        // 測試創建背景圖片資源
        let asset = BackgroundAsset::new(
            "test_asset".to_string(),
            "/test/path.png".to_string(),
            "image/png".to_string(),
            1024000,
        );
        repo.create_background_asset(&asset)
            .await
            .expect("創建資源失敗");

        // 測試查詢背景圖片資源
        let retrieved_asset = repo
            .get_background_asset("test_asset")
            .await
            .expect("查詢失敗")
            .expect("資源應該存在");
        assert_eq!(retrieved_asset.id, "test_asset");
        assert_eq!(retrieved_asset.file_path, "/test/path.png");
        assert_eq!(retrieved_asset.media_type, "image/png");
        assert_eq!(retrieved_asset.file_size, 1024000);

        // 測試刪除背景圖片資源
        let deleted = repo
            .delete_background_asset("test_asset")
            .await
            .expect("刪除失敗");
        assert!(deleted, "刪除應該成功");

        let result = repo
            .get_background_asset("test_asset")
            .await
            .expect("查詢失敗");
        assert!(result.is_none(), "刪除後資源應該不存在");
    }

    #[tokio::test]
    async fn test_get_all_configs() {
        let pool = setup_test_db().await;
        let repo = GuildConfigRepository::new(pool);

        // 執行遷移
        repo.migrate().await.expect("遷移失敗");

        // 創建多個配置
        let configs = vec![
            GuildConfig::new(111, Some(222), None),
            GuildConfig::new(333, Some(444), Some("bg_001".to_string())),
            GuildConfig::new(555, None, Some("bg_002".to_string())),
        ];

        for config in &configs {
            repo.upsert_config(config).await.expect("創建配置失敗");
        }

        // 測試獲取所有配置
        let all_configs = repo.get_all_configs().await.expect("查詢所有配置失敗");
        assert_eq!(all_configs.len(), 3);

        for config in &configs {
            let retrieved = all_configs.get(&config.guild_id).expect("配置應該存在");
            assert_eq!(retrieved.guild_id, config.guild_id);
            assert_eq!(retrieved.welcome_channel_id, config.welcome_channel_id);
            assert_eq!(retrieved.background_ref, config.background_ref);
        }
    }

    #[tokio::test]
    async fn test_config_statistics() {
        let pool = setup_test_db().await;
        let repo = GuildConfigRepository::new(pool);

        // 執行遷移
        repo.migrate().await.expect("遷移失敗");

        // 創建不同類型的配置
        let configs = vec![
            GuildConfig::new(111, Some(222), None), // 只有頻道
            GuildConfig::new(333, Some(444), Some("bg_001".to_string())), // 頻道 + 背景
            GuildConfig::new(555, None, Some("bg_002".to_string())), // 只有背景
            GuildConfig::new(777, None, None),      // 空配置
        ];

        for config in &configs {
            repo.upsert_config(config).await.expect("創建配置失敗");
        }

        let (total, with_channel, with_background) =
            repo.get_config_statistics().await.expect("查詢統計失敗");

        assert_eq!(total, 4);
        assert_eq!(with_channel, 2);
        assert_eq!(with_background, 2);
    }
}
