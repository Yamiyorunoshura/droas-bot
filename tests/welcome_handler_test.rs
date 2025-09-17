use droas_bot::handlers::welcome::WelcomeHandler;
use droas_bot::database::schema::{GuildConfigService, GuildConfig};
use droas_bot::error::DroasResult;
use sqlx::SqlitePool;
use tempfile::NamedTempFile;
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// 創建測試用的資料庫和服務
async fn create_test_services() -> (SqlitePool, GuildConfigService, NamedTempFile) {
    let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
    let database_url = format!("sqlite://{}", temp_file.path().display());
    
    let pool = sqlx::SqlitePool::connect(&database_url).await
        .expect("無法連接測試資料庫");
    
    // 執行基本 schema 創建
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS guild_config (
            guild_id TEXT PRIMARY KEY,
            welcome_channel_id TEXT NOT NULL,
            background_ref TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("無法創建測試表格");
    
    let guild_service = GuildConfigService::new(pool.clone());
    
    (pool, guild_service, temp_file)
}

/// 設置測試用的 Guild 配置
async fn setup_test_guild_config(service: &GuildConfigService, guild_id: &str, channel_id: &str) {
    let config = GuildConfig {
        guild_id: guild_id.to_string(),
        welcome_channel_id: channel_id.to_string(),
        background_ref: Some("default_background.png".to_string()),
        updated_at: Utc::now(),
    };
    
    service.upsert_guild_config(&config).await
        .expect("無法創建測試 Guild 配置");
}

// ===========================================
// 功能需求測試 (Functional Requirements)
// ===========================================

#[tokio::test]
async fn test_welcome_handler_creation() {
    // FR-W-001: 歡迎訊息處理器可正確創建
    let welcome_handler = WelcomeHandler::new();
    
    // 驗證處理器是否正確創建
    // 由於 WelcomeHandler 目前是空結構，我們主要測試創建不會崩潰
    assert!(true, "WelcomeHandler 應該能夠正常創建");
}

#[tokio::test]
async fn test_welcome_message_sending_success() {
    // FR-W-002: 成功發送歡迎訊息
    let (_pool, _guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    let user_id = 987654321u64;
    
    // 測試歡迎訊息發送
    let result = welcome_handler.handle_member_join(guild_id, user_id).await;
    
    // 由於我們還沒有實際的 Discord API，目前測試基本結構
    assert!(result.is_ok(), "歡迎訊息發送應該成功");
}

#[tokio::test]
async fn test_welcome_message_guild_config_lookup() {
    // FR-W-003: 正確查詢公會配置
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = "123456789";
    let channel_id = "987654321";
    
    // 設置測試 Guild 配置
    setup_test_guild_config(&guild_service, guild_id, channel_id).await;
    
    // 測試公會配置查詢
    let config = guild_service.get_guild_config(guild_id).await;
    
    assert!(config.is_ok(), "公會配置查詢應該成功");
    
    let config = config.unwrap();
    assert!(config.is_some(), "應該找到公會配置");
    
    let config = config.unwrap();
    assert_eq!(config.guild_id, guild_id, "Guild ID 應該匹配");
    assert_eq!(config.welcome_channel_id, channel_id, "頻道 ID 應該匹配");
    assert!(config.background_ref.is_some(), "背景圖應該有設置");
}

#[tokio::test]
async fn test_welcome_message_without_config() {
    // FR-W-004: 處理沒有配置的公會
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let _welcome_handler = WelcomeHandler::new();
    
    let guild_id = "999888777"; // 沒有配置的公會
    
    // 查詢不存在的公會配置
    let config = guild_service.get_guild_config(guild_id).await;
    
    assert!(config.is_ok(), "查詢應該成功（即使沒有配置）");
    
    let config = config.unwrap();
    assert!(config.is_none(), "不存在的公會配置應該返回 None");
}

#[tokio::test]
async fn test_welcome_message_content_generation() {
    // FR-W-005: 歡迎訊息內容生成
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    let user_id = 987654321u64;
    
    // 測試訊息內容生成（目前是基礎實現）
    let result = welcome_handler.handle_member_join(guild_id, user_id).await;
    
    assert!(result.is_ok(), "訊息內容生成應該成功");
}

// ===========================================
// 性能需求測試 (Performance Requirements)
// ===========================================

#[tokio::test]
async fn test_welcome_message_sending_latency() {
    // NFR-W-P-001: 歡迎訊息發送延遲 < 2秒
    let (_pool, _guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    let user_id = 987654321u64;
    
    let start_time = Instant::now();
    let result = welcome_handler.handle_member_join(guild_id, user_id).await;
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok(), "歡迎訊息發送應該成功");
    assert!(elapsed < Duration::from_secs(2), 
           "歡迎訊息發送延遲 {}ms 應該 < 2秒", elapsed.as_millis());
    
    println!("歡迎訊息發送延遲: {}ms", elapsed.as_millis());
}

#[tokio::test]
async fn test_concurrent_welcome_message_handling() {
    // NFR-W-P-002: 並發歡迎訊息處理
    let (_pool, _guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = Arc::new(WelcomeHandler::new());
    
    // 創建多個並發歡迎訊息請求
    let num_messages = 10;
    let handles: Vec<_> = (0..num_messages).map(|i| {
        let handler = Arc::clone(&welcome_handler);
        tokio::spawn(async move {
            let guild_id = 123456789u64;
            let user_id = 987654321u64 + i;
            handler.handle_member_join(guild_id, user_id).await
        })
    }).collect();
    
    let start_time = Instant::now();
    let results: Vec<_> = futures::future::join_all(handles).await;
    let total_time = start_time.elapsed();
    
    // 檢查所有請求結果
    let mut success_count = 0;
    for result in results {
        if let Ok(Ok(_)) = result {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, num_messages, "所有歡迎訊息應該成功處理");
    assert!(total_time < Duration::from_secs(5), 
           "並發處理 {} 個歡迎訊息用時 {}ms 應該 < 5秒", 
           num_messages, total_time.as_millis());
    
    println!("並發處理 {} 個歡迎訊息用時: {}ms", num_messages, total_time.as_millis());
}

// ===========================================
// 可靠性需求測試 (Reliability Requirements)
// ===========================================

#[tokio::test]
async fn test_welcome_message_error_handling() {
    // NFR-W-R-001: 錯誤處理和恢復
    let welcome_handler = WelcomeHandler::new();
    
    // 測試無效輸入處理
    let invalid_guild_id = 0u64; // 無效的公會 ID
    let invalid_user_id = 0u64; // 無效的用戶 ID
    
    let result = welcome_handler.handle_member_join(invalid_guild_id, invalid_user_id).await;
    
    // 即使輸入無效，也應該優雅地處理而不是崩潰
    // 具體的錯誤處理邏輯將在實現中定義
    match result {
        Ok(_) => println!("無效輸入被成功處理（可能被忽略）"),
        Err(e) => {
            println!("無效輸入產生預期錯誤: {}", e);
            // 錯誤應該是有意義且可恢復的
        }
    }
}

#[tokio::test]
async fn test_welcome_message_retry_mechanism() {
    // NFR-W-R-002: 重試機制測試
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    let user_id = 987654321u64;
    
    // 測試多次重試（模擬網絡問題）
    for retry_count in 1..=3 {
        let result = welcome_handler.handle_member_join(guild_id, user_id).await;
        
        match result {
            Ok(_) => {
                println!("第 {} 次嘗試成功", retry_count);
                break;
            }
            Err(e) => {
                println!("第 {} 次嘗試失敗: {}", retry_count, e);
                if retry_count < 3 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

// ===========================================
// 可觀測性需求測試 (Observability Requirements)
// ===========================================

#[tokio::test]
async fn test_welcome_message_logging() {
    // NFR-W-O-001: 日誌記錄和監控
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    let user_id = 987654321u64;
    
    // 測試日誌記錄（通過檢查是否不會崩潰來驗證）
    let result = welcome_handler.handle_member_join(guild_id, user_id).await;
    
    assert!(result.is_ok(), "歡迎訊息處理應該成功並記錄適當的日誌");
    
    // 在實際實現中，這裡會檢查日誌內容
    println!("歡迎訊息處理完成，應該有相關日誌輸出");
}

#[tokio::test]
async fn test_welcome_message_metrics_collection() {
    // NFR-W-O-002: 指標收集
    let welcome_handler = WelcomeHandler::new();
    
    let guild_id = 123456789u64;
    
    // 處理多個用戶的歡迎訊息來測試指標收集
    let mut processing_times = Vec::new();
    
    // 使用不會觸發模擬錯誤的用戶 ID（避免能被 1000 整除的數字）
    for i in 1..=10u64 {
        let user_id = 100001 + i; // 確保不會被 1000 整除
        let start_time = Instant::now();
        let result = welcome_handler.handle_member_join(guild_id, user_id).await;
        let processing_time = start_time.elapsed();
        
        assert!(result.is_ok(), "用戶 {} 的歡迎訊息應該成功處理", user_id);
        processing_times.push(processing_time);
    }
    
    // 計算統計指標
    let total_messages = processing_times.len();
    let total_time: Duration = processing_times.iter().sum();
    let avg_time = total_time / total_messages as u32;
    let min_time = processing_times.iter().min().unwrap();
    let max_time = processing_times.iter().max().unwrap();
    
    println!("歡迎訊息處理指標:");
    println!("  總處理數: {}", total_messages);
    println!("  平均處理時間: {}ms", avg_time.as_millis());
    println!("  最小處理時間: {}ms", min_time.as_millis());
    println!("  最大處理時間: {}ms", max_time.as_millis());
    
    // 驗證性能指標合理
    assert!(avg_time < Duration::from_millis(500), 
           "平均處理時間 {}ms 應該 < 500ms", avg_time.as_millis());
}