//! 公會配置緩存管理
//!
//! 此模組實現公會配置的內存緩存機制，使用 Cache-Aside 模式
//! 提高配置存取性能，減少資料庫負載。

use crate::config::models::GuildConfig;
use anyhow::Result;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// 緩存統計信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 緩存命中次數
    pub hits: u64,
    /// 緩存未命中次數
    pub misses: u64,
    /// 緩存條目數量
    pub entry_count: u64,
    /// 緩存命中率（百分比）
    pub hit_rate: f64,
}

/// 公會配置緩存管理器
///
/// 使用 moka 實現高性能的內存緩存，支持 TTL 過期機制和 LRU 淘汰策略。
/// 提供完整的緩存操作和統計功能。
#[derive(Clone)]
pub struct GuildConfigCache {
    cache: Arc<Cache<i64, GuildConfig>>,
}

impl GuildConfigCache {
    /// 創建新的緩存實例
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - 最大緩存條目數 (預設: 10000)
    /// * `ttl_seconds` - 緩存項目生存時間，秒 (預設: 3600，即 1 小時)
    pub fn new(max_capacity: Option<u64>, ttl_seconds: Option<u64>) -> Self {
        let capacity = max_capacity.unwrap_or(10_000);
        let ttl = Duration::from_secs(ttl_seconds.unwrap_or(3600)); // 1 hour default

        info!(
            "初始化公會配置緩存: capacity={}, ttl={}s",
            capacity,
            ttl.as_secs()
        );

        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    /// 獲取配置（緩存優先）
    ///
    /// # Arguments
    ///
    /// * `guild_id` - 公會 ID
    ///
    /// # Returns
    ///
    /// 返回 Option<GuildConfig>，如果緩存中沒有則返回 None
    pub async fn get(&self, guild_id: i64) -> Option<GuildConfig> {
        debug!("嘗試從緩存獲取配置: guild_id={}", guild_id);

        let config = self.cache.get(&guild_id).await;

        if config.is_some() {
            debug!("緩存命中: guild_id={}", guild_id);
        } else {
            debug!("緩存未命中: guild_id={}", guild_id);
        }

        config
    }

    /// 設置配置到緩存
    ///
    /// # Arguments
    ///
    /// * `config` - 要緩存的配置
    pub async fn set(&self, config: GuildConfig) {
        debug!("設置配置到緩存: guild_id={}", config.guild_id);
        self.cache.insert(config.guild_id, config).await;
    }

    /// 從緩存移除配置
    ///
    /// # Arguments
    ///
    /// * `guild_id` - 要移除的公會 ID
    pub async fn remove(&self, guild_id: i64) {
        debug!("從緩存移除配置: guild_id={}", guild_id);
        self.cache.remove(&guild_id).await;
    }

    /// 批量設置配置到緩存（用於預加載）
    ///
    /// # Arguments
    ///
    /// * `configs` - 配置映射，鍵為 guild_id，值為 GuildConfig
    pub async fn bulk_set(&self, configs: std::collections::HashMap<i64, GuildConfig>) {
        info!("批量加載配置到緩存: 數量={}", configs.len());

        for (guild_id, config) in configs {
            self.cache.insert(guild_id, config).await;
        }

        info!("批量加載完成，當前緩存條目數: {}", self.cache.entry_count());
    }

    /// 清空所有緩存
    pub async fn clear(&self) {
        info!("清空所有緩存");
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
    }

    /// 獲取緩存統計信息
    ///
    /// 注意：moka 0.12 版本不提供命中/未命中統計，我們只能提供條目數量
    pub fn get_stats(&self) -> CacheStats {
        // moka 不提供內建的命中率統計，我們需要手動追蹤或使用簡化版本
        let entry_count = self.cache.entry_count();

        CacheStats {
            hits: 0,   // moka 不提供此統計
            misses: 0, // moka 不提供此統計
            entry_count,
            hit_rate: 0.0, // 無法計算，因為沒有命中/未命中數據
        }
    }

    /// 強制運行待處理任務（主要用於測試和手動維護）
    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }

    /// 檢查緩存健康狀態
    ///
    /// # Returns
    ///
    /// 如果緩存使用率過高或其他異常，返回警告信息
    /// 注意：由於 moka 不提供命中率統計，我們只檢查使用率
    pub fn health_check(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        let stats = self.get_stats();

        // 檢查緩存使用率
        let estimated_max_capacity = 10_000u64; // 預設最大容量
        if stats.entry_count > estimated_max_capacity * 90 / 100 {
            warnings.push(format!(
                "緩存使用率過高: {}/{} ({:.1}%)",
                stats.entry_count,
                estimated_max_capacity,
                (stats.entry_count as f64 / estimated_max_capacity as f64) * 100.0
            ));
        }

        warnings
    }
}

/// 緩存失效策略
///
/// 定義何時以及如何使緩存項目失效
pub struct InvalidationStrategy {
    /// 是否在更新時自動失效相關緩存
    pub invalidate_on_update: bool,
    /// 批量失效模式（用於大量更新）
    pub batch_invalidation: bool,
}

impl Default for InvalidationStrategy {
    fn default() -> Self {
        Self {
            invalidate_on_update: true,
            batch_invalidation: false,
        }
    }
}

/// 緩存管理器擴展功能
impl GuildConfigCache {
    /// 條件失效：基於配置更新時間
    ///
    /// # Arguments
    ///
    /// * `guild_id` - 公會 ID
    /// * `last_update` - 最後更新時間戳
    pub async fn invalidate_if_stale(
        &self,
        guild_id: i64,
        last_update: chrono::DateTime<chrono::Utc>,
    ) {
        if let Some(cached_config) = self.get(guild_id).await {
            if cached_config.updated_at < last_update {
                debug!("配置已過期，失效緩存: guild_id={}", guild_id);
                self.remove(guild_id).await;
            }
        }
    }

    /// 預熱緩存（預加載熱門配置）
    ///
    /// # Arguments
    ///
    /// * `guild_ids` - 要預熱的公會 ID 列表
    /// * `load_fn` - 加載配置的異步函數
    pub async fn warm_up<F, Fut>(&self, guild_ids: Vec<i64>, load_fn: F) -> Result<usize>
    where
        F: Fn(i64) -> Fut,
        Fut: std::future::Future<Output = Result<Option<GuildConfig>>>,
    {
        info!("開始緩存預熱: {} 個公會", guild_ids.len());
        let mut warmed_count = 0;

        for guild_id in guild_ids {
            // 只預熱不在緩存中的項目
            if self.get(guild_id).await.is_none() {
                match load_fn(guild_id).await {
                    Ok(Some(config)) => {
                        self.set(config).await;
                        warmed_count += 1;
                    }
                    Ok(None) => {
                        debug!("預熱跳過不存在的配置: guild_id={}", guild_id);
                    }
                    Err(e) => {
                        warn!("預熱配置失敗: guild_id={}, 錯誤={}", guild_id, e);
                    }
                }
            }
        }

        info!("緩存預熱完成: {} 個配置已加載", warmed_count);
        Ok(warmed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = GuildConfigCache::new(Some(100), Some(60)); // 1 minute TTL for testing

        let guild_id = 12345i64;
        let config = GuildConfig::new(guild_id, Some(67890), None);

        // 測試設置和獲取
        cache.set(config.clone()).await;
        let retrieved = cache.get(guild_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().guild_id, guild_id);

        // 測試移除
        cache.remove(guild_id).await;
        let retrieved = cache.get(guild_id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let cache = GuildConfigCache::new(Some(100), Some(3600));

        let config1 = GuildConfig::new(111, Some(222), None);
        let config2 = GuildConfig::new(333, Some(444), None);

        // 初始統計應該為空
        let stats = cache.get_stats();
        assert_eq!(stats.entry_count, 0);

        // 設置一些配置
        cache.set(config1.clone()).await;
        cache.set(config2.clone()).await;
        cache.run_pending_tasks().await; // 確保緩存操作完成

        // 測試命中
        let result1 = cache.get(111).await;
        assert!(result1.is_some());

        let result2 = cache.get(333).await;
        assert!(result2.is_some());

        // 測試未命中
        let result3 = cache.get(999).await;
        assert!(result3.is_none());

        let final_stats = cache.get_stats();
        assert_eq!(final_stats.entry_count, 2);
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let cache = GuildConfigCache::new(Some(100), Some(3600));

        let mut configs = std::collections::HashMap::new();
        configs.insert(111, GuildConfig::new(111, Some(222), None));
        configs.insert(333, GuildConfig::new(333, Some(444), None));
        configs.insert(555, GuildConfig::new(555, Some(666), None));

        // 批量設置
        cache.bulk_set(configs).await;
        cache.run_pending_tasks().await; // 確保操作完成

        // 驗證所有配置都已緩存
        assert!(cache.get(111).await.is_some());
        assert!(cache.get(333).await.is_some());
        assert!(cache.get(555).await.is_some());

        let stats = cache.get_stats();
        assert_eq!(stats.entry_count, 3);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = GuildConfigCache::new(Some(100), Some(3600));

        // 添加一些配置
        cache.set(GuildConfig::new(111, Some(222), None)).await;
        cache.set(GuildConfig::new(333, Some(444), None)).await;
        cache.run_pending_tasks().await; // 確保操作完成

        let stats_before = cache.get_stats();
        assert_eq!(stats_before.entry_count, 2);

        // 清空緩存
        cache.clear().await;

        let stats_after = cache.get_stats();
        assert_eq!(stats_after.entry_count, 0);

        // 驗證配置已被清除
        assert!(cache.get(111).await.is_none());
        assert!(cache.get(333).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_if_stale() {
        let cache = GuildConfigCache::new(Some(100), Some(3600));

        let guild_id = 12345i64;
        let mut config = GuildConfig::new(guild_id, Some(67890), None);
        config.updated_at = Utc::now() - chrono::Duration::hours(1); // 1 hour ago

        cache.set(config.clone()).await;
        assert!(cache.get(guild_id).await.is_some());

        // 模擬更新時間更新
        let new_update_time = Utc::now() - chrono::Duration::minutes(30); // 30 minutes ago
        cache.invalidate_if_stale(guild_id, new_update_time).await;

        // 配置應該被失效（因為緩存中的配置更舊）
        assert!(cache.get(guild_id).await.is_none());
    }

    #[tokio::test]
    async fn test_warm_up() {
        let cache = GuildConfigCache::new(Some(100), Some(3600));

        // 模擬加載函數
        let load_fn = |guild_id: i64| async move {
            if guild_id == 999 {
                Ok(None) // 模擬不存在的配置
            } else {
                Ok(Some(GuildConfig::new(
                    guild_id,
                    Some(guild_id + 1000),
                    None,
                )))
            }
        };

        let guild_ids = vec![111, 222, 333, 999]; // 包含一個不存在的
        let warmed_count = cache.warm_up(guild_ids, load_fn).await.expect("預熱失敗");

        assert_eq!(warmed_count, 3); // 應該預熱 3 個配置（排除不存在的）

        // 驗證配置已被加載到緩存
        assert!(cache.get(111).await.is_some());
        assert!(cache.get(222).await.is_some());
        assert!(cache.get(333).await.is_some());
        assert!(cache.get(999).await.is_none());
    }

    #[tokio::test]
    async fn test_health_check() {
        let cache = GuildConfigCache::new(Some(10), Some(3600)); // 小容量用於測試

        // 初始狀態應該沒有警告
        let warnings = cache.health_check();
        assert!(warnings.is_empty());

        // 模擬低命中率（需要大量請求才能觸發）
        // 這個測試可能需要調整閾值或生成更多請求
    }
}
