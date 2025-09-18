use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

/// 緩存項目
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// 緩存的值
    pub value: T,
    /// 過期時間
    pub expires_at: DateTime<Utc>,
    /// 最後訪問時間
    pub last_accessed: DateTime<Utc>,
    /// 訪問次數
    pub access_count: u64,
}

impl<T> CacheEntry<T> {
    /// 創建新的緩存項目
    pub fn new(value: T, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            value,
            expires_at: now + Duration::seconds(ttl_seconds),
            last_accessed: now,
            access_count: 0,
        }
    }

    /// 檢查是否過期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 更新訪問時間和計數
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

/// LRU緩存實現
struct LruCache<T> {
    /// 存儲實際數據
    data: HashMap<String, CacheEntry<T>>,
    /// 最大容量
    max_capacity: usize,
    /// 默認TTL（秒）
    default_ttl: i64,
}

impl<T: Clone> LruCache<T> {
    /// 創建新的LRU緩存
    pub fn new(max_capacity: usize, default_ttl: i64) -> Self {
        Self {
            data: HashMap::with_capacity(max_capacity),
            max_capacity,
            default_ttl,
        }
    }

    /// 插入數據
    pub fn insert(&mut self, key: String, value: T) -> bool {
        // 如果需要，先進行清理
        if self.data.len() >= self.max_capacity {
            self.evict_lru();
        }

        let entry = CacheEntry::new(value, self.default_ttl);
        self.data.insert(key, entry);
        true
    }

    /// 獲取數據
    pub fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get_mut(key) {
            if entry.is_expired() {
                self.data.remove(key);
                return None;
            }
            entry.touch();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// 刪除數據
    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// 清理過期項目
    pub fn cleanup_expired(&mut self) -> usize {
        let before_count = self.data.len();
        self.data.retain(|_, entry| !entry.is_expired());
        before_count - self.data.len()
    }

    /// LRU淘汰：移除最久未訪問的項目
    fn evict_lru(&mut self) {
        if self.data.is_empty() {
            return;
        }

        // 找到最久未訪問的項目
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Utc::now();

        for (key, entry) in &self.data {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            self.data.remove(&key);
            debug!("LRU淘汰緩存項目: {}", key);
        }
    }

    /// 獲取緩存統計
    pub fn stats(&self) -> CacheStats {
        let mut total_access_count = 0;
        let mut expired_count = 0;
        let now = Utc::now();

        for entry in self.data.values() {
            total_access_count += entry.access_count;
            if entry.expires_at < now {
                expired_count += 1;
            }
        }

        CacheStats {
            total_entries: self.data.len(),
            max_capacity: self.max_capacity,
            expired_entries: expired_count,
            total_access_count,
        }
    }

    /// 估算記憶體使用（簡單估算）
    pub fn estimated_memory_usage(&self) -> u64 {
        // 每個String key估算50字節
        // 每個CacheEntry結構體估算200字節
        // 實際值大小無法精確估算，假設平均1KB
        (self.data.len() as u64) * (50 + 200 + 1024)
    }
}

/// 緩存統計信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 總項目數
    pub total_entries: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 過期項目數
    pub expired_entries: usize,
    /// 總訪問次數
    pub total_access_count: u64,
}

/// 緩存管理器
pub struct CacheManager {
    /// 字體緩存
    font_cache: Arc<RwLock<LruCache<Vec<u8>>>>,
    /// 背景元數據緩存
    background_metadata_cache: Arc<RwLock<LruCache<String>>>,
    /// 頭像緩存
    avatar_cache: Arc<RwLock<LruCache<Vec<u8>>>>,
    /// 清理任務狀態
    cleanup_running: Arc<Mutex<bool>>,
}

impl CacheManager {
    /// 創建新的緩存管理器
    ///
    /// # Arguments
    /// * `max_entries` - 每個緩存的最大項目數
    /// * `ttl_seconds` - 默認TTL（秒）
    pub async fn new(max_entries: usize, ttl_seconds: i64) -> Result<Self> {
        info!(
            "初始化緩存管理器: max_entries={}, ttl={}s",
            max_entries, ttl_seconds
        );

        let cache_manager = Self {
            font_cache: Arc::new(RwLock::new(LruCache::new(max_entries / 3, ttl_seconds))),
            background_metadata_cache: Arc::new(RwLock::new(LruCache::new(
                max_entries / 3,
                ttl_seconds,
            ))),
            avatar_cache: Arc::new(RwLock::new(LruCache::new(max_entries / 3, ttl_seconds * 2))), // 頭像緩存更久
            cleanup_running: Arc::new(Mutex::new(false)),
        };

        Ok(cache_manager)
    }

    /// 緩存字體數據
    pub async fn cache_font(&self, font_name: &str, font_data: Vec<u8>) -> Result<()> {
        debug!("緩存字體: {}, size={} bytes", font_name, font_data.len());
        let mut cache = self.font_cache.write().await;
        cache.insert(font_name.to_string(), font_data);
        Ok(())
    }

    /// 獲取字體數據
    pub async fn get_font(&self, font_name: &str) -> Option<Vec<u8>> {
        debug!("獲取字體: {}", font_name);
        let mut cache = self.font_cache.write().await;
        cache.get(font_name)
    }

    /// 緩存背景元數據
    pub async fn cache_background_metadata(&self, asset_id: &str, metadata: String) -> Result<()> {
        debug!("緩存背景元數據: {}", asset_id);
        let mut cache = self.background_metadata_cache.write().await;
        cache.insert(asset_id.to_string(), metadata);
        Ok(())
    }

    /// 獲取背景元數據
    pub async fn get_background_metadata(&self, asset_id: &str) -> Option<String> {
        debug!("獲取背景元數據: {}", asset_id);
        let mut cache = self.background_metadata_cache.write().await;
        cache.get(asset_id)
    }

    /// 緩存頭像數據
    pub async fn cache_avatar(&self, user_id: &str, avatar_data: Vec<u8>) -> Result<()> {
        debug!("緩存頭像: {}, size={} bytes", user_id, avatar_data.len());
        let mut cache = self.avatar_cache.write().await;
        cache.insert(format!("avatar_{}", user_id), avatar_data);
        Ok(())
    }

    /// 獲取頭像數據
    pub async fn get_avatar(&self, user_id: &str) -> Option<Vec<u8>> {
        debug!("獲取頭像: {}", user_id);
        let mut cache = self.avatar_cache.write().await;
        cache.get(&format!("avatar_{}", user_id))
    }

    /// 清理過期緩存
    pub async fn cleanup(&self) -> Result<()> {
        // 防止重複執行清理
        {
            let mut cleanup_guard = self.cleanup_running.lock().await;
            if *cleanup_guard {
                debug!("緩存清理已在運行中，跳過");
                return Ok(());
            }
            *cleanup_guard = true;
        }

        info!("開始清理過期緩存");

        // 清理字體緩存
        let font_cleaned = {
            let mut cache = self.font_cache.write().await;
            cache.cleanup_expired()
        };

        // 清理背景元數據緩存
        let bg_meta_cleaned = {
            let mut cache = self.background_metadata_cache.write().await;
            cache.cleanup_expired()
        };

        // 清理頭像緩存
        let avatar_cleaned = {
            let mut cache = self.avatar_cache.write().await;
            cache.cleanup_expired()
        };

        let total_cleaned = font_cleaned + bg_meta_cleaned + avatar_cleaned;
        info!("緩存清理完成: 清理了 {} 個過期項目", total_cleaned);

        // 重置清理狀態
        {
            let mut cleanup_guard = self.cleanup_running.lock().await;
            *cleanup_guard = false;
        }

        Ok(())
    }

    /// 清空所有緩存
    pub async fn clear_all(&self) -> Result<()> {
        info!("清空所有緩存");

        {
            let mut cache = self.font_cache.write().await;
            *cache = LruCache::new(cache.max_capacity, cache.default_ttl);
        }

        {
            let mut cache = self.background_metadata_cache.write().await;
            *cache = LruCache::new(cache.max_capacity, cache.default_ttl);
        }

        {
            let mut cache = self.avatar_cache.write().await;
            *cache = LruCache::new(cache.max_capacity, cache.default_ttl);
        }

        info!("所有緩存已清空");
        Ok(())
    }

    /// 獲取緩存統計信息
    pub async fn get_stats(&self) -> CombinedCacheStats {
        let font_stats = {
            let cache = self.font_cache.read().await;
            cache.stats()
        };

        let bg_meta_stats = {
            let cache = self.background_metadata_cache.read().await;
            cache.stats()
        };

        let avatar_stats = {
            let cache = self.avatar_cache.read().await;
            cache.stats()
        };

        CombinedCacheStats {
            font_cache: font_stats,
            background_metadata_cache: bg_meta_stats,
            avatar_cache: avatar_stats,
        }
    }

    /// 獲取記憶體使用估算
    pub async fn get_memory_usage(&self) -> u64 {
        let font_mem = {
            let cache = self.font_cache.read().await;
            cache.estimated_memory_usage()
        };

        let bg_meta_mem = {
            let cache = self.background_metadata_cache.read().await;
            cache.estimated_memory_usage()
        };

        let avatar_mem = {
            let cache = self.avatar_cache.read().await;
            cache.estimated_memory_usage()
        };

        font_mem + bg_meta_mem + avatar_mem
    }
}

/// 組合緩存統計
#[derive(Debug, Clone)]
pub struct CombinedCacheStats {
    pub font_cache: CacheStats,
    pub background_metadata_cache: CacheStats,
    pub avatar_cache: CacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let manager = CacheManager::new(100, 3600).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_font_cache() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        let font_data = vec![1, 2, 3, 4, 5];
        let font_name = "test_font";

        // 測試緩存
        let result = manager.cache_font(font_name, font_data.clone()).await;
        assert!(result.is_ok());

        // 測試讀取
        let cached_data = manager.get_font(font_name).await;
        assert_eq!(cached_data, Some(font_data));

        // 測試不存在的字體
        let missing_data = manager.get_font("missing_font").await;
        assert_eq!(missing_data, None);
    }

    #[tokio::test]
    async fn test_background_metadata_cache() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        let metadata = "test metadata";
        let asset_id = "test_asset_123";

        // 測試緩存
        let result = manager
            .cache_background_metadata(asset_id, metadata.to_string())
            .await;
        assert!(result.is_ok());

        // 測試讀取
        let cached_metadata = manager.get_background_metadata(asset_id).await;
        assert_eq!(cached_metadata, Some(metadata.to_string()));
    }

    #[tokio::test]
    async fn test_avatar_cache() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        let avatar_data = vec![10, 20, 30, 40, 50];
        let user_id = "user_456";

        // 測試緩存
        let result = manager.cache_avatar(user_id, avatar_data.clone()).await;
        assert!(result.is_ok());

        // 測試讀取
        let cached_avatar = manager.get_avatar(user_id).await;
        assert_eq!(cached_avatar, Some(avatar_data));
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut lru = LruCache::new(2, 3600); // 只允許2個項目

        // 添加兩個項目
        lru.insert("key1".to_string(), "value1".to_string());
        lru.insert("key2".to_string(), "value2".to_string());

        // 添加第三個項目應該觸發LRU淘汰
        lru.insert("key3".to_string(), "value3".to_string());

        // key1應該被淘汰
        assert!(lru.get("key1").is_none());
        assert!(lru.get("key2").is_some());
        assert!(lru.get("key3").is_some());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let mut lru = LruCache::new(10, 1); // 1秒TTL

        lru.insert("test_key".to_string(), "test_value".to_string());

        // 立即讀取應該成功
        assert!(lru.get("test_key").is_some());

        // 等待超過TTL
        sleep(TokioDuration::from_secs(2)).await;

        // 現在應該過期了
        assert!(lru.get("test_key").is_none());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let manager = CacheManager::new(100, 1).await.unwrap(); // 1秒TTL

        // 添加一些數據
        manager.cache_font("font1", vec![1, 2, 3]).await.unwrap();
        manager
            .cache_background_metadata("bg1", "metadata".to_string())
            .await
            .unwrap();
        manager.cache_avatar("user1", vec![4, 5, 6]).await.unwrap();

        // 等待過期
        sleep(TokioDuration::from_secs(2)).await;

        // 執行清理
        let result = manager.cleanup().await;
        assert!(result.is_ok());

        // 確認數據已被清理
        assert!(manager.get_font("font1").await.is_none());
        assert!(manager.get_background_metadata("bg1").await.is_none());
        assert!(manager.get_avatar("user1").await.is_none());
    }

    #[tokio::test]
    async fn test_clear_all() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        // 添加一些數據
        manager.cache_font("font1", vec![1, 2, 3]).await.unwrap();
        manager
            .cache_background_metadata("bg1", "metadata".to_string())
            .await
            .unwrap();
        manager.cache_avatar("user1", vec![4, 5, 6]).await.unwrap();

        // 確認數據存在
        assert!(manager.get_font("font1").await.is_some());
        assert!(manager.get_background_metadata("bg1").await.is_some());
        assert!(manager.get_avatar("user1").await.is_some());

        // 清空緩存
        let result = manager.clear_all().await;
        assert!(result.is_ok());

        // 確認數據已被清空
        assert!(manager.get_font("font1").await.is_none());
        assert!(manager.get_background_metadata("bg1").await.is_none());
        assert!(manager.get_avatar("user1").await.is_none());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        // 添加一些數據
        manager.cache_font("font1", vec![1, 2, 3]).await.unwrap();
        manager
            .cache_background_metadata("bg1", "metadata".to_string())
            .await
            .unwrap();

        let stats = manager.get_stats().await;

        assert!(stats.font_cache.total_entries > 0);
        assert!(stats.background_metadata_cache.total_entries > 0);
        assert_eq!(stats.avatar_cache.total_entries, 0);
    }

    #[tokio::test]
    async fn test_memory_usage() {
        let manager = CacheManager::new(100, 3600).await.unwrap();

        let initial_usage = manager.get_memory_usage().await;

        // 添加一些數據
        manager.cache_font("font1", vec![1; 1000]).await.unwrap();

        let after_usage = manager.get_memory_usage().await;
        assert!(after_usage > initial_usage);
    }
}
