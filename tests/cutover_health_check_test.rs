// Cutover 健康檢查測試 - RED 階段
// 測試 CUTOVER-005: 健康檢查端點顯示 Discord 連接狀態不一致

use droas_bot::services::monitoring_service::MonitoringService;
use droas_bot::config::DatabaseConfig;

#[tokio::test]
async fn test_health_check_discord_status_consistency() {
    // 測試健康檢查 Discord 連接狀態一致性 - CUTOVER-005 驗證

    let database_config = DatabaseConfig::from_env();
    if database_config.is_err() {
        println!("跳過測試：無法讀取資料庫配置");
        return;
    }

    let pool_result = droas_bot::database::create_user_pool(&database_config.unwrap()).await;
    if pool_result.is_err() {
        println!("跳過測試：無法創建資料庫連接");
        return;
    }

    let pool = pool_result.unwrap();

    // 創建監控服務（不包含 Discord Gateway）
    let monitoring_service = MonitoringService::new(
        droas_bot::health::HealthChecker::new(),
        droas_bot::metrics::MetricsCollector::new(),
        pool,
    );

    // 檢查系統健康狀態
    let health_status = monitoring_service.check_system_health().await;

    println!("健康檢查結果:");
    println!("  Discord 連接狀態: {}", health_status.discord_connected);
    println!("  資料庫連接狀態: {}", health_status.database_connected);
    println!("  快取連接狀態: {}", health_status.cache_connected);
    println!("  最後檢查時間: {:?}", health_status.last_check);

    // 驗證：沒有 Discord Gateway 時，discord_connected 應該為 false
    assert!(!health_status.discord_connected, "沒有 Discord Gateway 時應該顯示未連接");
}

#[tokio::test]
async fn test_health_check_with_discord_gateway() {
    // 測試包含 Discord Gateway 的健康檢查 - CUTOVER-005 診斷

    let database_config = DatabaseConfig::from_env();
    if database_config.is_err() {
        println!("跳過測試：無法讀取資料庫配置");
        return;
    }

    let pool_result = droas_bot::database::create_user_pool(&database_config.unwrap()).await;
    if pool_result.is_err() {
        println!("跳過測試：無法創建資料庫連接");
        return;
    }

    let pool = pool_result.unwrap();

    // 創建 Discord Gateway（未連接狀態）
    let discord_gateway = droas_bot::discord_gateway::DiscordGateway::new();

    // 創建包含 Gateway 的監控服務
    let monitoring_service = MonitoringService::with_gateway(
        pool,
        discord_gateway,
    );

    // 檢查系統健康狀態
    let health_status = monitoring_service.check_system_health().await;

    println!("包含 Gateway 的健康檢查結果:");
    println!("  Discord 連接狀態: {}", health_status.discord_connected);
    println!("  資料庫連接狀態: {}", health_status.database_connected);
    println!("  快取連接狀態: {}", health_status.cache_connected);

    // 注意：無法直接訪問 monitoring_service.discord_gateway（私有字段）
    // 只能檢查健康檢查的結果
    println!("  無法直接檢查 Gateway 狀態（私有字段）");
    println!("  健康檢查的 Discord 狀態: {}", health_status.discord_connected);
}

#[tokio::test]
async fn test_health_check_endpoint_format() {
    // 測試健康檢查端點的響應格式 - CUTOVER-005 補充測試

    let database_config = DatabaseConfig::from_env();
    if database_config.is_err() {
        println!("跳過測試：無法讀取資料庫配置");
        return;
    }

    let pool_result = droas_bot::database::create_user_pool(&database_config.unwrap()).await;
    if pool_result.is_err() {
        println!("跳過測試：無法創建資料庫連接");
        return;
    }

    let pool = pool_result.unwrap();

    // 創建監控服務
    let monitoring_service = MonitoringService::new(
        droas_bot::health::HealthChecker::new(),
        droas_bot::metrics::MetricsCollector::new(),
        pool,
    );

    // 檢查健康狀態
    let health_status = monitoring_service.check_system_health().await;

    // 驗證健康狀態結構包含所有必要字段
    println!("健康狀態結構驗證:");
    println!("  包含 discord_connected: {}", health_status.discord_connected);
    println!("  包含 database_connected: {}", health_status.database_connected);
    println!("  包含 cache_connected: {}", health_status.cache_connected);
    println!("  包含 last_check: {:?}", health_status.last_check);
    println!("  包含 uptime: {:?}", health_status.uptime);

    // 這個測試確保健康狀態結構是完整的
    // 實際的 HTTP 端點測試需要啟動服務器，這裡只測試內部結構
}