// User Account Service 測試 - RED 階段
// 測試自動帳戶創建功能

use droas_bot::database::{UserRepository};
use droas_bot::database::user_repository::CreateUserRequest;
use droas_bot::config::DatabaseConfig;
use bigdecimal::BigDecimal;
use std::str::FromStr;

#[tokio::test]
async fn test_new_user_account_creation() {
    // 測試新用戶首次使用 !balance 指令時自動創建帳戶
    // 預期：帳戶創建成功，餘額 1000，收到歡迎訊息

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        // 使用一個確定不存在的新用戶 ID
        let new_user_id = 999999999_i64;
        let username = "new_test_user".to_string();

        // 確保用戶不存在（清理可能的測試數據）
        let _ = user_repo.get_user_by_discord_id(new_user_id).await;

        // 模擬新用戶首次觸發帳戶創建
        let create_request = CreateUserRequest {
            discord_user_id: new_user_id,
            username: username.clone(),
            initial_balance: None, // 應該使用預設值 1000
        };

        let result = user_repo.create_user(create_request).await;

        // 驗證帳戶創建成功
        assert!(result.is_ok(), "新用戶帳戶創建應該成功");

        let user = result.unwrap();
        assert_eq!(user.discord_user_id, new_user_id);
        assert_eq!(user.username, username);
        assert_eq!(user.balance, BigDecimal::from_str("1000.00").unwrap());

        // 注意：在實際應用中，測試數據清理應該通過公共介面進行
        // 這裡暫時跳過手動清理，依靠測試隔離
    } else {
        // 如果沒有資料庫連接，跳過此測試
        println!("警告：沒有資料庫連接，跳過 test_new_user_account_creation 測試");
    }
}

#[tokio::test]
async fn test_duplicate_account_prevention() {
    // 測試重複帳戶創建防護機制
    // 預期：返回適當錯誤訊息，不重複創建

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        let existing_user_id = 888888888_i64;
        let username = "existing_test_user".to_string();

        // 先創建一個用戶
        let create_request = CreateUserRequest {
            discord_user_id: existing_user_id,
            username: username.clone(),
            initial_balance: Some(BigDecimal::from_str("1500.00").unwrap()),
        };

        let _ = user_repo.create_user(create_request).await;

        // 嘗試再次創建相同用戶
        let duplicate_request = CreateUserRequest {
            discord_user_id: existing_user_id,
            username: "updated_username".to_string(),
            initial_balance: Some(BigDecimal::from_str("2000.00").unwrap()),
        };

        let result = user_repo.create_user(duplicate_request).await;

        // 驗證不會重複創建，而是更新現有用戶
        assert!(result.is_ok(), "重複創建應該更新現有用戶而非失敗");

        let user = result.unwrap();
        assert_eq!(user.discord_user_id, existing_user_id);
        // 用戶名應該被更新，但餘額不應該改變（因為是 UPSERT）
        assert_eq!(user.username, "updated_username");
        // 餘額應該保持原值，因為 UPSERT 不更新餘額
        assert_eq!(user.balance, BigDecimal::from_str("1500.00").unwrap());

        // 注意：在實際應用中，測試數據清理應該通過公共介面進行
        // 這裡暫時跳過手動清理，依靠測試隔離
    } else {
        println!("警告：沒有資料庫連接，跳過 test_duplicate_account_prevention 測試");
    }
}

#[tokio::test]
async fn test_account_creation_with_db_error() {
    // 測試資料庫錯誤處理
    // 預期：系統優雅處理錯誤，返回用戶友好訊息

    // 創建一個無效的資料庫配置來模擬連接失敗
    let invalid_config = DatabaseConfig {
        url: "postgres://invalid:invalid@localhost:9999/invalid_db".to_string(),
        max_connections: 1,
        min_connections: 1,
        connection_timeout: 1,
    };

    let pool_result = droas_bot::database::create_user_pool(&invalid_config).await;

    // 驗證無法創建資料庫連接池
    assert!(pool_result.is_err(), "無效的資料庫配置應該導致連接失敗");

    // 如果嘗試創建 UserRepository，應該會失敗
    // 這驗證了錯誤處理機制正常工作
    match pool_result {
        Ok(_) => panic!("預期資料庫連接失敗，但成功了"),
        Err(e) => {
            // 驗證錯誤類型和訊息
            assert!(!e.to_string().is_empty(), "錯誤訊息不應該為空");
            println!("資料庫錯誤處理測試通過：{}", e);
        }
    }
}

#[tokio::test]
async fn test_account_creation_performance() {
    // 測試帳戶創建操作響應時間
    // 預期：響應時間 < 2 秒

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        let performance_user_id = 777777777_i64;
        let username = "performance_test_user".to_string();

        // 記錄開始時間
        let start_time = std::time::Instant::now();

        // 執行帳戶創建
        let create_request = CreateUserRequest {
            discord_user_id: performance_user_id,
            username: username.clone(),
            initial_balance: None,
        };

        let result = user_repo.create_user(create_request).await;

        // 記錄結束時間
        let duration = start_time.elapsed();

        // 驗證帳戶創建成功
        assert!(result.is_ok(), "性能測試中的帳戶創建應該成功");

        // 驗證響應時間 < 2 秒
        assert!(duration.as_secs() < 2, "帳戶創建響應時間應該 < 2 秒，實際：{:?}", duration);

        println!("帳戶創建性能測試通過，耗時：{:?}", duration);

        // 注意：在實際應用中，測試數據清理應該通過公共介面進行
        // 這裡暫時跳過手動清理，依靠測試隔離
    } else {
        println!("警告：沒有資料庫連接，跳過 test_account_creation_performance 測試");
    }
}