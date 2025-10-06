// Cutover 問題測試 - RED 階段
// 測試 CUTOVER-001: 資料庫時區類型不匹配問題

use droas_bot::database::UserRepository;
use droas_bot::database::user_repository::CreateUserRequest;
use droas_bot::config::DatabaseConfig;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use sqlx::Row;

#[tokio::test]
async fn test_user_account_creation_timezone_issue() {
    // 測試用戶帳戶創建中的時區類型問題 - CUTOVER-001 驗證

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
    let user_repo = UserRepository::new(pool);

    // 創建測試用戶請求
    let test_user_id = 999999i64; // 使用高數值避免與現有用戶衝突
    let create_request = CreateUserRequest {
        discord_user_id: test_user_id,
        username: "TimezoneTestUser".to_string(),
        initial_balance: Some(BigDecimal::from_str("1000.00").unwrap()),
    };

    // 嘗試創建用戶 - 這應該會因為時區類型問題而失敗
    let result = user_repo.create_user(create_request).await;

    match result {
        Ok(user) => {
            println!("用戶創建成功 - 可能已經修復時區問題");
            println!("用戶資訊: ID={}, Username={}, Created_at={}",
                user.discord_user_id, user.username, user.created_at);

            // 注意：無法清理測試數據，因為 UserRepository.pool 是私有的
        }
        Err(e) => {
            println!("用戶創建失敗 - 確認存在時區類型問題");
            println!("錯誤: {}", e);

            // 檢查是否是時區類型錯誤
            let error_str = e.to_string();
            if error_str.contains("TIMESTAMP") && error_str.contains("TIMESTAMPTZ") {
                println!("確認這是 CUTOVER-001 時區類型不匹配問題");
            } else if error_str.contains("database") {
                println!("資料庫相關錯誤，可能是連接問題");
            }
        }
    }
}

#[tokio::test]
async fn test_database_schema_timezone_columns() {
    // 檢查資料庫架構中的時間戳欄位類型 - CUTOVER-001 診斷

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

    // 檢查 users 表的時間戳欄位類型
    let result = sqlx::query(
        r#"
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name = 'users'
        AND column_name IN ('created_at', 'updated_at')
        ORDER BY column_name
        "#
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            println!("Users 表時間戳欄位類型:");
            for row in rows {
                let column_name: String = row.get("column_name");
                let data_type: String = row.get("data_type");
                println!("  {}: {}", column_name, data_type);

                if data_type == "timestamp" {
                    println!("    ⚠️  發現問題: 使用 TIMESTAMP (不帶時區)");
                } else if data_type == "timestamptz" {
                    println!("    ✅ 正確: 使用 TIMESTAMPTZ (帶時區)");
                }
            }
        }
        Err(e) => {
            println!("無法查詢資料庫架構: {}", e);
        }
    }

    // 檢查 transactions 表的時間戳欄位類型
    let result = sqlx::query(
        r#"
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name = 'transactions'
        AND column_name = 'created_at'
        "#
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(row) => {
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            println!("Transactions 表時間戳欄位類型:");
            println!("  {}: {}", column_name, data_type);

            if data_type == "timestamp" {
                println!("    ⚠️  發現問題: 使用 TIMESTAMP (不帶時區)");
            } else if data_type == "timestamptz" {
                println!("    ✅ 正確: 使用 TIMESTAMPTZ (帶時區)");
            }
        }
        Err(e) => {
            println!("無法查詢 transactions 表架構: {}", e);
        }
    }
}