//! CORE-001 整合測試
//!
//! 驗證 Discord 客戶端的端到端功能，包括配置加載、
//! 客戶端創建和基本功能驗證。

use droas_bot::config::Config;
use droas_bot::discord::DiscordClient;
use std::sync::Arc;

#[tokio::test]
async fn test_discord_client_integration() {
    // 測試環境設置
    std::env::set_var(
        "DISCORD_BOT_TOKEN",
        "NDkxNjM4NzExODE0MzY4Mjc3.YH5K_w.test_token_do_not_use_in_production",
    );
    std::env::set_var("DISCORD_APPLICATION_ID", "491638711814368277");
    std::env::set_var("SKIP_DISCORD_API_VALIDATION", "1");

    // 加載配置
    let config = Config::load().await.expect("應該能夠加載測試配置");
    let config = Arc::new(config);

    // 創建 Discord 客戶端
    let client = DiscordClient::new(Arc::clone(&config))
        .await
        .expect("應該能夠創建 Discord 客戶端");

    // 驗證客戶端狀態
    let initial_status = client.get_gateway_status().await;
    assert!(matches!(
        initial_status,
        droas_bot::discord::gateway::GatewayStatus::Disconnected
    ));

    // 清理環境變數
    std::env::remove_var("DISCORD_BOT_TOKEN");
    std::env::remove_var("DISCORD_APPLICATION_ID");
    std::env::remove_var("SKIP_DISCORD_API_VALIDATION");
}

#[tokio::test]
async fn test_rate_limiter_integration() {
    use droas_bot::discord::rate_limit::{ExponentialBackoffConfig, RateLimiter};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // 創建測試配置
    let config = ExponentialBackoffConfig {
        base_delay_ms: 10,
        max_delay_ms: 100,
        max_retries: 2,
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let limiter = RateLimiter::with_config(config);

    // 測試成功的重試場景
    let success_count = Arc::new(Mutex::new(0));
    let success_count_clone = Arc::clone(&success_count);

    let result = limiter
        .retry_with_exponential_backoff("integration_test", move || {
            let counter = Arc::clone(&success_count_clone);
            async move {
                let mut count = counter.lock().unwrap();
                *count += 1;
                let current_attempt = *count;
                drop(count);

                if current_attempt == 1 {
                    Err("First attempt fails")
                } else {
                    Ok("Success on retry")
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), "Success on retry");
    assert_eq!(*success_count.lock().unwrap(), 2);

    // 測試速率限制功能
    let wait_time = limiter.wait_if_rate_limited("test_route").await;
    assert_eq!(wait_time, Duration::from_secs(0)); // 沒有限制時應該是0

    // 設置速率限制並測試
    limiter
        .handle_rate_limit_response("test_route", 0.01, false)
        .await; // 10ms 限制
    let wait_time = limiter.wait_if_rate_limited("test_route").await;
    assert!(wait_time.as_millis() > 0); // 應該有等待時間
}
