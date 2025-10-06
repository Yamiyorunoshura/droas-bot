// Cache Module - 快取層
// 提供基本的快取功能，為餘額查詢優化

use std::collections::HashMap;
use std::time::{Duration, Instant};
use bigdecimal::BigDecimal;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn};

/// 快取項目
#[derive(Debug, Clone)]
pub struct CacheItem<T> {
    pub value: T,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl<T> CacheItem<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// 記憶體快取實作
pub struct MemoryCache<T: Clone> {
    cache: RwLock<HashMap<String, CacheItem<T>>>,
    default_ttl: Duration,
}

impl<T: Clone> MemoryCache<T> {
    pub fn new(default_ttl: Duration) -> Self {
        info!("Creating MemoryCache with default TTL: {:?}", default_ttl);
        Self {
            cache: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }

    /// 獲取快取項目
    pub async fn get(&self, key: &str) -> Option<T> {
        debug!("Getting cache item for key: {}", key);

        let cache = self.cache.read().await;

        if let Some(item) = cache.get(key) {
            if !item.is_expired() {
                debug!("Cache hit for key: {}", key);
                return Some(item.value.clone());
            } else {
                debug!("Cache expired for key: {}", key);
            }
        } else {
            debug!("Cache miss for key: {}", key);
        }

        None
    }

    /// 設置快取項目
    pub async fn set(&self, key: String, value: T) {
        debug!("Setting cache item for key: {}", key);

        let item = CacheItem::new(value, self.default_ttl);

        let mut cache = self.cache.write().await;
        cache.insert(key, item);

        debug!("Cache item set successfully");
    }

    /// 設置快取項目（帶自定義 TTL）
    pub async fn set_with_ttl(&self, key: String, value: T, ttl: Duration) {
        debug!("Setting cache item for key: {} with custom TTL: {:?}", key, ttl);

        let item = CacheItem::new(value, ttl);

        let mut cache = self.cache.write().await;
        cache.insert(key, item);

        debug!("Cache item set successfully with custom TTL");
    }

    /// 刪除快取項目
    pub async fn remove(&self, key: &str) -> Option<T> {
        debug!("Removing cache item for key: {}", key);

        let mut cache = self.cache.write().await;
        cache.remove(key).map(|item| {
            debug!("Cache item removed successfully");
            item.value
        })
    }

    /// 清理過期的快取項目
    pub async fn cleanup_expired(&self) {
        debug!("Cleaning up expired cache items");

        let mut cache = self.cache.write().await;
        let initial_size = cache.len();

        cache.retain(|key, item| {
            if item.is_expired() {
                debug!("Removing expired cache item: {}", key);
                false
            } else {
                true
            }
        });

        let final_size = cache.len();
        if initial_size != final_size {
            info!("Cleaned up {} expired cache items", initial_size - final_size);
        }
    }

    /// 清空所有快取
    pub async fn clear(&self) {
        debug!("Clearing all cache items");

        let mut cache = self.cache.write().await;
        cache.clear();

        info!("All cache items cleared");
    }

    /// 獲取快取統計信息
    pub async fn stats(&self) -> MemoryCacheStats {
        let cache = self.cache.read().await;
        let total_items = cache.len();
        let expired_items = cache.values()
            .filter(|item| item.is_expired())
            .count();

        MemoryCacheStats {
            total_items,
            active_items: total_items - expired_items,
            expired_items,
        }
    }
}

/// 記憶體快取統計信息
#[derive(Debug, Clone)]
pub struct MemoryCacheStats {
    pub total_items: usize,
    pub active_items: usize,
    pub expired_items: usize,
}

/// Redis 快取實作
pub struct RedisCache {
    #[allow(dead_code)]
    client: redis::Client,
    connection: redis::aio::MultiplexedConnection,
    default_ttl: Duration,
}

impl RedisCache {
    /// 創建新的 Redis 快取實例
    pub async fn new(connection_string: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating RedisCache with connection: {}", connection_string);

        let client = redis::Client::open(connection_string)?;
        let connection = client.get_multiplexed_async_connection().await?;

        info!("Redis connection established successfully");

        Ok(Self {
            client,
            connection,
            default_ttl: Duration::from_secs(300), // 預設 5 分鐘
        })
    }

    /// 創建帶自定義 TTL 的 Redis 快取實例
    pub async fn new_with_ttl(connection_string: &str, default_ttl: Duration) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating RedisCache with connection: {} and TTL: {:?}", connection_string, default_ttl);

        let mut cache = Self::new(connection_string).await?;
        cache.default_ttl = default_ttl;

        Ok(cache)
    }

    /// 獲取快取項目
    pub async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Getting Redis cache item for key: {}", key);

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<Option<String>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(Some(value)) => {
                debug!("Redis cache hit for key: {}", key);
                Ok(Some(value))
            }
            Ok(None) => {
                debug!("Redis cache miss for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                error!("Redis cache error for key {}: {}", key, e);
                Err(Box::new(e))
            }
        }
    }

    /// 設置快取項目（帶預設 TTL）
    pub async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Setting Redis cache item for key: {}", key);

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<()> = redis::cmd("SETEX")
            .arg(key)
            .arg(self.default_ttl.as_secs())
            .arg(value)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                debug!("Redis cache item set successfully for key: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set Redis cache item for key {}: {}", key, e);
                Err(Box::new(e))
            }
        }
    }

    /// 設置快取項目（帶自定義 TTL）
    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Setting Redis cache item for key: {} with custom TTL: {:?}", key, ttl);

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<()> = redis::cmd("SETEX")
            .arg(key)
            .arg(ttl.as_secs())
            .arg(value)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                debug!("Redis cache item set successfully with custom TTL for key: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set Redis cache item with custom TTL for key {}: {}", key, e);
                Err(Box::new(e))
            }
        }
    }

    /// 刪除快取項目
    pub async fn remove(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Removing Redis cache item for key: {}", key);

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<i32> = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(deleted_count) => {
                debug!("Redis cache item removed for key: {}, deleted count: {}", key, deleted_count);
                Ok(deleted_count > 0)
            }
            Err(e) => {
                error!("Failed to remove Redis cache item for key {}: {}", key, e);
                Err(Box::new(e))
            }
        }
    }

    /// 檢查鍵是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Checking if Redis cache key exists: {}", key);

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<i32> = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(exists) => {
                debug!("Redis cache key {} exists: {}", key, exists > 0);
                Ok(exists > 0)
            }
            Err(e) => {
                error!("Failed to check Redis cache key existence for {}: {}", key, e);
                Err(Box::new(e))
            }
        }
    }

    /// 獲取快取統計信息
    pub async fn stats(&self) -> Result<RedisCacheStats, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Getting Redis cache statistics");

        let mut conn = self.connection.clone();

        // 獲取 Redis info
        let info_result: redis::RedisResult<String> = redis::cmd("INFO")
            .query_async(&mut conn)
            .await;

        match info_result {
            Ok(info) => {
                // 解析 Redis info 來獲取統計信息
                let stats = RedisCacheStats::from_redis_info(&info);
                debug!("Redis cache stats retrieved: {:?}", stats);
                Ok(stats)
            }
            Err(e) => {
                error!("Failed to get Redis cache statistics: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// 測試 Redis 連接
    pub async fn ping(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Testing Redis connection");

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<String> = redis::cmd("PING")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(response) => {
                let is_healthy = response == "PONG";
                debug!("Redis ping response: {}, healthy: {}", response, is_healthy);
                Ok(is_healthy)
            }
            Err(e) => {
                error!("Redis ping failed: {}", e);
                Err(Box::new(e))
            }
        }
    }
}

/// Redis 快取統計信息
#[derive(Debug, Clone)]
pub struct RedisCacheStats {
    pub used_memory: String,
    pub used_memory_human: String,
    pub used_memory_peak: String,
    pub used_memory_peak_human: String,
    pub connected_clients: u64,
    pub total_commands_processed: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub hit_rate: f64,
}

impl RedisCacheStats {
    fn from_redis_info(info: &str) -> Self {
        let mut stats = Self {
            used_memory: "0".to_string(),
            used_memory_human: "0B".to_string(),
            used_memory_peak: "0".to_string(),
            used_memory_peak_human: "0B".to_string(),
            connected_clients: 0,
            total_commands_processed: 0,
            keyspace_hits: 0,
            keyspace_misses: 0,
            hit_rate: 0.0,
        };

        for line in info.lines() {
            if line.starts_with("used_memory:") {
                stats.used_memory = line.split(':').nth(1).unwrap_or("0").to_string();
            } else if line.starts_with("used_memory_human:") {
                stats.used_memory_human = line.split(':').nth(1).unwrap_or("0B").to_string();
            } else if line.starts_with("used_memory_peak:") {
                stats.used_memory_peak = line.split(':').nth(1).unwrap_or("0").to_string();
            } else if line.starts_with("used_memory_peak_human:") {
                stats.used_memory_peak_human = line.split(':').nth(1).unwrap_or("0B").to_string();
            } else if line.starts_with("connected_clients:") {
                stats.connected_clients = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("total_commands_processed:") {
                stats.total_commands_processed = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("keyspace_hits:") {
                stats.keyspace_hits = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("keyspace_misses:") {
                stats.keyspace_misses = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }

        let total_requests = stats.keyspace_hits + stats.keyspace_misses;
        if total_requests > 0 {
            stats.hit_rate = stats.keyspace_hits as f64 / total_requests as f64;
        }

        stats
    }
}

/// 快取統計信息（支援記憶體和 Redis 快取）
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_items: usize,
    pub active_items: usize,
    pub expired_items: usize,
    pub redis_stats: Option<RedisCacheStats>,
}

/// 快取類型
#[derive(Debug, Clone)]
pub enum CacheType {
    Memory,
    Redis,
    Hybrid,
}

/// 餘額快取服務（支援記憶體和 Redis 快取）- 性能優化版本
pub struct BalanceCache {
    memory_cache: MemoryCache<BigDecimal>,
    redis_cache: Option<RedisCache>,
    cache_type: CacheType,
}

impl BalanceCache {
    /// 創建記憶體快取
    pub fn new() -> Self {
        info!("Creating optimized BalanceCache with memory backend");
        Self {
            memory_cache: MemoryCache::new(Duration::from_secs(300)),
            redis_cache: None,
            cache_type: CacheType::Memory,
        }
    }

    /// 創建帶自定義 TTL 的記憶體快取
    pub fn new_with_ttl(ttl: Duration) -> Self {
        info!("Creating optimized BalanceCache with memory backend and custom TTL: {:?}", ttl);
        Self {
            memory_cache: MemoryCache::new(ttl),
            redis_cache: None,
            cache_type: CacheType::Memory,
        }
    }

    /// 從 Redis 快取創建
    pub async fn from_redis(redis_cache: RedisCache) -> Self {
        info!("Creating optimized BalanceCache with Redis backend");
        Self {
            memory_cache: MemoryCache::new(Duration::from_secs(300)), // 保持記憶體快取作為備用
            redis_cache: Some(redis_cache),
            cache_type: CacheType::Redis,
        }
    }

    /// 從 Redis 快取創建（帶自定義 TTL）
    pub async fn from_redis_with_ttl(redis_cache: RedisCache, ttl: Duration) -> Self {
        info!("Creating optimized BalanceCache with Redis backend and custom TTL: {:?}", ttl);
        Self {
            memory_cache: MemoryCache::new(ttl),
            redis_cache: Some(redis_cache),
            cache_type: CacheType::Redis,
        }
    }

    /// 生成餘額快取鍵
    pub fn balance_key(user_id: u64) -> String {
        format!("balance:{}", user_id)
    }

    /// 獲取用戶餘額快取
    pub async fn get_balance(&self, user_id: u64) -> Option<BigDecimal> {
        let key = Self::balance_key(user_id);
        debug!("Getting balance cache for user ID: {}", user_id);

        match self.cache_type {
            CacheType::Memory => {
                self.memory_cache.get(&key).await
            }
            CacheType::Redis => {
                if let Some(redis_cache) = &self.redis_cache {
                    match redis_cache.get(&key).await {
                        Ok(Some(value)) => {
                            debug!("Redis cache hit for user ID: {}", user_id);
                            // 將 Redis 中的字符串轉換回 BigDecimal
                            match value.parse::<BigDecimal>() {
                                Ok(balance) => Some(balance),
                                Err(e) => {
                                    error!("Failed to parse cached balance for user {}: {}", user_id, e);
                                    None
                                }
                            }
                        }
                        Ok(None) => {
                            debug!("Redis cache miss for user ID: {}", user_id);
                            None
                        }
                        Err(e) => {
                            error!("Redis cache error for user {}: {}", user_id, e);
                            warn!("Falling back to memory cache for user ID: {}", user_id);
                            // Redis 錯誤時回退到記憶體快取
                            self.memory_cache.get(&key).await
                        }
                    }
                } else {
                    None
                }
            }
            CacheType::Hybrid => {
                // 先檢查記憶體快取
                if let Some(balance) = self.memory_cache.get(&key).await {
                    debug!("Memory cache hit for user ID: {}", user_id);
                    return Some(balance);
                }

                // 記憶體快取未命中，檢查 Redis
                if let Some(redis_cache) = &self.redis_cache {
                    match redis_cache.get(&key).await {
                        Ok(Some(value)) => {
                            debug!("Redis cache hit for user ID: {}", user_id);
                            if let Ok(balance) = value.parse::<BigDecimal>() {
                                // 將 Redis 中的資料也快取到記憶體中
                                self.memory_cache.set(key.clone(), balance.clone()).await;
                                Some(balance)
                            } else {
                                error!("Failed to parse cached balance for user {}: {}", user_id, value);
                                None
                            }
                        }
                        Ok(None) => {
                            debug!("Both cache layers miss for user ID: {}", user_id);
                            None
                        }
                        Err(e) => {
                            error!("Redis cache error for user {}: {}", user_id, e);
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    /// 設置用戶餘額快取
    pub async fn set_balance(&self, user_id: u64, balance: BigDecimal) {
        let key = Self::balance_key(user_id);
        debug!("Setting balance cache for user ID: {}", user_id);

        // 設置記憶體快取
        self.memory_cache.set(key.clone(), balance.clone()).await;

        // 如果使用 Redis，同時設置 Redis 快取
        if let Some(redis_cache) = &self.redis_cache {
            match redis_cache.set(&key, &balance.to_string()).await {
                Ok(_) => {
                    debug!("Balance cached to Redis for user ID: {}", user_id);
                }
                Err(e) => {
                    error!("Failed to cache balance to Redis for user {}: {}", user_id, e);
                    warn!("Balance only cached to memory for user ID: {}", user_id);
                }
            }
        }
    }

    /// 設置用戶餘額快取（帶自定義 TTL）
    pub async fn set_balance_with_ttl(&self, user_id: u64, balance: BigDecimal, ttl: Duration) {
        let key = Self::balance_key(user_id);
        debug!("Setting balance cache for user ID: {} with custom TTL: {:?}", user_id, ttl);

        // 設置記憶體快取
        self.memory_cache.set_with_ttl(key.clone(), balance.clone(), ttl).await;

        // 如果使用 Redis，同時設置 Redis 快取
        if let Some(redis_cache) = &self.redis_cache {
            match redis_cache.set_with_ttl(&key, &balance.to_string(), ttl).await {
                Ok(_) => {
                    debug!("Balance cached to Redis with custom TTL for user ID: {}", user_id);
                }
                Err(e) => {
                    error!("Failed to cache balance to Redis with custom TTL for user {}: {}", user_id, e);
                    warn!("Balance only cached to memory for user ID: {}", user_id);
                }
            }
        }
    }

    /// 刪除用戶餘額快取
    pub async fn remove_balance(&self, user_id: u64) -> Option<BigDecimal> {
        let key = Self::balance_key(user_id);
        debug!("Removing balance cache for user ID: {}", user_id);

        // 先從記憶體快取中刪除
        let memory_result = self.memory_cache.remove(&key).await;

        // 如果使用 Redis，同時從 Redis 刪除
        if let Some(redis_cache) = &self.redis_cache {
            match redis_cache.remove(&key).await {
                Ok(deleted) => {
                    if deleted {
                        debug!("Balance removed from Redis for user ID: {}", user_id);
                    } else {
                        debug!("Balance not found in Redis for user ID: {}", user_id);
                    }
                }
                Err(e) => {
                    error!("Failed to remove balance from Redis for user {}: {}", user_id, e);
                }
            }
        }

        memory_result
    }

    /// 清理過期的餘額快取
    pub async fn cleanup(&self) {
        debug!("Cleaning up expired balance cache items");

        self.memory_cache.cleanup_expired().await;

        // Redis 會自動處理過期，所以不需要額外的清理邏輯
        if self.redis_cache.is_some() {
            debug!("Redis handles TTL expiration automatically");
        }
    }

    /// 獲取快取統計信息
    pub async fn stats(&self) -> CacheStats {
        debug!("Getting balance cache statistics");

        let memory_stats = self.memory_cache.stats().await;
        let redis_stats = if let Some(redis_cache) = &self.redis_cache {
            match redis_cache.stats().await {
                Ok(stats) => Some(stats),
                Err(e) => {
                    error!("Failed to get Redis cache statistics: {}", e);
                    None
                }
            }
        } else {
            None
        };

        CacheStats {
            total_items: memory_stats.total_items,
            active_items: memory_stats.active_items,
            expired_items: memory_stats.expired_items,
            redis_stats,
        }
    }

    /// 測試 Redis 連接健康狀態
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(redis_cache) = &self.redis_cache {
            redis_cache.ping().await
        } else {
            // 如果不使用 Redis，總是返回健康
            Ok(true)
        }
    }
}

impl Default for BalanceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_memory_cache_basic_operations() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        // 測試設置和獲取
        cache.set("test_key".to_string(), 42).await;
        assert_eq!(cache.get("test_key").await, Some(42));

        // 測試不存在的鍵
        assert_eq!(cache.get("non_existent").await, None);
    }

    #[tokio::test]
    async fn test_memory_cache_ttl() {
        let cache = MemoryCache::new(Duration::from_millis(100));

        // 設置快取項目
        cache.set("test_key".to_string(), 42).await;
        assert_eq!(cache.get("test_key").await, Some(42));

        // 等待 TTL 過期
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("test_key").await, None);
    }

    #[tokio::test]
    async fn test_balance_cache() {
        let balance_cache = BalanceCache::new();
        let user_id = 12345_u64;
        let balance = BigDecimal::from_str("1000.50").unwrap();

        // 測試設置和獲取餘額
        balance_cache.set_balance(user_id, balance.clone()).await;
        assert_eq!(balance_cache.get_balance(user_id).await, Some(balance));

        // 測試不存在的用戶
        assert_eq!(balance_cache.get_balance(99999).await, None);
    }

    #[tokio::test]
    async fn test_balance_cache_key_generation() {
        assert_eq!(BalanceCache::balance_key(12345), "balance:12345");
        assert_eq!(BalanceCache::balance_key(0), "balance:0");
    }

    #[tokio::test]
    async fn test_redis_cache_stats_parsing() {
        let info = r#"used_memory:1048576
used_memory_human:1M
used_memory_peak:2097152
used_memory_peak_human:2M
connected_clients:5
total_commands_processed:1000
keyspace_hits:800
keyspace_misses:200"#;

        let stats = RedisCacheStats::from_redis_info(info);

        assert_eq!(stats.used_memory, "1048576");
        assert_eq!(stats.used_memory_human, "1M");
        assert_eq!(stats.connected_clients, 5);
        assert_eq!(stats.keyspace_hits, 800);
        assert_eq!(stats.keyspace_misses, 200);
        assert!((stats.hit_rate - 0.8).abs() < 0.001); // 800/(800+200) = 0.8
    }

    #[tokio::test]
    async fn test_balance_cache_memory_backend() {
        let cache = BalanceCache::new();
        assert!(matches!(cache.cache_type, CacheType::Memory));
        assert!(cache.redis_cache.is_none());
    }

    #[tokio::test]
    async fn test_balance_cache_health_check_memory() {
        let cache = BalanceCache::new();
        let health = cache.health_check().await.unwrap();
        assert!(health); // Memory cache should always be healthy
    }
}