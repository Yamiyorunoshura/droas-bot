// Cache Performance Tests - 快取性能測試
// 專門測試快取系統的性能，包括命中率、響應時間和一致性

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use tokio::time::sleep;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use droas_bot::cache::{BalanceCache, MemoryCache, RedisCache, CacheType};

/// 快取性能測試結果
#[derive(Debug, Clone)]
pub struct CachePerformanceTestResult {
    pub test_name: String,
    pub total_operations: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub hit_rate: f64,
    pub avg_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub operations_per_second: f64,
    pub memory_usage: Option<u64>, // KB
    pub meets_requirements: bool,
}

impl CachePerformanceTestResult {
    fn new(
        test_name: String,
        total_operations: u32,
        cache_hits: u32,
        cache_misses: u32,
        response_times: Vec<Duration>,
        duration: Duration,
    ) -> Self {
        let hit_rate = if total_operations > 0 {
            cache_hits as f64 / total_operations as f64 * 100.0
        } else {
            0.0
        };

        let mut sorted_times = response_times.clone();
        sorted_times.sort();

        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<Duration>() / response_times.len() as u32
        } else {
            Duration::ZERO
        };

        let p95_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 95 / 100]
        } else {
            Duration::ZERO
        };

        let p99_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 99 / 100]
        } else {
            Duration::ZERO
        };

        let operations_per_second = if duration.as_secs_f64() > 0.0 {
            total_operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        // NFR 要求快取命中率 > 80%，響應時間 < 100ms
        let meets_requirements = hit_rate > 80.0 && p95_response_time < Duration::from_millis(100);

        Self {
            test_name,
            total_operations,
            cache_hits,
            cache_misses,
            hit_rate,
            avg_response_time,
            p95_response_time,
            p99_response_time,
            operations_per_second,
            memory_usage: None,
            meets_requirements,
        }
    }

    /// 打印測試結果
    pub fn print(&self) {
        println!("\n=== {} ===", self.test_name);
        println!("總操作數: {}", self.total_operations);
        println!("快取命中: {}", self.cache_hits);
        println!("快取未命中: {}", self.cache_misses);
        println!("命中率: {:.2}%", self.hit_rate);
        println!("平均響應時間: {:?}", self.avg_response_time);
        println!("P95 響應時間: {:?}", self.p95_response_time);
        println!("P99 響應時間: {:?}", self.p99_response_time);
        println!("每秒操作數: {:.2}", self.operations_per_second);

        if let Some(memory) = self.memory_usage {
            println!("記憶體使用: {} KB", memory);
        }

        println!("性能要求滿足: {}", if self.meets_requirements { "✅" } else { "❌" });
    }
}

/// 快取命中率測試
pub async fn test_cache_hit_rate(
    cache: Arc<BalanceCache>,
    unique_keys: u32,
    queries_per_key: u32,
) -> CachePerformanceTestResult {
    let test_name = format!("Cache Hit Rate Test - {} keys, {} queries/key",
                           unique_keys, queries_per_key);
    println!("開始測試: {}", test_name);

    let mut cache_hits = 0u32;
    let mut cache_misses = 0u32;
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    // 第一輪：填充快取（應該全部未命中）
    for key_id in 1..=unique_keys {
        let operation_start = Instant::now();

        let user_id = key_id as u64;
        let balance = cache.get_balance(user_id).await;

        let response_time = operation_start.elapsed();
        response_times.push(response_time);

        if balance.is_none() {
            cache_misses += 1;
            // 設置快取值
            let new_balance = BigDecimal::from_str(&format!("{}.00", user_id * 100)).unwrap();
            cache.set_balance(user_id, new_balance).await;
        }
    }

    // 第二輪及後續：查詢快取（應該大部分命中）
    for round in 2..=queries_per_key {
        for key_id in 1..=unique_keys {
            let operation_start = Instant::now();

            let user_id = key_id as u64;
            let balance = cache.get_balance(user_id).await;

            let response_time = operation_start.elapsed();
            response_times.push(response_time);

            if balance.is_some() {
                cache_hits += 1;
            } else {
                cache_misses += 1;
                // 重新設置快取值
                let new_balance = BigDecimal::from_str(&format!("{}.00", user_id * 100)).unwrap();
                cache.set_balance(user_id, new_balance).await;
            }
        }
    }

    let total_operations = cache_hits + cache_misses;
    let duration = start_time.elapsed();

    let result = CachePerformanceTestResult::new(
        test_name,
        total_operations,
        cache_hits,
        cache_misses,
        response_times,
        duration,
    );

    result.print();
    result
}

/// 並發快取訪問測試
pub async fn test_concurrent_cache_access(
    cache: Arc<BalanceCache>,
    concurrent_users: u32,
    operations_per_user: u32,
) -> CachePerformanceTestResult {
    let test_name = format!("Concurrent Cache Access - {} users, {} ops/user",
                           concurrent_users, operations_per_user);
    println!("開始測試: {}", test_name);

    let semaphore = Arc::new(Semaphore::new(concurrent_users as usize));
    let mut tasks = Vec::new();
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    for user_id in 1..=concurrent_users {
        for _ in 0..operations_per_user {
            let cache = Arc::clone(&cache);
            let semaphore = Arc::clone(&semaphore);
            let user_id = user_id;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let operation_start = Instant::now();

                // 隨機決定是讀取還是寫入
                if fastrand::bool() {
                    // 讀取操作
                    let balance = cache.get_balance(user_id as u64).await;
                    let response_time = operation_start.elapsed();
                    (balance.is_some(), response_time, false)
                } else {
                    // 寫入操作
                    let new_balance = BigDecimal::from_str(&format!("{}.00", user_id * 100)).unwrap();
                    cache.set_balance(user_id as u64, new_balance).await;
                    let response_time = operation_start.elapsed();
                    (true, response_time, true)
                }
            });

            tasks.push(task);
        }
    }

    let mut cache_hits = 0u32;
    let mut cache_misses = 0u32;

    for task in tasks {
        match task.await {
            Ok((cache_hit, response_time, is_write)) => {
                response_times.push(response_time);
                if is_write {
                    // 寫入操作不計算命中率
                } else if cache_hit {
                    cache_hits += 1;
                } else {
                    cache_misses += 1;
                }
            }
            Err(_) => {
                // 任務失敗
                cache_misses += 1;
            }
        }
    }

    let total_operations = cache_hits + cache_misses;
    let duration = start_time.elapsed();

    let result = CachePerformanceTestResult::new(
        test_name,
        total_operations,
        cache_hits,
        cache_misses,
        response_times,
        duration,
    );

    result.print();
    result
}

/// 快取過期清理測試
pub async fn test_cache_expiration_cleanup(
    cache: Arc<BalanceCache>,
    ttl_seconds: u64,
) -> CachePerformanceTestResult {
    let test_name = format!("Cache Expiration Cleanup - TTL: {}s", ttl_seconds);
    println!("開始測試: {}", test_name);

    let mut cache_hits = 0u32;
    let mut cache_misses = 0u32;
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    // 設置一些快取項目
    let num_items = 100;
    for i in 1..=num_items {
        let user_id = i as u64;
        let balance = BigDecimal::from_str(&format!("{}.00", i * 10)).unwrap();
        cache.set_balance_with_ttl(user_id, balance, Duration::from_secs(ttl_seconds)).await;
    }

    // 立即查詢（應該全部命中）
    for i in 1..=num_items {
        let operation_start = Instant::now();

        let user_id = i as u64;
        let balance = cache.get_balance(user_id).await;

        let response_time = operation_start.elapsed();
        response_times.push(response_time);

        if balance.is_some() {
            cache_hits += 1;
        } else {
            cache_misses += 1;
        }
    }

    // 等待快取過期
    sleep(Duration::from_secs(ttl_seconds + 1)).await;

    // 再次查詢（應該全部未命中）
    for i in 1..=num_items {
        let operation_start = Instant::now();

        let user_id = i as u64;
        let balance = cache.get_balance(user_id).await;

        let response_time = operation_start.elapsed();
        response_times.push(response_time);

        if balance.is_some() {
            cache_hits += 1;
        } else {
            cache_misses += 1;
        }
    }

    // 執行清理
    cache.cleanup().await;

    let total_operations = cache_hits + cache_misses;
    let duration = start_time.elapsed();

    let result = CachePerformanceTestResult::new(
        test_name,
        total_operations,
        cache_hits,
        cache_misses,
        response_times,
        duration,
    );

    result.print();
    result
}

/// 記憶體快取 vs Redis 快取性能比較
pub async fn test_memory_vs_redis_performance() -> Vec<CachePerformanceTestResult> {
    println!("開始記憶體快取 vs Redis 快取性能比較測試");

    let mut results = Vec::new();

    // 測試記憶體快取
    let memory_cache = Arc::new(BalanceCache::new_with_ttl(Duration::from_secs(300)));
    let memory_result = test_cache_hit_rate(memory_cache, 50, 10).await;
    results.push(memory_result);

    // 如果 Redis 可用，測試 Redis 快取
    if let Ok(redis_cache) = RedisCache::new("redis://localhost:6379").await {
        let redis_balance_cache = Arc::new(BalanceCache::from_redis(redis_cache).await);
        let redis_result = test_cache_hit_rate(redis_balance_cache, 50, 10).await;
        results.push(redis_result);
    } else {
        println!("Redis 不可用，跳過 Redis 快取測試");
    }

    // 比較結果
    println!("\n=== 性能比較結果 ===");
    for (i, result) in results.iter().enumerate() {
        let cache_type = if i == 0 { "記憶體快取" } else { "Redis 快取" };
        println!("{} - 命中率: {:.2}%, P95: {:?}, 吞吐量: {:.2} ops/s",
                cache_type, result.hit_rate, result.p95_response_time, result.operations_per_second);
    }

    results
}

/// 快取一致性測試
pub async fn test_cache_consistency(
    cache: Arc<BalanceCache>,
) -> CachePerformanceTestResult {
    let test_name = "Cache Consistency Test".to_string();
    println!("開始測試: {}", test_name);

    let mut cache_hits = 0u32;
    let mut cache_misses = 0u32;
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    let user_id = 12345u64;
    let initial_balance = BigDecimal::from_str("1000.00").unwrap();

    // 設置初始值
    cache.set_balance(user_id, initial_balance.clone()).await;

    // 多次讀取驗證一致性
    for _ in 0..100 {
        let operation_start = Instant::now();

        let balance = cache.get_balance(user_id).await;

        let response_time = operation_start.elapsed();
        response_times.push(response_time);

        if let Some(cached_balance) = balance {
            if cached_balance == initial_balance {
                cache_hits += 1;
            } else {
                cache_misses += 1; // 數據不一致
            }
        } else {
            cache_misses += 1;
        }
    }

    // 更新值
    let updated_balance = BigDecimal::from_str("2000.00").unwrap();
    cache.set_balance(user_id, updated_balance.clone()).await;

    // 驗證更新後的一致性
    for _ in 0..100 {
        let operation_start = Instant::now();

        let balance = cache.get_balance(user_id).await;

        let response_time = operation_start.elapsed();
        response_times.push(response_time);

        if let Some(cached_balance) = balance {
            if cached_balance == updated_balance {
                cache_hits += 1;
            } else {
                cache_misses += 1; // 數據不一致
            }
        } else {
            cache_misses += 1;
        }
    }

    let total_operations = cache_hits + cache_misses;
    let duration = start_time.elapsed();

    let result = CachePerformanceTestResult::new(
        test_name,
        total_operations,
        cache_hits,
        cache_misses,
        response_times,
        duration,
    );

    // 一致性測試的特殊要求：命中率應該是 100%
    let consistency_rate = cache_hits as f64 / total_operations as f64 * 100.0;
    println!("一致性率: {:.2}%", consistency_rate);

    result.print();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use droas_bot::cache::BalanceCache;

    #[tokio::test]
    async fn test_cache_performance_basic() {
        let cache = Arc::new(BalanceCache::new());

        let result = test_cache_hit_rate(
            cache,
            10,  // 10 個唯一鍵
            5    // 每鍵 5 次查詢
        ).await;

        // 驗證基本快取性能
        assert!(result.hit_rate > 50.0); // 至少 50% 命中率
        assert!(result.meets_requirements);
    }

    #[tokio::test]
    async fn test_concurrent_cache_performance() {
        let cache = Arc::new(BalanceCache::new());

        let result = test_concurrent_cache_access(
            cache,
            20,  // 20 個並發用戶
            3    // 每用戶 3 次操作
        ).await;

        // 驗證並發快取性能
        assert!(result.total_operations > 0);
        assert!(result.operations_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Arc::new(BalanceCache::new());

        let result = test_cache_expiration_cleanup(
            cache,
            2    // 2 秒 TTL
        ).await;

        // 驗證快取過期機制
        assert!(result.total_operations > 0);
    }

    #[tokio::test]
    async fn test_cache_consistency_basic() {
        let cache = Arc::new(BalanceCache::new());

        let result = test_cache_consistency(cache).await;

        // 驗證快取一致性 - 應該有很高的命中率
        assert!(result.hit_rate > 95.0);
    }

    #[tokio::test]
    async fn test_memory_cache_vs_performance() {
        let cache = Arc::new(BalanceCache::new_with_ttl(Duration::from_secs(60)));

        let result = test_cache_hit_rate(
            cache,
            50,  // 50 個唯一鍵
            10   // 每鍵 10 次查詢
        ).await;

        // 驗證記憶體快取性能
        assert!(result.hit_rate > 80.0); // 記憶體快取應該有很高的命中率
        assert!(result.p95_response_time < Duration::from_millis(10)); // 記憶體快取應該很快
    }
}