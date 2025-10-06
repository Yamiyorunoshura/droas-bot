// Cache Integration Tests - 快取整合測試
// 測試快取層的完整功能

use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;
use droas_bot::cache::{RedisCache, BalanceCache};
use droas_bot::services::balance_service::BalanceService;
use droas_bot::database::BalanceRepository;
use droas_bot::config::CacheConfig;

/// 測試快取命中場景
/// 場景：重複查詢相同用戶餘額
/// 預期結果：第二次查詢響應時間 < 100ms
#[tokio::test]
async fn test_cache_hit_performance() {
    // 設置測試環境
    let user_id = 12345_u64;
    let expected_balance = BigDecimal::from_str("1000.50").unwrap();

    // 創建記憶體快取（測試環境）
    let balance_cache = BalanceCache::new_with_ttl(Duration::from_secs(60));
    let mock_repo = create_mock_balance_repository().await;
    let balance_service = BalanceService::new_with_cache(mock_repo, balance_cache);

    // 設置初始餘額到快取
    balance_service.set_balance(user_id, expected_balance.clone()).await.unwrap();

    // 第一次查詢（應該從快取載入）
    let start_time = std::time::Instant::now();
    let result1 = balance_service.get_cached_balance(user_id).await;
    let _first_query_duration = start_time.elapsed();

    // 第二次查詢（應該從快取載入）
    let start_time = std::time::Instant::now();
    let result2 = balance_service.get_cached_balance(user_id).await;
    let second_query_duration = start_time.elapsed();

    // 驗證結果
    assert_eq!(result1, Some(expected_balance.clone()));
    assert_eq!(result2, Some(expected_balance));

    // 驗證：第二次查詢應該很快（< 100ms）
    assert!(second_query_duration < Duration::from_millis(100),
            "Cache hit should be faster than 100ms, got {:?}", second_query_duration);
}

/// 測試快取未命中場景
/// 場景：查詢不存在於快取的用戶資料
/// 預期結果：正確返回 None（因為沒有資料庫載入）
#[tokio::test]
async fn test_cache_miss_returns_none() {
    let user_id = 67890_u64;

    // 創建記憶體快取
    let balance_cache = BalanceCache::new();
    let mock_repo = create_mock_balance_repository().await;
    let balance_service = BalanceService::new_with_cache(mock_repo, balance_cache);

    // 查詢用戶餘額（快取未命中，應該返回 None）
    let result = balance_service.get_cached_balance(user_id).await;

    // 驗證結果為 None
    assert_eq!(result, None);
}

/// 測試快取一致性場景
/// 場景：更新用戶餘額後立即查詢
/// 預期結果：查詢結果為更新後的餘額
#[tokio::test]
async fn test_cache_consistency_after_update() {
    let user_id = 11111_u64;
    let initial_balance = BigDecimal::from_str("1000.00").unwrap();
    let updated_balance = BigDecimal::from_str("1500.00").unwrap();

    // 創建記憶體快取和服務
    let balance_cache = BalanceCache::new();
    let mock_repo = create_mock_balance_repository().await;
    let balance_service = BalanceService::new_with_cache(mock_repo, balance_cache);

    // 設置初始餘額並快取
    balance_service.set_balance(user_id, initial_balance.clone()).await.unwrap();
    let initial_result = balance_service.get_cached_balance(user_id).await;
    assert_eq!(initial_result, Some(initial_balance));

    // 更新餘額
    balance_service.update_balance(user_id, updated_balance.clone()).await.unwrap();

    // 立即查詢餘額，應該得到更新後的值
    let updated_result = balance_service.get_cached_balance(user_id).await;
    assert_eq!(updated_result, Some(updated_balance));
}

/// 測試快取失效場景
/// 場景：快取過期時間到達後查詢
/// 預期結果：快取項目已失效，返回 None
#[tokio::test]
async fn test_cache_expiration() {
    let user_id = 22222_u64;
    let initial_balance = BigDecimal::from_str("750.00").unwrap();

    // 創建短期快取（1秒過期）
    let balance_cache = BalanceCache::new_with_ttl(Duration::from_secs(1));
    let mock_repo = create_mock_balance_repository().await;
    let balance_service = BalanceService::new_with_cache(mock_repo, balance_cache);

    // 設置初始餘額
    balance_service.set_balance(user_id, initial_balance.clone()).await.unwrap();

    // 查詢確認快取有效
    let result1 = balance_service.get_cached_balance(user_id).await;
    assert_eq!(result1, Some(initial_balance));

    // 等待快取過期
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 再次查詢，應該得到 None（快取過期）
    let result2 = balance_service.get_cached_balance(user_id).await;
    assert_eq!(result2, None);
}

/// 測試並發安全場景
/// 場景：100 個並發請求同時訪問同一用戶資料
/// 預期結果：無資料競爭，結果一致性保證
#[tokio::test]
async fn test_cache_concurrent_safety() {
    let user_id = 33333_u64;
    let expected_balance = BigDecimal::from_str("2000.00").unwrap();
    let concurrent_requests = 100;

    // 創建記憶體快取和服務
    let balance_cache = BalanceCache::new();
    let mock_repo = create_mock_balance_repository().await;
    let balance_service = std::sync::Arc::new(BalanceService::new_with_cache(mock_repo, balance_cache));

    // 設置初始餘額
    balance_service.set_balance(user_id, expected_balance.clone()).await.unwrap();

    // 創建並發任務
    let mut handles = Vec::new();

    for _i in 0..concurrent_requests {
        let service = balance_service.clone();
        let handle = tokio::spawn(async move {
            service.get_cached_balance(user_id).await
        });
        handles.push(handle);
    }

    // 等待所有請求完成
    let mut results = Vec::new();
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await.unwrap().unwrap();
        results.push(result);
    }

    // 驗證所有結果都一致
    for result in results {
        assert_eq!(result, Some(expected_balance.clone()));
    }

    // 驗證快取統計
    let cache_stats = balance_service.get_cache_stats().await;
    assert!(cache_stats.total_items >= 1, "Should have at least one cache item");
}

/// 測試 Redis 快取連接和健康檢查
#[tokio::test]
async fn test_redis_connection_health() {
    // 嘗試創建 Redis 快取（如果 Redis 可用）
    let redis_result = RedisCache::new("redis://localhost:6379").await;

    match redis_result {
        Ok(redis_cache) => {
            // 測試健康檢查
            let health = redis_cache.ping().await.unwrap();
            assert!(health, "Redis should be healthy");

            // 測試基本操作
            let test_key = "test_health_key";
            let test_value = "test_value";

            redis_cache.set(test_key, test_value).await.unwrap();
            let retrieved = redis_cache.get(test_key).await.unwrap();
            assert_eq!(retrieved, Some(test_value.to_string()));

            redis_cache.remove(test_key).await.unwrap();
        }
        Err(_) => {
            // Redis 不可用，跳過測試
            println!("Redis not available, skipping Redis connection test");
        }
    }
}

/// 測試快取配置驗證
#[tokio::test]
async fn test_cache_config_validation() {
    // 測試預設配置
    let default_config = CacheConfig::default();
    assert!(default_config.validate().is_ok());
    assert_eq!(default_config.namespace, "droas");

    // 測試記憶體快取配置
    let memory_config = CacheConfig::memory_only();
    assert!(memory_config.validate().is_ok());
    assert!(!memory_config.enable_redis);
    assert!(memory_config.fallback_to_memory);

    // 測試測試配置
    let test_config = CacheConfig::for_test();
    assert!(test_config.validate().is_ok());
    assert!(!test_config.enable_redis);
    assert_eq!(test_config.namespace, "droas_test");

    // 測試命名空間鍵生成
    let namespaced_key = default_config.namespaced_key("test_key");
    assert_eq!(namespaced_key, "droas:test_key");
}

/// 測試快取統計信息
#[tokio::test]
async fn test_cache_statistics() {
    let balance_cache = BalanceCache::new();
    let user_id = 44444_u64;
    let balance = BigDecimal::from_str("500.25").unwrap();

    // 初始統計
    let initial_stats = balance_cache.stats().await;
    assert_eq!(initial_stats.total_items, 0);
    assert_eq!(initial_stats.active_items, 0);

    // 添加快取項目
    balance_cache.set_balance(user_id, balance.clone()).await;

    // 更新後的統計
    let updated_stats = balance_cache.stats().await;
    assert_eq!(updated_stats.total_items, 1);
    assert_eq!(updated_stats.active_items, 1);
    assert_eq!(updated_stats.expired_items, 0);

    // 驗證快取命中
    let cached_balance = balance_cache.get_balance(user_id).await;
    assert_eq!(cached_balance, Some(balance));
}

// === 輔助函數 ===

/// 創建模擬的餘額儲存庫（不依賴真實資料庫）
async fn create_mock_balance_repository() -> BalanceRepository {
    // 注意：這是一個用於測試的假實現
    // 為了測試快取功能，我們創建一個虛假的儲存庫實例

    // 由於我們無法在測試中輕鬆創建模擬的資料庫連接，
    // 我們使用一個簡化的方法來創建一個結構上正確的實例
    // 這僅用於測試快取層，不會實際執行資料庫操作

    // 注意：在沒有真實資料庫連接的情況下，這個函數會失敗
    // 在 CI/CD 環境中應該設置測試資料庫
    panic!("需要資料庫連接來執行快取整合測試。請設置測試資料庫或使用 mock 依賴。");
}