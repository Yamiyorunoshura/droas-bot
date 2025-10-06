pub mod user_repository;
pub mod transaction_repository;
pub mod balance_repository;

pub use user_repository::UserRepository;
pub use transaction_repository::TransactionRepository;
pub use balance_repository::{BalanceRepository, Balance};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use crate::config::DatabaseConfig;
use crate::error::{DiscordError, Result};
use tracing::{info, error, debug, warn};

/// 創建資料庫連接池（性能優化版本）
pub async fn create_user_pool(database_config: &DatabaseConfig) -> Result<PgPool> {
    info!("Creating optimized database connection pool to: {}", database_config.url);
    debug!("Connection pool config - max: {}, min: {}, timeout: {}s",
           database_config.max_connections,
           database_config.min_connections,
           database_config.connection_timeout);

    let pool = PgPoolOptions::new()
        .max_connections(database_config.max_connections)
        .min_connections(database_config.min_connections)
        .acquire_timeout(Duration::from_secs(database_config.connection_timeout))
        .idle_timeout(Duration::from_secs(600)) // 10 分鐘空閑超時
        .max_lifetime(Duration::from_secs(1800)) // 30 分鐘最大連接生命週期
        .test_before_acquire(true) // 獲取前測試連接
        .connect(&database_config.url)
        .await
        .map_err(|e| {
            error!("Failed to create optimized database connection pool: {}", e);
            DiscordError::DatabaseConnectionError(e.to_string())
        })?;

    info!("Optimized database connection pool created successfully");
    Ok(pool)
}

/// 創建測試用資料庫連接池
pub async fn create_test_pool() -> Result<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/droas_test".to_string());

    info!("Creating test database connection pool to: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .map_err(|e| {
            error!("Failed to create test database connection pool: {}", e);
            DiscordError::DatabaseConnectionError(e.to_string())
        })?;

    info!("Test database connection pool created successfully");
    Ok(pool)
}

/// 執行資料庫遷移
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Starting database migrations");

    // 確保資料表存在
    info!("Creating users table if not exists");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            discord_user_id BIGINT PRIMARY KEY,
            username VARCHAR(100) NOT NULL,
            balance DECIMAL(15,2) DEFAULT 1000.00,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to create users table: {}", e);
        DiscordError::MigrationError(format!("Failed to create users table: {}", e))
    })?;

    info!("Creating transactions table if not exists");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id BIGSERIAL PRIMARY KEY,
            from_user_id BIGINT REFERENCES users(discord_user_id),
            to_user_id BIGINT REFERENCES users(discord_user_id),
            amount DECIMAL(15,2) NOT NULL,
            transaction_type VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to create transactions table: {}", e);
        DiscordError::MigrationError(format!("Failed to create transactions table: {}", e))
    })?;

    // 修復時區類型問題 - CUTOVER-001
    info!("Migrating timezone column types from TIMESTAMP to TIMESTAMPTZ");

    // 更新 users 表的時間戳欄位
    let _ = sqlx::query(
        r#"
        ALTER TABLE users
        ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
        ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC'
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("Failed to migrate users table timezone columns (may already be migrated): {}", e);
    });

    // 更新 transactions 表的時間戳欄位
    let _ = sqlx::query(
        r#"
        ALTER TABLE transactions
        ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC'
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("Failed to migrate transactions table timezone columns (may already be migrated): {}", e);
    });

    // 創建索引以優化查詢效能
    info!("Creating indexes for performance optimization");
    let indexes = vec![
        "idx_transactions_from_user_id",
        "idx_transactions_to_user_id",
        "idx_transactions_created_at"
    ];

    for index_name in indexes {
        debug!("Creating index: {}", index_name);
        let query = format!(
            "CREATE INDEX IF NOT EXISTS {} ON transactions({})",
            index_name,
            index_name.strip_prefix("idx_transactions_").unwrap()
        );

        sqlx::query(&query)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("Failed to create index {}: {}", index_name, e);
                DiscordError::MigrationError(format!("Failed to create index {}: {}", index_name, e))
            })?;
    }

    info!("Database migrations completed successfully");
    Ok(())
}

/// 獲取交易 Repository 的工廠函數
pub async fn get_transaction_repository(pool: PgPool) -> TransactionRepository {
    TransactionRepository::new(pool)
}

/// 獲取用戶 Repository 的工廠函數
pub async fn get_user_repository(pool: PgPool) -> UserRepository {
    UserRepository::new(pool)
}