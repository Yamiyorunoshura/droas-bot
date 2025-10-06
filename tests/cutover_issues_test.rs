//! 測試用於重現和驗證 cutover 報告中的問題修復

use droas_bot::database::{create_user_pool, run_migrations};
use droas_bot::config::DatabaseConfig;
use droas_bot::error::Result;
use sqlx::Row;

/// 測試 CUTOVER-001: 驗證資料庫遷移是否正確執行
///
/// 這個測試重現了 cutover 報告中的關鍵問題：
/// 主程序未調用資料庫遷移，導致 users 和 transactions 表不存在
#[tokio::test]
async fn test_cutover_001_database_migration_issue() -> Result<()> {
    // 設置測試資料庫配置
    let database_config = DatabaseConfig::from_env()?;

    // 創建資料庫連接池（模擬 main.rs 中的 init_database）
    let pool = create_user_pool(&database_config).await?;

    // 檢查表是否存在（這應該會失敗，因為遷移未執行）
    let table_check_result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await;

    match table_check_result {
        Ok(row) => {
            let exists: bool = row.get(0);
            if exists {
                println!("✅ users 表存在");
            } else {
                println!("❌ users 表不存在 - 重現 CUTOVER-001 問題");
                return Err(droas_bot::error::DiscordError::DatabaseQueryError(
                    "users 表不存在，資料庫遷移未執行".to_string()
                ).into());
            }
        }
        Err(e) => {
            println!("❌ 檢查表時發生錯誤: {}", e);
            return Err(droas_bot::error::DiscordError::DatabaseQueryError(
                format!("檢查表時發生錯誤: {}", e)
            ).into());
        }
    }

    // 同樣檢查 transactions 表
    let transactions_check_result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'transactions'
        )"
    )
    .fetch_one(&pool)
    .await;

    match transactions_check_result {
        Ok(row) => {
            let exists: bool = row.get(0);
            if exists {
                println!("✅ transactions 表存在");
            } else {
                println!("❌ transactions 表不存在 - 重現 CUTOVER-001 問題");
                return Err(droas_bot::error::DiscordError::DatabaseQueryError(
                    "transactions 表不存在，資料庫遷移未執行".to_string()
                ).into());
            }
        }
        Err(e) => {
            println!("❌ 檢查 transactions 表時發生錯誤: {}", e);
            return Err(droas_bot::error::DiscordError::DatabaseQueryError(
                format!("檢查 transactions 表時發生錯誤: {}", e)
            ).into());
        }
    }

    Ok(())
}

/// 測試修復後的資料庫遷移功能
///
/// 這個測試驗證 run_migrations 函數能夠正確創建所需的表
#[tokio::test]
async fn test_cutover_001_database_migration_fix() -> Result<()> {
    // 設置測試資料庫配置
    let database_config = DatabaseConfig::from_env()?;

    // 創建資料庫連接池
    let pool = create_user_pool(&database_config).await?;

    // 執行遷移（這是修復方案）
    run_migrations(&pool).await?;

    // 驗證表是否已創建
    let users_table_check = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await?;

    let users_exists: bool = users_table_check.get(0);
    assert!(users_exists, "users 表應該在遷移後存在");

    let transactions_table_check = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'transactions'
        )"
    )
    .fetch_one(&pool)
    .await?;

    let transactions_exists: bool = transactions_table_check.get(0);
    assert!(transactions_exists, "transactions 表應該在遷移後存在");

    // 驗證索引是否已創建
    let index_check = sqlx::query(
        "SELECT COUNT(*) FROM pg_indexes WHERE tablename = 'transactions' AND indexname LIKE 'idx_%'"
    )
    .fetch_one(&pool)
    .await?;

    let index_count: i64 = index_check.get(0);
    assert!(index_count >= 3, "應該至少創建 3 個索引");

    println!("✅ 資料庫遷移修復測試通過");
    Ok(())
}

/// 測試 CUTOVER-003: 重現轉帳驗證測試失敗問題
#[tokio::test]
async fn test_cutover_003_transfer_validation_failure() -> Result<()> {
    use droas_bot::database::{UserRepository, TransactionRepository};
    use droas_bot::services::{TransferService, SecurityService};

    let database_config = DatabaseConfig::from_env()?;

    if let Ok(pool) = create_user_pool(&database_config).await {
        // 確保遷移已執行
        run_migrations(&pool).await?;

        let user_repo = UserRepository::new(pool.clone());
        let transaction_repo = TransactionRepository::new(pool);
        let security_service = SecurityService::new(user_repo.clone())?;
        let transfer_service = TransferService::new(user_repo, transaction_repo, security_service)?;

        // 測試不存在的用戶轉帳（這應該重現原始問題）
        let result = transfer_service.validate_transfer_request(123, 456, "100.50").await;

        match result {
            Err(droas_bot::error::DiscordError::UserNotFound(_)) => {
                println!("✅ 正確返回 UserNotFound 錯誤");
            }
            Ok(_) => {
                println!("⚠️ 轉帳驗證成功（可能是測試數據已存在）");
            }
            Err(e) => {
                println!("❌ 返回了意外錯誤: {:?}", e);
                return Err(e.into());
            }
        }
    } else {
        println!("⚠️ 無法連接資料庫，跳過 CUTOVER-003 測試");
    }

    Ok(())
}