// Basic Cache Tests - 基本快取測試
// 測試快取層的基本功能，不依賴資料庫

use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::time::Duration;
use droas_bot::cache::BalanceCache;
use droas_bot::config::CacheConfig;

/// 測試快取基本功能
#[tokio::test]
async fn test_balance_cache_basic_operations() {
    let balance_cache = BalanceCache::new();
    let user_id = 12345_u64;
    let balance = BigDecimal::from_str("1000.50").unwrap();

    // 測試設置和獲取餘額
    balance_cache.set_balance(user_id, balance.clone()).await;
    let retrieved_balance = balance_cache.get_balance(user_id).await;
    assert_eq!(retrieved_balance, Some(balance));

    // 測試不存在的用戶
    assert_eq!(balance_cache.get_balance(99999).await, None);
}

/// 測試快取過期功能
#[tokio::test]
async fn test_balance_cache_expiration() {
    let balance_cache = BalanceCache::new_with_ttl(Duration::from_millis(100));
    let user_id = 12345_u64;
    let balance = BigDecimal::from_str("1000.50").unwrap();

    // 設置快取項目
    balance_cache.set_balance(user_id, balance.clone()).await;
    assert_eq!(balance_cache.get_balance(user_id).await, Some(balance));

    // 等待 TTL 過期
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(balance_cache.get_balance(user_id).await, None);
}

/// 測試快取鍵生成
#[tokio::test]
async fn test_balance_cache_key_generation() {
    assert_eq!(BalanceCache::balance_key(12345), "balance:12345");
    assert_eq!(BalanceCache::balance_key(0), "balance:0");
    assert_eq!(BalanceCache::balance_key(999999999), "balance:999999999");
}

/// 測試快取統計
#[tokio::test]
async fn test_balance_cache_statistics() {
    let balance_cache = BalanceCache::new();

    // 初始統計
    let initial_stats = balance_cache.stats().await;
    assert_eq!(initial_stats.total_items, 0);
    assert_eq!(initial_stats.active_items, 0);
    assert_eq!(initial_stats.expired_items, 0);

    // 添加一些快取項目
    balance_cache.set_balance(1, BigDecimal::from_str("100").unwrap()).await;
    balance_cache.set_balance(2, BigDecimal::from_str("200").unwrap()).await;
    balance_cache.set_balance(3, BigDecimal::from_str("300").unwrap()).await;

    // 更新後的統計
    let updated_stats = balance_cache.stats().await;
    assert_eq!(updated_stats.total_items, 3);
    assert_eq!(updated_stats.active_items, 3);
    assert_eq!(updated_stats.expired_items, 0);
}

/// 測試快取刪除功能
#[tokio::test]
async fn test_balance_cache_removal() {
    let balance_cache = BalanceCache::new();
    let user_id = 12345_u64;
    let balance = BigDecimal::from_str("1000.50").unwrap();

    // 設置快取項目
    balance_cache.set_balance(user_id, balance.clone()).await;
    assert_eq!(balance_cache.get_balance(user_id).await, Some(balance.clone()));

    // 刪除快取項目
    let removed_balance = balance_cache.remove_balance(user_id).await;
    assert_eq!(removed_balance, Some(balance));

    // 確認項目已被刪除
    assert_eq!(balance_cache.get_balance(user_id).await, None);

    // 刪除不存在的項目
    assert_eq!(balance_cache.remove_balance(99999).await, None);
}

/// 測試快取健康檢查（記憶體快取應該總是健康）
#[tokio::test]
async fn test_balance_cache_health_check() {
    let balance_cache = BalanceCache::new();
    let health = balance_cache.health_check().await.unwrap();
    assert!(health, "Memory cache should always be healthy");
}

/// 測試快取配置
#[tokio::test]
async fn test_cache_config() {
    // 測試預設配置
    let default_config = CacheConfig::default();
    assert!(default_config.validate().is_ok());
    assert_eq!(default_config.namespace, "droas");
    assert!(default_config.enable_redis);
    assert!(default_config.fallback_to_memory);

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

    let namespaced_key_2 = test_config.namespaced_key("user_balance");
    assert_eq!(namespaced_key_2, "droas_test:user_balance");
}

/// 測試快取一致性（更新後立即查詢）
#[tokio::test]
async fn test_balance_cache_consistency() {
    let balance_cache = BalanceCache::new();
    let user_id = 12345_u64;
    let initial_balance = BigDecimal::from_str("1000.00").unwrap();
    let updated_balance = BigDecimal::from_str("1500.00").unwrap();

    // 設置初始餘額
    balance_cache.set_balance(user_id, initial_balance.clone()).await;
    assert_eq!(balance_cache.get_balance(user_id).await, Some(initial_balance));

    // 更新餘額
    balance_cache.set_balance(user_id, updated_balance.clone()).await;

    // 立即查詢，應該得到更新後的值
    assert_eq!(balance_cache.get_balance(user_id).await, Some(updated_balance));
}

/// 測試快取自定義 TTL
#[tokio::test]
async fn test_balance_cache_custom_ttl() {
    let balance_cache = BalanceCache::new();
    let user_id = 12345_u64;
    let balance = BigDecimal::from_str("1000.50").unwrap();
    let short_ttl = Duration::from_millis(50);

    // 設置帶自定義 TTL 的快取項目
    balance_cache.set_balance_with_ttl(user_id, balance.clone(), short_ttl).await;
    assert_eq!(balance_cache.get_balance(user_id).await, Some(balance));

    // 等待 TTL 過期
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(balance_cache.get_balance(user_id).await, None);
}

/// 測試快取清理功能
#[tokio::test]
async fn test_balance_cache_cleanup() {
    let balance_cache = BalanceCache::new_with_ttl(Duration::from_millis(50));

    // 設置一些快取項目
    balance_cache.set_balance(1, BigDecimal::from_str("100").unwrap()).await;
    balance_cache.set_balance(2, BigDecimal::from_str("200").unwrap()).await;

    // 等待快取過期
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 執行清理
    balance_cache.cleanup().await;

    // 檢查統計，應該沒有活動項目
    let stats = balance_cache.stats().await;
    assert_eq!(stats.active_items, 0);
}