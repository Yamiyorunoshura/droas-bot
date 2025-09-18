use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{debug, error, info};

/// 遷移版本號
pub const MIGRATION_VERSION_001: &str = "001_initial_schema";
pub const MIGRATION_VERSION_002: &str = "002_add_indexes";

/// 遷移記錄
#[derive(Debug)]
pub struct Migration {
    pub version: &'static str,
    pub description: &'static str,
    pub sql: &'static str,
}

/// 所有遷移定義
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: MIGRATION_VERSION_001,
        description: "創建初始資料庫結構 - guild_config 和 background_asset 表",
        sql: r#"
            -- Guild配置表
            CREATE TABLE IF NOT EXISTS guild_config (
                guild_id TEXT PRIMARY KEY NOT NULL,
                welcome_channel_id TEXT NOT NULL,
                background_ref TEXT,
                updated_at TEXT NOT NULL
            );

            -- 背景資源表
            CREATE TABLE IF NOT EXISTS background_asset (
                asset_id TEXT PRIMARY KEY NOT NULL,
                file_path TEXT NOT NULL,
                media_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
        "#,
    },
    Migration {
        version: MIGRATION_VERSION_002,
        description: "添加索引以優化查詢性能",
        sql: r#"
            -- 為guild_config表添加索引
            CREATE INDEX IF NOT EXISTS idx_guild_config_updated_at ON guild_config(updated_at);
            CREATE INDEX IF NOT EXISTS idx_guild_config_background_ref ON guild_config(background_ref) WHERE background_ref IS NOT NULL;
            
            -- 為background_asset表添加索引
            CREATE INDEX IF NOT EXISTS idx_background_asset_created_at ON background_asset(created_at);
            CREATE INDEX IF NOT EXISTS idx_background_asset_media_type ON background_asset(media_type);
            CREATE INDEX IF NOT EXISTS idx_background_asset_file_size ON background_asset(file_size);
        "#,
    },
];

/// 檢查遷移是否已應用
pub async fn is_migration_applied(pool: &SqlitePool, version: &str) -> Result<bool> {
    debug!("檢查遷移狀態: version={}", version);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_one(pool)
        .await?;

    let applied = count > 0;
    debug!("遷移狀態: version={}, applied={}", version, applied);
    Ok(applied)
}

/// 記錄遷移已應用
pub async fn record_migration(pool: &SqlitePool, migration: &Migration) -> Result<()> {
    debug!("記錄遷移: version={}", migration.version);

    sqlx::query(
        r#"
        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(migration.version)
    .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
    .bind(migration.description)
    .execute(pool)
    .await?;

    debug!("遷移記錄成功: version={}", migration.version);
    Ok(())
}

/// 應用單個遷移
pub async fn apply_migration(pool: &SqlitePool, migration: &Migration) -> Result<()> {
    info!(
        "應用遷移: {} - {}",
        migration.version, migration.description
    );

    // 開始事務
    let mut tx = pool.begin().await?;

    // 執行遷移SQL
    for statement in migration.sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            debug!("執行SQL: {}", statement);
            sqlx::query(statement).execute(&mut *tx).await?;
        }
    }

    // 記錄遷移
    sqlx::query(
        r#"
        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(migration.version)
    .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
    .bind(migration.description)
    .execute(&mut *tx)
    .await?;

    // 提交事務
    tx.commit().await?;

    info!("遷移應用成功: {}", migration.version);
    Ok(())
}

/// 應用所有待執行的遷移
pub async fn apply_migrations(pool: &SqlitePool) -> Result<()> {
    info!("開始應用資料庫遷移");

    let mut applied_count = 0;

    for migration in MIGRATIONS {
        if !is_migration_applied(pool, migration.version).await? {
            apply_migration(pool, migration).await?;
            applied_count += 1;
        } else {
            debug!("跳過已應用的遷移: {}", migration.version);
        }
    }

    info!("遷移應用完成，共應用 {} 個遷移", applied_count);
    Ok(())
}

/// 獲取已應用的遷移列表
pub async fn get_applied_migrations(pool: &SqlitePool) -> Result<Vec<(String, String)>> {
    debug!("查詢已應用的遷移列表");

    let migrations: Vec<(String, String)> =
        sqlx::query_as("SELECT version, applied_at FROM schema_migrations ORDER BY applied_at")
            .fetch_all(pool)
            .await?;

    debug!("找到 {} 個已應用的遷移", migrations.len());
    Ok(migrations)
}

/// 回滾遷移（僅支持最後一個遷移）
pub async fn rollback_last_migration(pool: &SqlitePool) -> Result<Option<String>> {
    info!("開始回滾最後一個遷移");

    // 獲取最後應用的遷移
    let last_migration: Option<(String,)> =
        sqlx::query_as("SELECT version FROM schema_migrations ORDER BY applied_at DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;

    if let Some((version,)) = last_migration {
        info!("回滾遷移: {}", version);

        // 開始事務
        let mut tx = pool.begin().await?;

        // 根據版本執行特定的回滾操作
        match version.as_str() {
            MIGRATION_VERSION_002 => {
                // 回滾索引創建
                sqlx::query("DROP INDEX IF EXISTS idx_guild_config_updated_at")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DROP INDEX IF EXISTS idx_guild_config_background_ref")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DROP INDEX IF EXISTS idx_background_asset_created_at")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DROP INDEX IF EXISTS idx_background_asset_media_type")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DROP INDEX IF EXISTS idx_background_asset_file_size")
                    .execute(&mut *tx)
                    .await?;
            }
            MIGRATION_VERSION_001 => {
                // 回滾表創建
                sqlx::query("DROP TABLE IF EXISTS background_asset")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DROP TABLE IF EXISTS guild_config")
                    .execute(&mut *tx)
                    .await?;
            }
            _ => {
                error!("不支持回滾未知的遷移版本: {}", version);
                return Err(anyhow::anyhow!("不支持回滾未知的遷移版本: {}", version));
            }
        }

        // 從遷移記錄中刪除
        sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
            .bind(&version)
            .execute(&mut *tx)
            .await?;

        // 提交事務
        tx.commit().await?;

        info!("遷移回滾成功: {}", version);
        Ok(Some(version))
    } else {
        info!("沒有可回滾的遷移");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> (DatabaseManager, NamedTempFile) {
        let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let db = DatabaseManager::new(&database_url)
            .await
            .expect("無法創建測試資料庫");
        (db, temp_file)
    }

    #[tokio::test]
    async fn test_migrations_apply() {
        let (db, _temp_file) = create_test_database().await;
        let pool = db.pool();

        // 創建 migrations 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("無法創建 migrations 表");

        // 測試應用遷移
        let result = apply_migrations(pool).await;
        assert!(result.is_ok());

        // 驗證所有遷移都已應用
        for migration in MIGRATIONS {
            let applied = is_migration_applied(pool, migration.version).await.unwrap();
            assert!(applied, "遷移 {} 未被應用", migration.version);
        }

        // 驗證表已創建
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('guild_config', 'background_asset')"
        )
        .fetch_all(pool)
        .await
        .expect("無法查詢表");

        assert_eq!(tables.len(), 2);
    }

    #[tokio::test]
    async fn test_migration_idempotency() {
        let (db, _temp_file) = create_test_database().await;
        let pool = db.pool();

        // 創建 migrations 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("無法創建 migrations 表");

        // 第一次應用
        let result1 = apply_migrations(pool).await;
        assert!(result1.is_ok());

        // 第二次應用（應該跳過）
        let result2 = apply_migrations(pool).await;
        assert!(result2.is_ok());

        // 驗證遷移記錄沒有重複
        let applied = get_applied_migrations(pool).await.unwrap();
        assert_eq!(applied.len(), MIGRATIONS.len());
    }

    #[tokio::test]
    async fn test_rollback_migration() {
        let (db, _temp_file) = create_test_database().await;
        let pool = db.pool();

        // 創建 migrations 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("無法創建 migrations 表");

        // 應用遷移
        apply_migrations(pool).await.expect("無法應用遷移");

        // 記錄應用前的遷移數量
        let before_count = get_applied_migrations(pool).await.unwrap().len();

        // 回滾最後一個遷移
        let rolled_back = rollback_last_migration(pool).await.unwrap();
        assert!(rolled_back.is_some());
        assert_eq!(rolled_back.unwrap(), MIGRATION_VERSION_002);

        // 驗證遷移記錄減少
        let after_count = get_applied_migrations(pool).await.unwrap().len();
        assert_eq!(after_count, before_count - 1);

        // 驗證該遷移確實被回滾
        let applied = is_migration_applied(pool, MIGRATION_VERSION_002)
            .await
            .unwrap();
        assert!(!applied);
    }

    #[tokio::test]
    async fn test_get_applied_migrations() {
        let (db, _temp_file) = create_test_database().await;
        let pool = db.pool();

        // 創建 migrations 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("無法創建 migrations 表");

        // 初始狀態應該沒有遷移
        let initial = get_applied_migrations(pool).await.unwrap();
        assert_eq!(initial.len(), 0);

        // 應用遷移
        apply_migrations(pool).await.expect("無法應用遷移");

        // 現在應該有遷移
        let applied = get_applied_migrations(pool).await.unwrap();
        assert_eq!(applied.len(), MIGRATIONS.len());
    }
}
