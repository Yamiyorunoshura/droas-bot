// Security Service 測試 - RED 階段
// 測試用戶驗證和重複檢測功能

use droas_bot::database::{UserRepository};
use droas_bot::database::user_repository::CreateUserRequest;
use droas_bot::config::DatabaseConfig;
use bigdecimal::BigDecimal;
use std::str::FromStr;

#[tokio::test]
async fn test_duplicate_account_creation_prevention() {
    // 測試用戶嘗試創建已存在的帳戶
    // 預期：返回錯誤訊息「帳戶已存在」且不創建新帳戶

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        // 先創建一個用戶
        let existing_user_id = 666666666_i64;
        let username = "existing_security_user".to_string();

        let create_request = CreateUserRequest {
            discord_user_id: existing_user_id,
            username: username.clone(),
            initial_balance: None,
        };

        let _ = user_repo.create_user(create_request).await;

        // 使用 Security Service 創建相同帳戶
        // 這應該會失敗，因為帳戶已存在
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 模擬重複帳戶創建請求
        let result = security_service.validate_and_create_user(existing_user_id, username.clone()).await;

        // 驗證返回適當錯誤
        assert!(result.is_err(), "重複帳戶創建應該失敗");

        match result {
            Err(error) => {
                let error_msg = error.to_string();
                assert!(error_msg.contains("帳戶已存在") || error_msg.contains("already exists"),
                       "錯誤訊息應該包含 '帳戶已存在'，實際：{}", error_msg);
            }
            Ok(_) => panic!("預期重複帳戶創建失敗，但成功了"),
        }
    } else {
        println!("警告：沒有資料庫連接，跳過 test_duplicate_account_creation_prevention 測試");
    }
}

#[tokio::test]
async fn test_successful_user_authentication() {
    // 測試有效用戶進行身份驗證
    // 預期：驗證成功，返回用戶帳戶資訊

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        // 創建一個測試用戶
        let valid_user_id = 555555555_i64;
        let username = "valid_auth_user".to_string();

        let create_request = CreateUserRequest {
            discord_user_id: valid_user_id,
            username: username.clone(),
            initial_balance: None,
        };

        let _ = user_repo.create_user(create_request).await;

        // 使用 Security Service 驗證用戶
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        let result = security_service.authenticate_user(valid_user_id).await;

        // 驗證驗證成功
        assert!(result.is_ok(), "有效用戶驗證應該成功");

        let user = result.unwrap();
        assert_eq!(user.discord_user_id, valid_user_id);
        assert_eq!(user.username, username);
        assert_eq!(user.balance, BigDecimal::from_str("1000.00").unwrap());
    } else {
        println!("警告：沒有資料庫連接，跳過 test_successful_user_authentication 測試");
    }
}

#[tokio::test]
async fn test_invalid_user_authentication() {
    // 測試無效用戶嘗試驗證
    // 預期：驗證失敗，返回適當錯誤訊息

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        // 使用一個不存在的用戶 ID
        let invalid_user_id = 444444444_i64;

        // 使用 Security Service 驗證無效用戶
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        let result = security_service.authenticate_user(invalid_user_id).await;

        // 驗證驗證失敗
        assert!(result.is_err(), "無效用戶驗證應該失敗");

        match result {
            Err(error) => {
                let error_msg = error.to_string();
                assert!(error_msg.contains("用戶不存在") || error_msg.contains("not found") || error_msg.contains("invalid"),
                       "錯誤訊息應該表明用戶不存在，實際：{}", error_msg);
            }
            Ok(_) => panic!("預期無效用戶驗證失敗，但成功了"),
        }
    } else {
        println!("警告：沒有資料庫連接，跳過 test_invalid_user_authentication 測試");
    }
}

#[tokio::test]
async fn test_security_event_logging() {
    // 測試惡意輸入驗證嘗試
    // 預期：記錄安全事件並阻止操作

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);

        // 測試惡意輸入（負數用戶 ID）
        let malicious_user_id = -1_i64;

        // 使用 Security Service 處理惡意輸入
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        let result = security_service.authenticate_user(malicious_user_id).await;

        // 驗證惡意輸入被拒絕
        assert!(result.is_err(), "惡意輸入應該被拒絕");

        match result {
            Err(error) => {
                let error_msg = error.to_string();
                assert!(error_msg.contains("無效") || error_msg.contains("invalid") || error_msg.contains("malicious"),
                       "錯誤訊息應該表明輸入無效，實際：{}", error_msg);
            }
            Ok(_) => panic!("預期惡意輸入驗證失敗，但成功了"),
        }

        // TODO: 驗證安全事件被記錄（需要實作日誌檢查機制）

    } else {
        println!("警告：沒有資料庫連接，跳過 test_security_event_logging 測試");
    }
}

#[tokio::test]
async fn test_validation_with_database_unavailable() {
    // 測試資料庫不可用時的驗證行為
    // 預期：提供適當錯誤處理，優雅降級

    // 創建一個無效的資料庫配置
    let invalid_config = DatabaseConfig {
        url: "postgres://invalid:invalid@localhost:9999/invalid_db".to_string(),
        max_connections: 1,
        min_connections: 1,
        connection_timeout: 1,
    };

    let pool_result = droas_bot::database::create_user_pool(&invalid_config).await;

    // 驗證無法創建資料庫連接池
    assert!(pool_result.is_err(), "無效的資料庫配置應該導致連接失敗");

    // 嘗試創建 Security Service
    match pool_result {
        Ok(_) => panic!("預期資料庫連接失敗，但成功了"),
        Err(_) => {
            // 在資料庫不可用的情況下，Security Service 應該能夠適當處理
            // 這個測試主要驗證錯誤處理邏輯
            println!("資料庫不可用時的驗證錯誤處理測試通過");
        }
    }
}

// ===== N2 計劃新增的安全驗證測試案例 (RED 階段) =====

#[tokio::test]
async fn test_discord_user_id_authentication() {
    // 測試有效 Discord 用戶 ID 進行交易
    // 預期：交易成功執行 (NFR-S-001)

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 測試有效的 Discord 用戶 ID
        let valid_user_ids = vec![
            123456789_i64,
            987654321098765432_i64,
            12345_i64,
        ];

        for user_id in valid_user_ids {
            let result = security_service.validate_discord_user_id(user_id);
            assert!(result.is_ok(), "有效的 Discord 用戶 ID {} 應該通過驗證", user_id);
            assert!(result.unwrap(), "有效的 Discord 用戶 ID {} 應該返回 true", user_id);
        }

        // 測試無效的 Discord 用戶 ID
        let invalid_user_ids = vec![
            -1_i64,  // 負數
            0_i64,   // 零
            -12345_i64, // 負數
        ];

        for user_id in invalid_user_ids {
            let result = security_service.validate_discord_user_id(user_id);
            assert!(result.is_err(), "無效的 Discord 用戶 ID {} 應該被拒絕", user_id);
        }

    } else {
        println!("警告：沒有資料庫連接，跳過 test_discord_user_id_authentication 測試");
    }
}

#[tokio::test]
async fn test_invalid_discord_user_id_rejection() {
    // 測試無效 Discord 用戶 ID 嘗試交易
    // 預期：交易被拒絕並返回錯誤 (NFR-S-001)

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);
        let mut security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 測試黑名單用戶
        let blacklisted_user_id = 999999999_i64;
        security_service.add_user_to_blacklist(blacklisted_user_id);

        let result = security_service.validate_discord_user_id(blacklisted_user_id);
        assert!(result.is_err(), "黑名單用戶 {} 應該被拒絕", blacklisted_user_id);

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("封鎖") || error_msg.contains("blocked"),
                "黑名單用戶錯誤訊息應該包含封鎖提示，實際：{}", error_msg);

    } else {
        println!("警告：沒有資料庫連接，跳過 test_invalid_discord_user_id_rejection 測試");
    }
}

#[tokio::test]
async fn test_input_sanitization() {
    // 測試輸入包含惡意腳本和 SQL 注入
    // 預期：輸入被清理和轉義 (NFR-S-002)

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 測試 SQL 注入攻擊
        let sql_injection_inputs = vec![
            "'; DROP TABLE users; --",
            "admin' OR '1'='1",
            "'; DELETE FROM transactions; --",
            "1'; UPDATE users SET balance = 999999; --",
        ];

        for malicious_input in sql_injection_inputs {
            let result = security_service.sanitize_string_input(malicious_input, 100);
            assert!(result.is_ok(), "SQL 注入輸入應該被處理而不是直接拒絕：{}", malicious_input);

            let sanitized = result.unwrap();
            assert!(!sanitized.contains("'"), "清理後的輸入不應包含單引號：{}", sanitized);
            assert!(!sanitized.contains(";"), "清理後的輸入不應包含分號：{}", sanitized);
            assert!(!sanitized.contains("--"), "清理後的輸入不應包含註釋符號：{}", sanitized);
        }

        // 測試 XSS 攻擊
        let xss_inputs = vec![
            "<script>alert('xss')</script>",
            "<img src=x onerror=alert('xss')>",
            "javascript:alert('xss')",
            "<iframe src='javascript:alert(1)'></iframe>",
        ];

        for xss_input in xss_inputs {
            let result = security_service.sanitize_string_input(xss_input, 100);
            assert!(result.is_ok(), "XSS 輸入應該被處理而不是直接拒絕：{}", xss_input);

            let sanitized = result.unwrap();
            // 確保危險的 HTML 標籤被移除或轉義
            assert!(!sanitized.contains("<script>"), "清理後的輸入不應包含 script 標籤：{}", sanitized);
            assert!(!sanitized.contains("javascript:"), "清理後的輸入不應包含 javascript 協議：{}", sanitized);
        }

    } else {
        println!("警告：沒有資料庫連接，跳過 test_input_sanitization 測試");
    }
}

#[tokio::test]
async fn test_format_validation() {
    // 測試輸入格式不符合預期
    // 預期：返回格式錯誤訊息 (NFR-S-002)

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 測試無效用戶名稱格式
        let long_username = "a".repeat(33);
        let invalid_usernames = vec![
            "",                    // 空字符串
            "a",                   // 太短
            long_username.as_str(), // 太長
            "user@domain",         // 包含無效字符
            "user#123",           // 包含無效字符
            "   ",                // 只有空格
            "\t\n",              // 只有控制字符
        ];

        for invalid_username in &invalid_usernames {
            let result = security_service.validate_username(invalid_username);
            assert!(result.is_err(), "無效用戶名稱 '{}' 應該被拒絕", invalid_username);
        }

        // 測試無效金額格式
        let invalid_amounts = vec![
            "abc",                 // 非數字
            "-100",                // 負數
            "0",                   // 零
            "100.999",             // 超過兩位小數
            "1e10",                // 科學記號
            "",                    // 空字符串
            "   ",                 // 只有空格
            "1000000.01",         // 超過上限
        ];

        for invalid_amount in invalid_amounts {
            let result = security_service.validate_amount(invalid_amount);
            assert!(result.is_err(), "無效金額 '{}' 應該被拒絕", invalid_amount);
        }

    } else {
        println!("警告：沒有資料庫連接，跳過 test_format_validation 測試");
    }
}

#[tokio::test]
async fn test_comprehensive_security_validation() {
    // 綜合安全驗證測試
    // 預期：所有安全檢查通過，交易被允許

    let database_config = DatabaseConfig::from_env().unwrap();
    let pool_result = droas_bot::database::create_user_pool(&database_config).await;

    if let Ok(pool) = pool_result {
        let user_repo = UserRepository::new(pool);
        let security_service = droas_bot::services::security_service::SecurityService::new(user_repo.clone()).unwrap();

        // 創建一個測試用戶
        let test_user_id = 888888888_i64;
        let test_username = "security_test_user".to_string();

        let create_request = CreateUserRequest {
            discord_user_id: test_user_id,
            username: test_username.clone(),
            initial_balance: None,
        };

        let _ = user_repo.create_user(create_request).await;

        // 測試完整的身份驗證流程
        let auth_result = security_service.authenticate_user(test_user_id).await;
        assert!(auth_result.is_ok(), "有效用戶身份驗證應該成功");

        // 測試用戶名稱驗證
        let username_result = security_service.validate_username(&test_username);
        assert!(username_result.is_ok(), "有效用戶名稱應該通過驗證");

        // 測試金額驗證
        let valid_amounts = vec!["100", "100.50", "0.01", "999999.99"];
        for amount in valid_amounts {
            let amount_result = security_service.validate_amount(amount);
            assert!(amount_result.is_ok(), "有效金額 {} 應該通過驗證", amount);
        }

        // 測試自我轉帳防護
        let self_transfer_result = security_service.validate_no_self_transfer(test_user_id, test_user_id);
        assert!(self_transfer_result.is_err(), "自我轉帳應該被拒絕");

        // 測試不同用戶間轉帳
        let other_user_id = 777777777_i64;
        let valid_transfer_result = security_service.validate_no_self_transfer(test_user_id, other_user_id);
        assert!(valid_transfer_result.is_ok(), "不同用戶間轉帳應該被允許");

    } else {
        println!("警告：沒有資料庫連接，跳過 test_comprehensive_security_validation 測試");
    }
}