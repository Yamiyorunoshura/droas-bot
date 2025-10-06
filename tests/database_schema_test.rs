use sqlx::{PgPool, Row};
use bigdecimal::BigDecimal;
use std::str::FromStr;

// 資料表創建測試
#[sqlx::test]
async fn test_users_table_creation(pool: PgPool) -> Result<(), sqlx::Error> {
    // 檢查 users 表是否存在
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await?;

    let table_exists: bool = result.get(0);
    assert!(table_exists, "users table should exist");

    Ok(())
}

#[sqlx::test]
async fn test_transactions_table_creation(pool: PgPool) -> Result<(), sqlx::Error> {
    // 檢查 transactions 表是否存在
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'transactions'
        )"
    )
    .fetch_one(&pool)
    .await?;

    let table_exists: bool = result.get(0);
    assert!(table_exists, "transactions table should exist");

    Ok(())
}

// 資料表結構測試
#[sqlx::test]
async fn test_users_table_structure(pool: PgPool) -> Result<(), sqlx::Error> {
    // 檢查 users 表欄位
    let result = sqlx::query(
        "SELECT column_name, data_type, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_name = 'users'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await?;

    // 驗證必要欄位存在
    let columns: Vec<String> = result.iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect();

    assert!(columns.contains(&"discord_user_id".to_string()), "Missing discord_user_id column");
    assert!(columns.contains(&"username".to_string()), "Missing username column");
    assert!(columns.contains(&"balance".to_string()), "Missing balance column");
    assert!(columns.contains(&"created_at".to_string()), "Missing created_at column");
    assert!(columns.contains(&"updated_at".to_string()), "Missing updated_at column");

    Ok(())
}

#[sqlx::test]
async fn test_transactions_table_structure(pool: PgPool) -> Result<(), sqlx::Error> {
    // 檢查 transactions 表欄位
    let result = sqlx::query(
        "SELECT column_name, data_type, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_name = 'transactions'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await?;

    // 驗證必要欄位存在
    let columns: Vec<String> = result.iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect();

    assert!(columns.contains(&"id".to_string()), "Missing id column");
    assert!(columns.contains(&"from_user_id".to_string()), "Missing from_user_id column");
    assert!(columns.contains(&"to_user_id".to_string()), "Missing to_user_id column");
    assert!(columns.contains(&"amount".to_string()), "Missing amount column");
    assert!(columns.contains(&"transaction_type".to_string()), "Missing transaction_type column");
    assert!(columns.contains(&"created_at".to_string()), "Missing created_at column");

    Ok(())
}

// 外鍵約束測試
#[sqlx::test]
async fn test_foreign_key_constraints(pool: PgPool) -> Result<(), sqlx::Error> {
    // 插入測試用戶
    let user_id: i64 = 12345;
    sqlx::query(
        "INSERT INTO users (discord_user_id, username, balance)
         VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind("testuser")
    .bind(BigDecimal::from_str("1000.00").unwrap()) // 1000.00
    .execute(&pool)
    .await?;

    // 測試有效外鍵插入 - 應該成功
    let result = sqlx::query(
        "INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
         VALUES ($1, $1, $2, $3)"
    )
    .bind(user_id)
    .bind(BigDecimal::from_str("100.00").unwrap()) // 100.00
    .bind("transfer")
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Valid foreign key should be accepted");

    // 測試無效外鍵插入 - 應該失敗
    let invalid_user_id = 99999;
    let result = sqlx::query(
        "INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(invalid_user_id)
    .bind(user_id)
    .bind(BigDecimal::from_str("50.00").unwrap()) // 50.00
    .bind("transfer")
    .execute(&pool)
    .await;

    assert!(result.is_err(), "Invalid foreign key should be rejected");

    Ok(())
}

// 事務支援測試
#[sqlx::test]
async fn test_transaction_support(pool: PgPool) -> Result<(), sqlx::Error> {
    // 插入兩個測試用戶
    let user1_id: i64 = 11111;
    let user2_id: i64 = 22222;

    sqlx::query(
        "INSERT INTO users (discord_user_id, username, balance)
         VALUES ($1, $2, $3), ($4, $5, $6)"
    )
    .bind(user1_id)
    .bind("user1")
    .bind(BigDecimal::from_str("1000.00").unwrap()) // 1000.00
    .bind(user2_id)
    .bind("user2")
    .bind(BigDecimal::from_str("500.00").unwrap()) // 500.00
    .execute(&pool)
    .await?;

    // 開始事務
    let mut tx = pool.begin().await?;

    // 執行轉帳操作
    let transfer_amount = BigDecimal::from_str("100.00").unwrap(); // 100.00

    // 扣除用戶1餘額
    sqlx::query(
        "UPDATE users SET balance = balance - $1 WHERE discord_user_id = $2"
    )
    .bind(&transfer_amount)
    .bind(user1_id)
    .execute(&mut *tx)
    .await?;

    // 增加用戶2餘額
    sqlx::query(
        "UPDATE users SET balance = balance + $1 WHERE discord_user_id = $2"
    )
    .bind(&transfer_amount)
    .bind(user2_id)
    .execute(&mut *tx)
    .await?;

    // 記錄交易
    sqlx::query(
        "INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(user1_id)
    .bind(user2_id)
    .bind(&transfer_amount)
    .bind("transfer")
    .execute(&mut *tx)
    .await?;

    // 提交事務
    tx.commit().await?;

    // 驗證事務成功
    let user1_balance = sqlx::query(
        "SELECT balance FROM users WHERE discord_user_id = $1"
    )
    .bind(user1_id)
    .fetch_one(&pool)
    .await?
    .get::<BigDecimal, _>("balance");

    let user2_balance = sqlx::query(
        "SELECT balance FROM users WHERE discord_user_id = $1"
    )
    .bind(user2_id)
    .fetch_one(&pool)
    .await?
    .get::<BigDecimal, _>("balance");

    assert_eq!(user1_balance, BigDecimal::from_str("900.00").unwrap()); // 900.00
    assert_eq!(user2_balance, BigDecimal::from_str("600.00").unwrap()); // 600.00

    Ok(())
}

#[sqlx::test]
async fn test_transaction_rollback(pool: PgPool) -> Result<(), sqlx::Error> {
    // 插入測試用戶
    let user_id: i64 = 33333;
    let initial_balance = BigDecimal::from_str("1000.00").unwrap(); // 1000.00

    sqlx::query(
        "INSERT INTO users (discord_user_id, username, balance)
         VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind("testuser")
    .bind(&initial_balance)
    .execute(&pool)
    .await?;

    // 開始事務但故意回滾
    {
        let mut tx = pool.begin().await?;

        // 執行一些更新
        sqlx::query(
            "UPDATE users SET balance = balance - $1 WHERE discord_user_id = $2"
        )
        .bind(BigDecimal::from_str("100.00").unwrap()) // 100.00
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // 故意回滾事務
        tx.rollback().await?;
    }

    // 驗證回滾後餘額不變
    let final_balance = sqlx::query(
        "SELECT balance FROM users WHERE discord_user_id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?
    .get::<BigDecimal, _>("balance");

    assert_eq!(final_balance, BigDecimal::from_str("1000.00").unwrap());

    Ok(())
}

// 索引效能測試
#[sqlx::test]
async fn test_query_performance(pool: PgPool) -> Result<(), sqlx::Error> {
    // 插入大量測試資料
    let mut user_ids = Vec::new();
    for i in 1..=1000 {
        let user_id = 40000 + i;
        user_ids.push(user_id);

        sqlx::query(
            "INSERT INTO users (discord_user_id, username, balance)
             VALUES ($1, $2, $3)"
        )
        .bind(user_id)
        .bind(&format!("user{}", i))
        .bind(BigDecimal::from_str("1000.00").unwrap()) // 1000.00
        .execute(&pool)
        .await?;
    }

    // 測試餘額查詢效能
    let start = std::time::Instant::now();

    for user_id in &user_ids {
        sqlx::query(
            "SELECT balance FROM users WHERE discord_user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await?;
    }

    let duration = start.elapsed();
    let avg_query_time = duration / user_ids.len() as u32;

    // 驗證平均查詢時間在 500ms 以內
    assert!(avg_query_time.as_millis() < 500, "Query should complete within 500ms");

    Ok(())
}