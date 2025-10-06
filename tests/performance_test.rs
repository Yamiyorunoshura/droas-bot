// Performance Tests - 性能測試
// 測試系統在高負載下的性能表現，確保滿足 NFR 要求

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use droas_bot::cache::BalanceCache;

// 模擬服務層用於性能測試
struct MockBalanceService {
    cache: Arc<BalanceCache>,
    // 模擬資料庫延遲
    db_delay: Duration,
}

impl MockBalanceService {
    fn new(cache: Arc<BalanceCache>, db_delay: Duration) -> Self {
        Self { cache, db_delay }
    }

    async fn get_balance(&self, user_id: u64) -> Option<BigDecimal> {
        // 先檢查快取
        if let Some(balance) = self.cache.get_balance(user_id).await {
            return Some(balance);
        }

        // 模擬資料庫查詢延遲
        sleep(self.db_delay).await;

        // 模擬從資料庫獲取餘額
        let balance = BigDecimal::from_str("1000.00").unwrap();
        self.cache.set_balance(user_id, balance.clone()).await;
        Some(balance)
    }

    async fn set_balance(&self, user_id: u64, balance: BigDecimal) {
        // 模擬資料庫更新延遲
        sleep(self.db_delay).await;

        // 更新快取
        self.cache.set_balance(user_id, balance).await;
    }
}

/// 性能測試結果
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub duration: Duration,
    pub avg_response_time: Duration,
    pub p50_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub operations_per_second: f64,
}

impl PerformanceTestResult {
    fn new(
        test_name: String,
        total_operations: u32,
        successful_operations: u32,
        failed_operations: u32,
        duration: Duration,
        response_times: Vec<Duration>,
    ) -> Self {
        let mut sorted_times = response_times.clone();
        sorted_times.sort();

        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<Duration>() / response_times.len() as u32
        } else {
            Duration::ZERO
        };

        let p50_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 50 / 100]
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
            successful_operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            test_name,
            total_operations,
            successful_operations,
            failed_operations,
            duration,
            avg_response_time,
            p50_response_time,
            p95_response_time,
            p99_response_time,
            operations_per_second,
        }
    }

    /// 驗證性能是否滿足 NFR 要求
    pub fn meets_nfr_requirements(&self) -> bool {
        // NFR-P-001: 95% 的命令在 2 秒內響應
        let meets_response_time = self.p95_response_time < Duration::from_secs(2);

        // NFR-P-002: 餘額查詢在 500ms 內完成（對於餘額查詢測試）
        let meets_balance_query_time = self.test_name.contains("balance")
            && self.p95_response_time < Duration::from_millis(500);

        // 成功率應該 > 99%
        let success_rate = self.successful_operations as f64 / self.total_operations as f64;
        let meets_success_rate = success_rate >= 0.99;

        match self.test_name.as_str() {
            name if name.contains("balance") => meets_response_time && meets_balance_query_time && meets_success_rate,
            _ => meets_response_time && meets_success_rate,
        }
    }

    /// 打印測試結果
    pub fn print(&self) {
        println!("\n=== {} ===", self.test_name);
        println!("總操作數: {}", self.total_operations);
        println!("成功操作數: {}", self.successful_operations);
        println!("失敗操作數: {}", self.failed_operations);

        let success_rate = self.successful_operations as f64 / self.total_operations as f64 * 100.0;
        println!("成功率: {:.2}%", success_rate);

        println!("執行時間: {:?}", self.duration);
        println!("平均響應時間: {:?}", self.avg_response_time);
        println!("P50 響應時間: {:?}", self.p50_response_time);
        println!("P95 響應時間: {:?}", self.p95_response_time);
        println!("P99 響應時間: {:?}", self.p99_response_time);
        println!("每秒操作數: {:.2}", self.operations_per_second);

        let meets_requirements = self.meets_nfr_requirements();
        println!("NFR 要求滿足: {}", if meets_requirements { "✓" } else { "✗" });
    }
}

/// 餘額查詢性能測試
pub async fn test_balance_query_performance(
    service: Arc<MockBalanceService>,
    concurrent_users: u32,
    operations_per_user: u32,
) -> PerformanceTestResult {
    let test_name = format!("Balance Query Performance - {} users, {} ops/user",
                           concurrent_users, operations_per_user);
    println!("開始測試: {}", test_name);

    let semaphore = Arc::new(Semaphore::new(concurrent_users as usize));
    let mut tasks = Vec::new();
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    for user_id in 1..=concurrent_users {
        for _ in 0..operations_per_user {
            let service = Arc::clone(&service);
            let semaphore = Arc::clone(&semaphore);
            let user_id = user_id;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let operation_start = Instant::now();

                let result = service.get_balance(user_id as u64).await;

                let response_time = operation_start.elapsed();
                (result.is_some(), response_time)
            });

            tasks.push(task);
        }
    }

    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for task in tasks {
        match task.await {
            Ok((success, response_time)) => {
                response_times.push(response_time);
                if success {
                    successful_operations += 1;
                } else {
                    failed_operations += 1;
                }
            }
            Err(_) => {
                failed_operations += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let total_operations = successful_operations + failed_operations;

    let result = PerformanceTestResult::new(
        test_name,
        total_operations,
        successful_operations,
        failed_operations,
        duration,
        response_times,
    );

    result.print();
    result
}

/// 快取命中率測試
pub async fn test_cache_hit_rate(
    service: Arc<MockBalanceService>,
    unique_users: u32,
    queries_per_user: u32,
) -> PerformanceTestResult {
    let test_name = format!("Cache Hit Rate Test - {} users, {} queries/user",
                           unique_users, queries_per_user);
    println!("開始測試: {}", test_name);

    let semaphore = Arc::new(Semaphore::new(unique_users as usize));
    let mut tasks = Vec::new();
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    for user_id in 1..=unique_users {
        for query_num in 0..queries_per_user {
            let service = Arc::clone(&service);
            let semaphore = Arc::clone(&semaphore);
            let user_id = user_id;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let operation_start = Instant::now();

                let result = service.get_balance(user_id as u64).await;

                let response_time = operation_start.elapsed();
                (result.is_some(), response_time)
            });

            tasks.push(task);
        }
    }

    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for task in tasks {
        match task.await {
            Ok((success, response_time)) => {
                response_times.push(response_time);
                if success {
                    successful_operations += 1;
                } else {
                    failed_operations += 1;
                }
            }
            Err(_) => {
                failed_operations += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let total_operations = successful_operations + failed_operations;

    // 獲取快取統計
    let cache_stats = service.cache.stats().await;
    if let Some(redis_stats) = cache_stats.redis_stats {
        let hit_rate = redis_stats.hit_rate * 100.0;
        println!("快取命中率: {:.2}%", hit_rate);

        // NFR 要求快取命中率 > 80%
        if hit_rate > 80.0 {
            println!("✓ 快取命中率滿足要求 (>80%)");
        } else {
            println!("✗ 快取命中率不滿足要求 (<80%)");
        }
    }

    let result = PerformanceTestResult::new(
        test_name,
        total_operations,
        successful_operations,
        failed_operations,
        duration,
        response_times,
    );

    result.print();
    result
}

/// 並發餘額更新測試
pub async fn test_concurrent_balance_updates(
    service: Arc<MockBalanceService>,
    concurrent_users: u32,
) -> PerformanceTestResult {
    let test_name = format!("Concurrent Balance Updates - {} users", concurrent_users);
    println!("開始測試: {}", test_name);

    let semaphore = Arc::new(Semaphore::new(concurrent_users as usize));
    let mut tasks = Vec::new();
    let mut response_times = Vec::new();
    let start_time = Instant::now();

    for user_id in 1..=concurrent_users {
        let service = Arc::clone(&service);
        let semaphore = Arc::clone(&semaphore);

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let operation_start = Instant::now();

            // 執行餘額更新
            let new_balance = BigDecimal::from_str(&format!("{}.00", user_id * 100)).unwrap();
            service.set_balance(user_id as u64, new_balance).await;

            let response_time = operation_start.elapsed();
            (true, response_time) // 更新操作總是成功
        });

        tasks.push(task);
    }

    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for task in tasks {
        match task.await {
            Ok((success, response_time)) => {
                response_times.push(response_time);
                if success {
                    successful_operations += 1;
                } else {
                    failed_operations += 1;
                }
            }
            Err(_) => {
                failed_operations += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let total_operations = successful_operations + failed_operations;

    let result = PerformanceTestResult::new(
        test_name,
        total_operations,
        successful_operations,
        failed_operations,
        duration,
        response_times,
    );

    result.print();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use droas_bot::cache::BalanceCache;

    #[tokio::test]
    async fn test_performance_balance_query_basic() {
        let cache = Arc::new(BalanceCache::new());
        let service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(10)));

        let result = test_balance_query_performance(
            service,
            10,  // 10 個並發用戶
            5    // 每用戶 5 次操作
        ).await;

        // 驗證基本性能要求
        assert!(result.meets_nfr_requirements());
        assert_eq!(result.failed_operations, 0);
    }

    #[tokio::test]
    async fn test_performance_cache_hit_rate() {
        let cache = Arc::new(BalanceCache::new());
        let service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(10)));

        let result = test_cache_hit_rate(
            service,
            5,   // 5 個唯一用戶
            10   // 每用戶 10 次查詢
        ).await;

        // 驗證快取效果
        assert!(result.meets_nfr_requirements());
        assert!(result.p95_response_time < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_performance_concurrent_updates() {
        let cache = Arc::new(BalanceCache::new());
        let service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(5)));

        let result = test_concurrent_balance_updates(
            service,
            20   // 20 個並發更新
        ).await;

        // 驗證並發更新性能
        assert_eq!(result.failed_operations, 0);
        assert!(result.avg_response_time < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_performance_scalability() {
        let cache = Arc::new(BalanceCache::new());
        let service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(1)));

        // 測試不同負載級別
        for users in [10, 50, 100] {
            let result = test_balance_query_performance(
                Arc::clone(&service),
                users,
                2
            ).await;

            // 即使在高負載下也應該滿足基本要求
            assert!(result.successful_operations > 0);
            assert!(result.operations_per_second > 0.0);
        }
    }
}