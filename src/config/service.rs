//! 公會配置服務
//! 
//! 此模組是公會配置管理系統的主要入口點，整合了資料庫操作、緩存管理、
//! 事務控制等各個組件，提供完整的配置管理服務。

use crate::config::{
    models::{GuildConfig, BackgroundAsset},
    repository::GuildConfigRepository,
    cache::{GuildConfigCache, CacheStats},
    transaction::{ConfigTransactionManager, TransactionResult, TransactionStats},
};
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{debug, info, warn, error};

/// 配置服務統計信息
#[derive(Debug, Clone)]
pub struct ConfigServiceStats {
    /// 緩存統計
    pub cache_stats: CacheStats,
    /// 事務統計
    pub transaction_stats: TransactionStats,
    /// 總配置數量
    pub total_configs: i64,
    /// 有歡迎頻道的配置數量
    pub configs_with_channel: i64,
    /// 有背景圖片的配置數量
    pub configs_with_background: i64,
}

/// 配置更新結果
#[derive(Debug)]
pub enum ConfigUpdateResult {
    /// 更新成功
    Success,
    /// 更新失敗
    Failed(String),
    /// 事務超時
    Timeout,
}

/// 公會配置服務
/// 
/// 提供高層次的配置管理 API，封裝所有底層複雜性
pub struct GuildConfigService {
    repository: GuildConfigRepository,
    cache: GuildConfigCache,
    transaction_manager: ConfigTransactionManager,
    /// 是否啟用緩存預加載
    enable_preloading: bool,
}

impl GuildConfigService {
    /// 創建新的配置服務實例
    /// 
    /// # Arguments
    /// 
    /// * `pool` - SQLite 連接池
    /// * `cache_capacity` - 緩存最大容量（預設: 10000）
    /// * `cache_ttl_seconds` - 緩存 TTL（預設: 3600秒）
    /// * `transaction_timeout` - 事務超時時間（預設: 30秒）
    /// * `enable_preloading` - 是否啟用配置預加載（預設: true）
    pub async fn new(
        pool: Pool<Sqlite>,
        cache_capacity: Option<u64>,
        cache_ttl_seconds: Option<u64>,
        transaction_timeout: Option<u64>,
        enable_preloading: Option<bool>,
    ) -> Result<Self> {
        let repository = GuildConfigRepository::new(pool);
        
        // 執行資料庫遷移
        repository.migrate().await.context("配置服務初始化失敗：資料庫遷移錯誤")?;
        
        let cache = GuildConfigCache::new(cache_capacity, cache_ttl_seconds);
        let transaction_manager = ConfigTransactionManager::new(repository.clone(), transaction_timeout);
        let enable_preloading = enable_preloading.unwrap_or(true);
        
        let mut service = Self {
            repository,
            cache,
            transaction_manager,
            enable_preloading,
        };
        
        // 預加載配置
        if enable_preloading {
            service.preload_configs().await?;
        }
        
        info!("公會配置服務初始化完成");
        Ok(service)
    }
    
    /// 獲取公會配置（優先從緩存獲取）
    /// 
    /// # Arguments
    /// 
    /// * `guild_id` - 公會 ID
    pub async fn get_config(&self, guild_id: i64) -> Result<Option<GuildConfig>> {
        debug!("獲取公會配置: guild_id={}", guild_id);
        
        // 首先嘗試從緩存獲取
        if let Some(config) = self.cache.get(guild_id).await {
            debug!("緩存命中: guild_id={}", guild_id);
            return Ok(Some(config));
        }
        
        // 緩存未命中，從資料庫獲取
        let result = self.transaction_manager
            .execute_read_transaction(vec![guild_id], |repo| {
                Box::pin(async move {
                    repo.get_config(guild_id).await
                })
            })
            .await
            .context("獲取配置事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(config_opt) => {
                // 將結果加入緩存
                if let Some(ref config) = config_opt {
                    self.cache.set(config.clone()).await;
                    debug!("配置已加入緩存: guild_id={}", guild_id);
                }
                
                Ok(config_opt)
            }
            TransactionResult::Aborted(error) => {
                error!("獲取配置事務失敗: guild_id={}, error={}", guild_id, error);
                Err(anyhow::anyhow!("獲取配置失敗: {}", error))
            }
            TransactionResult::Timeout => {
                error!("獲取配置事務超時: guild_id={}", guild_id);
                Err(anyhow::anyhow!("獲取配置超時"))
            }
        }
    }
    
    /// 更新公會配置
    /// 
    /// # Arguments
    /// 
    /// * `config` - 要更新的配置
    pub async fn update_config(&self, config: &GuildConfig) -> Result<ConfigUpdateResult> {
        debug!("更新公會配置: guild_id={}", config.guild_id);
        
        let config_clone = config.clone();
        let result = self.transaction_manager
            .execute_write_transaction(vec![config.guild_id], move |repo| {
                Box::pin(async move {
                    repo.upsert_config(&config_clone).await
                })
            })
            .await
            .context("更新配置事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(_) => {
                // 更新緩存
                self.cache.set(config.clone()).await;
                info!("配置更新成功: guild_id={}", config.guild_id);
                Ok(ConfigUpdateResult::Success)
            }
            TransactionResult::Aborted(error) => {
                error!("配置更新事務失敗: guild_id={}, error={}", config.guild_id, error);
                Ok(ConfigUpdateResult::Failed(error))
            }
            TransactionResult::Timeout => {
                error!("配置更新事務超時: guild_id={}", config.guild_id);
                Ok(ConfigUpdateResult::Timeout)
            }
        }
    }
    
    /// 批量更新配置
    /// 
    /// # Arguments
    /// 
    /// * `configs` - 要更新的配置列表
    pub async fn batch_update_configs(&self, configs: Vec<GuildConfig>) -> Result<ConfigUpdateResult> {
        debug!("批量更新配置: {} 個公會", configs.len());
        
        let result = self.transaction_manager
            .batch_update_configs(configs.clone())
            .await
            .context("批量更新配置事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(_) => {
                // 批量更新緩存
                let config_map: HashMap<i64, GuildConfig> = configs
                    .into_iter()
                    .map(|config| (config.guild_id, config))
                    .collect();
                
                self.cache.bulk_set(config_map).await;
                info!("批量配置更新成功");
                Ok(ConfigUpdateResult::Success)
            }
            TransactionResult::Aborted(error) => {
                error!("批量配置更新事務失敗: error={}", error);
                Ok(ConfigUpdateResult::Failed(error))
            }
            TransactionResult::Timeout => {
                error!("批量配置更新事務超時");
                Ok(ConfigUpdateResult::Timeout)
            }
        }
    }
    
    /// 刪除公會配置
    /// 
    /// # Arguments
    /// 
    /// * `guild_id` - 要刪除配置的公會 ID
    pub async fn delete_config(&self, guild_id: i64) -> Result<bool> {
        debug!("刪除公會配置: guild_id={}", guild_id);
        
        let result = self.transaction_manager
            .execute_write_transaction(vec![guild_id], |repo| {
                Box::pin(async move {
                    repo.delete_config(guild_id).await
                })
            })
            .await
            .context("刪除配置事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(deleted) => {
                if deleted {
                    // 從緩存中移除
                    self.cache.remove(guild_id).await;
                    info!("配置刪除成功: guild_id={}", guild_id);
                }
                Ok(deleted)
            }
            TransactionResult::Aborted(error) => {
                error!("配置刪除事務失敗: guild_id={}, error={}", guild_id, error);
                Err(anyhow::anyhow!("刪除配置失敗: {}", error))
            }
            TransactionResult::Timeout => {
                error!("配置刪除事務超時: guild_id={}", guild_id);
                Err(anyhow::anyhow!("刪除配置超時"))
            }
        }
    }
    
    /// 創建背景圖片資源
    /// 
    /// # Arguments
    /// 
    /// * `asset` - 要創建的背景圖片資源
    pub async fn create_background_asset(&self, asset: &BackgroundAsset) -> Result<ConfigUpdateResult> {
        debug!("創建背景圖片資源: id={}", asset.id);
        
        let asset_clone = asset.clone();
        let result = self.transaction_manager
            .execute_write_transaction(vec![], move |repo| {
                Box::pin(async move {
                    repo.create_background_asset(&asset_clone).await
                })
            })
            .await
            .context("創建背景資源事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(_) => {
                info!("背景圖片資源創建成功: id={}", asset.id);
                Ok(ConfigUpdateResult::Success)
            }
            TransactionResult::Aborted(error) => {
                error!("背景圖片資源創建事務失敗: id={}, error={}", asset.id, error);
                Ok(ConfigUpdateResult::Failed(error))
            }
            TransactionResult::Timeout => {
                error!("背景圖片資源創建事務超時: id={}", asset.id);
                Ok(ConfigUpdateResult::Timeout)
            }
        }
    }
    
    /// 獲取背景圖片資源
    /// 
    /// # Arguments
    /// 
    /// * `asset_id` - 背景圖片資源 ID
    pub async fn get_background_asset(&self, asset_id: &str) -> Result<Option<BackgroundAsset>> {
        debug!("獲取背景圖片資源: id={}", asset_id);
        
        let asset_id_str = asset_id.to_string(); // 克隆為 owned 類型
        let asset_id_for_error = asset_id.to_string(); // 為錯誤處理保留副本
        let result = self.transaction_manager
            .execute_read_transaction(vec![], move |repo| {
                Box::pin(async move {
                    repo.get_background_asset(&asset_id_str).await
                })
            })
            .await
            .context("獲取背景資源事務執行失敗")?;
        
        match result {
            TransactionResult::Committed(asset_opt) => {
                Ok(asset_opt)
            }
            TransactionResult::Aborted(error) => {
                error!("獲取背景圖片資源事務失敗: id={}, error={}", asset_id_for_error, error);
                Err(anyhow::anyhow!("獲取背景資源失敗: {}", error))
            }
            TransactionResult::Timeout => {
                error!("獲取背景圖片資源事務超時: id={}", asset_id_for_error);
                Err(anyhow::anyhow!("獲取背景資源超時"))
            }
        }
    }
    
    /// 預加載所有配置到緩存
    pub async fn preload_configs(&self) -> Result<usize> {
        info!("開始預加載配置到緩存...");
        
        let configs = self.repository.get_all_configs()
            .await
            .context("預加載配置失敗：無法獲取所有配置")?;
        
        let count = configs.len();
        self.cache.bulk_set(configs).await;
        
        info!("配置預加載完成: {} 個配置", count);
        Ok(count)
    }
    
    /// 強制刷新緩存
    pub async fn refresh_cache(&self) -> Result<usize> {
        info!("開始刷新緩存...");
        
        // 清空現有緩存
        self.cache.clear().await;
        
        // 重新加載配置
        self.preload_configs().await
    }
    
    /// 失效特定公會的緩存
    /// 
    /// # Arguments
    /// 
    /// * `guild_id` - 公會 ID
    pub async fn invalidate_cache(&self, guild_id: i64) {
        debug!("失效緩存: guild_id={}", guild_id);
        self.cache.remove(guild_id).await;
    }
    
    /// 獲取服務統計信息
    pub async fn get_service_stats(&self) -> Result<ConfigServiceStats> {
        let cache_stats = self.cache.get_stats();
        let transaction_stats = self.transaction_manager.get_transaction_stats().await;
        let (total_configs, configs_with_channel, configs_with_background) = 
            self.repository.get_config_statistics().await?;
        
        Ok(ConfigServiceStats {
            cache_stats,
            transaction_stats,
            total_configs,
            configs_with_channel,
            configs_with_background,
        })
    }
    
    /// 執行維護任務
    /// 
    /// 包括清理未使用的鎖、執行緩存維護等
    pub async fn perform_maintenance(&self) -> Result<()> {
        debug!("執行配置服務維護任務...");
        
        // 清理事務鎖
        self.transaction_manager.cleanup_locks().await;
        
        // 運行緩存待處理任務
        self.cache.run_pending_tasks().await;
        
        // 檢查緩存健康狀態
        let warnings = self.cache.health_check();
        for warning in warnings {
            warn!("緩存健康檢查警告: {}", warning);
        }
        
        info!("配置服務維護任務完成");
        Ok(())
    }
    
    /// 優雅關閉服務
    pub async fn shutdown(&self) -> Result<()> {
        info!("正在關閉配置服務...");
        
        // 執行最後的維護任務
        self.perform_maintenance().await?;
        
        // 清空緩存
        self.cache.clear().await;
        
        info!("配置服務已關閉");
        Ok(())
    }
}

impl Clone for GuildConfigService {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
            cache: self.cache.clone(),
            transaction_manager: ConfigTransactionManager::new(
                self.repository.clone(),
                Some(30), // 使用預設超時時間
            ),
            enable_preloading: self.enable_preloading,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::NamedTempFile;
    
    async fn setup_test_service() -> GuildConfigService {
        let temp_file = NamedTempFile::new().expect("創建臨時文件失敗");
        let database_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("連接測試資料庫失敗");
        
        GuildConfigService::new(
            pool,
            Some(100),   // 小緩存用於測試
            Some(300),   // 5分鐘TTL
            Some(10),    // 10秒事務超時
            Some(true),  // 啟用預加載
        )
        .await
        .expect("創建測試服務失敗")
    }
    
    #[tokio::test]
    async fn test_config_lifecycle() {
        let service = setup_test_service().await;
        let guild_id = 12345i64;
        let channel_id = 67890i64;
        
        // 測試獲取不存在的配置
        let config = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(config.is_none());
        
        // 測試創建配置
        let new_config = GuildConfig::new(guild_id, Some(channel_id), None);
        let update_result = service.update_config(&new_config).await.expect("更新配置失敗");
        assert!(matches!(update_result, ConfigUpdateResult::Success));
        
        // 測試獲取創建的配置
        let retrieved_config = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(retrieved_config.is_some());
        assert_eq!(retrieved_config.unwrap().guild_id, guild_id);
        
        // 測試更新配置
        let mut updated_config = new_config.clone();
        updated_config.update_background(Some("bg_test".to_string()));
        let update_result = service.update_config(&updated_config).await.expect("更新配置失敗");
        assert!(matches!(update_result, ConfigUpdateResult::Success));
        
        // 驗證更新
        let retrieved_updated = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(retrieved_updated.is_some());
        assert_eq!(retrieved_updated.unwrap().background_ref, Some("bg_test".to_string()));
        
        // 測試刪除配置
        let deleted = service.delete_config(guild_id).await.expect("刪除配置失敗");
        assert!(deleted);
        
        // 驗證刪除
        let config_after_delete = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(config_after_delete.is_none());
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let service = setup_test_service().await;
        
        let configs = vec![
            GuildConfig::new(111, Some(222), None),
            GuildConfig::new(333, Some(444), Some("bg_333".to_string())),
            GuildConfig::new(555, Some(666), Some("bg_555".to_string())),
        ];
        
        // 測試批量更新
        let update_result = service.batch_update_configs(configs.clone()).await.expect("批量更新失敗");
        assert!(matches!(update_result, ConfigUpdateResult::Success));
        
        // 驗證所有配置都已創建
        for config in &configs {
            let retrieved = service.get_config(config.guild_id).await.expect("獲取配置失敗");
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().guild_id, config.guild_id);
        }
    }
    
    #[tokio::test]
    async fn test_background_asset_operations() {
        let service = setup_test_service().await;
        
        let asset = BackgroundAsset::new(
            "test_asset".to_string(),
            "/test/path.png".to_string(),
            "image/png".to_string(),
            1024000,
        );
        
        // 測試創建背景資源
        let create_result = service.create_background_asset(&asset).await.expect("創建資源失敗");
        assert!(matches!(create_result, ConfigUpdateResult::Success));
        
        // 測試獲取背景資源
        let retrieved_asset = service.get_background_asset("test_asset").await.expect("獲取資源失敗");
        assert!(retrieved_asset.is_some());
        assert_eq!(retrieved_asset.unwrap().id, "test_asset");
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let service = setup_test_service().await;
        let guild_id = 99999i64;
        
        // 創建配置
        let config = GuildConfig::new(guild_id, Some(11111), None);
        let _ = service.update_config(&config).await.expect("更新配置失敗");
        
        // 第一次獲取（應該從資料庫獲取並緩存）
        let retrieved1 = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(retrieved1.is_some());
        
        // 第二次獲取（應該從緩存獲取）
        let retrieved2 = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(retrieved2.is_some());
        
        // 失效緩存
        service.invalidate_cache(guild_id).await;
        
        // 再次獲取（應該重新從資料庫獲取）
        let retrieved3 = service.get_config(guild_id).await.expect("獲取配置失敗");
        assert!(retrieved3.is_some());
    }
    
    #[tokio::test]
    async fn test_service_stats() {
        let service = setup_test_service().await;
        
        // 創建一些配置
        let configs = vec![
            GuildConfig::new(111, Some(222), None),
            GuildConfig::new(333, Some(444), Some("bg_test".to_string())),
        ];
        
        let _ = service.batch_update_configs(configs).await.expect("批量更新失敗");
        
        // 獲取統計信息
        let stats = service.get_service_stats().await.expect("獲取統計失敗");
        
        assert_eq!(stats.total_configs, 2);
        assert_eq!(stats.configs_with_channel, 2);
        assert_eq!(stats.configs_with_background, 1);
        assert!(stats.cache_stats.entry_count >= 0);
    }
    
    #[tokio::test]
    async fn test_maintenance_and_shutdown() {
        let service = setup_test_service().await;
        
        // 執行維護任務
        let maintenance_result = service.perform_maintenance().await;
        assert!(maintenance_result.is_ok());
        
        // 測試優雅關閉
        let shutdown_result = service.shutdown().await;
        assert!(shutdown_result.is_ok());
    }
}