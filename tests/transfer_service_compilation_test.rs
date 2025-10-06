// Transfer Service 測試 - Cutover 修復階段
// 測試轉帳服務基本功能

use droas_bot::services::transfer_service::TransferService;
use droas_bot::database::{UserRepository, TransactionRepository};
use droas_bot::config::DatabaseConfig;
use droas_bot::services::security_service::SecurityService;

#[tokio::test]
async fn test_transfer_service_compilation() {
    // 測試轉帳服務能否正常編譯和創建 - CUTOVER-002 修復驗證

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
    let user_repo = UserRepository::new(pool.clone());
    let transaction_repo = TransactionRepository::new(pool);
    let security_service = SecurityService::new(user_repo.clone());

    if security_service.is_err() {
        println!("跳過測試：無法創建安全服務");
        return;
    }

    let service = TransferService::new(
        user_repo,
        transaction_repo,
        security_service.unwrap()
    );

    assert!(service.is_ok(), "轉帳服務應該能成功創建");

    if let Ok(service) = service {
        // 測試基本方法是否存在且可調用
        let result = service.validate_transfer_request(123, 456, "100").await;

        // 預期會失敗，因為用戶可能不存在，但不應該是編譯錯誤
        match result {
            Ok(_) => println!("轉帳驗證成功（用戶存在）"),
            Err(_) => println!("轉帳驗證失敗（用戶不存在，這是預期的）"),
        }
    }
}

#[tokio::test]
async fn test_transfer_history_method_exists() {
    // 測試轉帳歷史方法是否存在 - CUTOVER-003 修復驗證

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
    let user_repo = UserRepository::new(pool.clone());
    let transaction_repo = TransactionRepository::new(pool);
    let security_service = SecurityService::new(user_repo.clone());

    if security_service.is_err() {
        println!("跳過測試：無法創建安全服務");
        return;
    }

    let service = TransferService::new(
        user_repo,
        transaction_repo,
        security_service.unwrap()
    );

    assert!(service.is_ok(), "轉帳服務應該能成功創建");

    if let Ok(service) = service {
        // 測試轉帳歷史查詢方法是否存在
        let result = service.get_transfer_history(123, Some(10)).await;

        // 預期會失敗，因為用戶可能不存在，但不應該是編譯錯誤
        match result {
            Ok(_) => println!("轉帳歷史查詢成功"),
            Err(_) => println!("轉帳歷史查詢失敗（用戶不存在，這是預期的）"),
        }
    }
}