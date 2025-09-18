use chrono::Utc;
use droas_bot::database::schema::{GuildConfig, GuildConfigService};
use droas_bot::discord::{EventHandler, EventResult, TestMemberJoinEvent};
use droas_bot::discord::{GatewayManager, GatewayStatus};
use droas_bot::handlers::welcome::WelcomeHandler;
use sqlx::SqlitePool;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

/// 創建測試用的資料庫和服務
async fn create_test_services() -> (SqlitePool, GuildConfigService, NamedTempFile) {
    let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
    let database_url = format!("sqlite://{}", temp_file.path().display());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
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
        "#,
    )
    .execute(&pool)
    .await
    .expect("無法創建測試表格");

    let guild_service = GuildConfigService::new(pool.clone());

    (pool, guild_service, temp_file)
}

/// 創建測試用的 Guild 配置
async fn setup_test_guild_config(service: &GuildConfigService, guild_id: &str, channel_id: &str) {
    let config = GuildConfig {
        guild_id: guild_id.to_string(),
        welcome_channel_id: channel_id.to_string(),
        background_ref: None,
        updated_at: Utc::now(),
    };

    service
        .upsert_guild_config(&config)
        .await
        .expect("無法創建測試 Guild 配置");
}

#[tokio::test]
async fn test_guild_member_add_event_reception() {
    // F-REQ-001: GUILD_MEMBER_ADD 事件接收與路由
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let mut gateway_manager = GatewayManager::new();
    let welcome_handler = WelcomeHandler::new();

    // 設置 Gateway 為已連接狀態
    gateway_manager.set_status(GatewayStatus::Connected);

    // 設置測試 Guild 配置
    setup_test_guild_config(&guild_service, "123456789", "987654321").await;

    // 創建測試事件
    let test_event = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 555666777,
        username: "TestUser".to_string(),
        timestamp: Instant::now(),
    };

    // 測試事件接收（這裡我們模擬事件處理器接收事件）
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 驗證事件處理器能夠接收事件
    let result = event_handler.handle_member_join_event(&test_event).await;

    // 事件應該被成功接收和處理
    assert!(result.is_ok(), "事件處理應該成功");

    // 驗證 Gateway 連接狀態仍然健康
    assert!(
        gateway_manager.is_connection_healthy(),
        "Gateway 連接應該保持健康"
    );
}

#[tokio::test]
async fn test_event_validation_valid_event() {
    // F-REQ-002: 事件有效性驗證（檢查事件格式、來源）
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 測試有效事件
    let valid_event = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 555666777,
        username: "ValidUser".to_string(),
        timestamp: Instant::now(),
    };

    // 驗證有效事件通過驗證
    let is_valid = event_handler.validate_event(&valid_event);
    assert!(is_valid, "有效事件應該通過驗證");
}

#[tokio::test]
async fn test_event_validation_invalid_event() {
    // F-REQ-002: 事件有效性驗證（檢查無效事件）
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 測試無效事件（Guild ID 為 0）
    let invalid_event_1 = TestMemberJoinEvent {
        guild_id: 0,
        user_id: 555666777,
        username: "InvalidUser".to_string(),
        timestamp: Instant::now(),
    };

    // 測試無效事件（User ID 為 0）
    let invalid_event_2 = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 0,
        username: "InvalidUser".to_string(),
        timestamp: Instant::now(),
    };

    // 測試無效事件（用戶名為空）
    let invalid_event_3 = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 555666777,
        username: "".to_string(),
        timestamp: Instant::now(),
    };

    // 驗證無效事件被拒絕
    assert!(
        !event_handler.validate_event(&invalid_event_1),
        "Guild ID 為 0 的事件應該被拒絕"
    );
    assert!(
        !event_handler.validate_event(&invalid_event_2),
        "User ID 為 0 的事件應該被拒絕"
    );
    assert!(
        !event_handler.validate_event(&invalid_event_3),
        "用戶名為空的事件應該被拒絕"
    );
}

#[tokio::test]
async fn test_event_deduplication() {
    // F-REQ-003: 事件去重處理（防止重複處理）
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 設置測試 Guild 配置
    setup_test_guild_config(&event_handler.guild_service, "123456789", "987654321").await;

    let test_event = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 555666777,
        username: "TestUser".to_string(),
        timestamp: Instant::now(),
    };

    // 第一次處理事件
    let result1 = event_handler.handle_member_join_event(&test_event).await;
    assert!(result1.is_ok(), "第一次事件處理應該成功");

    // 立即重複處理相同事件（應該被去重）
    let result2 = event_handler.handle_member_join_event(&test_event).await;

    // 檢查去重結果（根據實作邏輯，重複事件應該被拒絕或跳過）
    match result2 {
        Ok(EventResult::Skipped(reason)) => {
            assert!(
                reason.contains("duplicate") || reason.contains("重複"),
                "跳過原因應該提到重複事件"
            );
        }
        Err(e) => {
            assert!(
                e.to_string().contains("duplicate") || e.to_string().contains("重複"),
                "錯誤訊息應該提到重複事件"
            );
        }
        _ => panic!("重複事件應該被去重處理"),
    }
}

#[tokio::test]
async fn test_gateway_connection_management() {
    // F-REQ-004: Gateway 連接管理與自動重連
    let mut gateway_manager = GatewayManager::new();

    // 測試初始狀態
    assert_eq!(gateway_manager.get_status(), GatewayStatus::Disconnected);
    assert!(!gateway_manager.is_connection_healthy());

    // 測試連接流程
    gateway_manager.set_status(GatewayStatus::Connecting);
    assert_eq!(gateway_manager.get_status(), GatewayStatus::Connecting);

    gateway_manager.set_status(GatewayStatus::Connected);
    assert_eq!(gateway_manager.get_status(), GatewayStatus::Connected);

    // 更新心跳以保持連接健康
    gateway_manager.update_heartbeat(45000); // 45 秒間隔
    assert!(gateway_manager.is_connection_healthy());

    // 測試斷線和重連計數
    gateway_manager.set_status(GatewayStatus::Disconnected);
    gateway_manager.increment_reconnect_count();
    assert_eq!(gateway_manager.get_reconnect_count(), 1);
}

#[tokio::test]
async fn test_auto_reconnection() {
    // F-REQ-004: 自動重連機制測試
    let mut gateway_manager = GatewayManager::new();

    // 模擬連接錯誤
    gateway_manager.set_status(GatewayStatus::Error("Connection lost".to_string()));
    assert!(!gateway_manager.is_connection_healthy());

    // 模擬重連嘗試
    for i in 1..=3 {
        gateway_manager.increment_reconnect_count();
        assert_eq!(gateway_manager.get_reconnect_count(), i);

        // 模擬重連成功
        if i == 3 {
            gateway_manager.set_status(GatewayStatus::Connected);
            gateway_manager.update_heartbeat(45000);
            assert!(gateway_manager.is_connection_healthy());
        }
    }

    // 驗證可靠性評分因重連而降低
    let reliability_score = gateway_manager.calculate_reliability_score();

    // 由於有重連，評分應該受到影響（具體數值取決於實作）
    assert!(reliability_score < 100, "有重連的情況下評分應該低於100");
}

#[tokio::test]
async fn test_event_processing_pipeline() {
    // F-REQ-005: 事件處理流水線（異步處理、排隊）
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 設置測試 Guild 配置
    setup_test_guild_config(&event_handler.guild_service, "123456789", "987654321").await;

    // 創建多個測試事件
    let events: Vec<TestMemberJoinEvent> = (0..5)
        .map(|i| TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777 + i,
            username: format!("TestUser{}", i),
            timestamp: Instant::now(),
        })
        .collect();

    // 並發處理多個事件（測試異步處理能力）
    let start_time = Instant::now();

    // 將事件處理器包裝在 Arc 中以支援多個並發任務
    let event_handler = std::sync::Arc::new(event_handler);

    let handles: Vec<_> = events
        .into_iter()
        .map(|event| {
            let handler = std::sync::Arc::clone(&event_handler);
            tokio::spawn(async move { handler.handle_member_join_event(&event).await })
        })
        .collect();

    // 等待所有事件處理完成
    let results: Vec<_> = futures::future::join_all(handles).await;
    let processing_time = start_time.elapsed();

    // 驗證所有事件處理結果
    for result in results {
        let event_result = result.expect("任務不應該 panic").expect("事件處理應該成功");

        // 根據實際的 EventResult 類型進行檢查
        match event_result {
            EventResult::Success => {}    // 處理成功
            EventResult::Skipped(_) => {} // 被跳過（可能由於去重）
            _ => panic!("未預期的事件處理結果"),
        }
    }

    // 驗證處理時間合理（應該並發處理，而非串行）
    assert!(
        processing_time < Duration::from_millis(1000),
        "並發處理5個事件應該在1秒內完成"
    );
}

#[tokio::test]
async fn test_async_event_handling() {
    // F-REQ-005: 異步事件處理測試
    let (_pool, guild_service, _temp_file) = create_test_services().await;
    let welcome_handler = WelcomeHandler::new();
    let event_handler = EventHandler::new(guild_service, welcome_handler);

    // 設置測試 Guild 配置
    setup_test_guild_config(&event_handler.guild_service, "123456789", "987654321").await;

    let test_event = TestMemberJoinEvent {
        guild_id: 123456789,
        user_id: 555666777,
        username: "AsyncTestUser".to_string(),
        timestamp: Instant::now(),
    };

    // 測試異步事件處理不會阻塞
    let start_time = Instant::now();

    let result = tokio::time::timeout(
        Duration::from_millis(500), // 500ms 超時
        event_handler.handle_member_join_event(&test_event),
    )
    .await;

    let processing_time = start_time.elapsed();

    // 驗證事件在超時時間內完成
    assert!(result.is_ok(), "事件處理不應該超時");
    assert!(result.unwrap().is_ok(), "事件處理應該成功");

    // 驗證處理時間符合性能要求（< 500ms）
    assert!(
        processing_time < Duration::from_millis(500),
        "事件處理應該在500ms內完成"
    );
}
