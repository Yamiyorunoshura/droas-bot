use sqlx::{PgPool, Row};
use bigdecimal::BigDecimal;
use crate::error::{DiscordError, Result};
use chrono::{DateTime, Utc};
use tracing::{info, error, debug};
use async_trait::async_trait;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct User {
    pub discord_user_id: i64,
    pub username: String,
    pub balance: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateUserRequest {
    pub discord_user_id: i64,
    pub username: String,
    pub initial_balance: Option<BigDecimal>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    /// 創建新用戶
    async fn create_user(&self, discord_user_id: i64, username: &str) -> Result<User>;

    /// 根據 Discord 用戶 ID 獲取用戶資訊
    async fn find_by_user_id(&self, user_id: i64) -> Result<Option<User>>;

    /// 更新用戶餘額
    async fn update_balance(&self, user_id: i64, new_balance: &BigDecimal) -> Result<User>;

    /// 檢查用戶是否存在
    async fn user_exists(&self, user_id: i64) -> Result<bool>;
}

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 創建新用戶或更新現有用戶資訊
///
/// 使用 UPSERT 操作：如果用戶不存在則創建，如果已存在則更新用戶名稱和更新時間。
/// 新用戶將獲得預設初始餘額 1000.00。
///
/// # Arguments
///
/// * `request` - 包含用戶資訊的創建請求
///
/// # Returns
///
/// 返回創建或更新後的用戶物件
///
/// # Errors
///
/// 當資料庫操作失敗時返回錯誤
///
/// # Example
///
/// ```rust
/// let request = CreateUserRequest {
///     discord_user_id: 12345,
///     username: "TestUser".to_string(),
///     initial_balance: Some(BigDecimal::from_str("1500.00").unwrap()),
/// };
/// let user = repository.create_user(request).await?;
/// println!("用戶 {} 創建成功，餘額: {}", user.username, user.balance);
/// ```
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let initial_balance = request.initial_balance
            .unwrap_or_else(|| BigDecimal::from_str("1000.00").unwrap());

        debug!("Creating user: Discord ID={}, Username={}, Initial Balance={}",
               request.discord_user_id, request.username, initial_balance);

        let row = sqlx::query(
            r#"
            INSERT INTO users (discord_user_id, username, balance)
            VALUES ($1, $2, $3)
            ON CONFLICT (discord_user_id) DO UPDATE SET
                username = EXCLUDED.username,
                updated_at = CURRENT_TIMESTAMP
            RETURNING discord_user_id, username, balance, created_at, updated_at
            "#
        )
        .bind(request.discord_user_id)
        .bind(&request.username)
        .bind(&initial_balance)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create user {}: {}", request.discord_user_id, e);
            DiscordError::DatabaseQueryError(e.to_string())
        })?;

        let user = User {
            discord_user_id: row.get("discord_user_id"),
            username: row.get("username"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        info!("User created/updated successfully: {}", user.discord_user_id);
        Ok(user)
    }

    /// 根據 Discord 用戶 ID 獲取用戶資訊
///
/// # Arguments
///
/// * `discord_user_id` - Discord 用戶 ID
///
/// # Returns
///
/// 返回 `Option<User>`，如果用戶存在則返回用戶物件，否則返回 None
///
/// # Errors
///
/// 當資料庫查詢失敗時返回錯誤
///
/// # Example
///
/// ```rust
/// match repository.get_user_by_discord_id(12345).await? {
///     Some(user) => println!("找到用戶: {}", user.username),
///     None => println!("用戶不存在"),
/// }
/// ```
    pub async fn get_user_by_discord_id(&self, discord_user_id: i64) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT discord_user_id, username, balance, created_at, updated_at FROM users WHERE discord_user_id = $1"
        )
        .bind(discord_user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(User {
                discord_user_id: row.get("discord_user_id"),
                username: row.get("username"),
                balance: row.get("balance"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    /// 更新用戶餘額
    pub async fn update_balance(&self, discord_user_id: i64, new_balance: &BigDecimal) -> Result<User> {
        let row = sqlx::query(
            r#"
            UPDATE users SET balance = $1, updated_at = CURRENT_TIMESTAMP
            WHERE discord_user_id = $2
            RETURNING discord_user_id, username, balance, created_at, updated_at
            "#
        )
        .bind(new_balance)
        .bind(discord_user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            discord_user_id: row.get("discord_user_id"),
            username: row.get("username"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// 檢查用戶是否存在
    pub async fn user_exists(&self, discord_user_id: i64) -> Result<bool> {
        let result = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE discord_user_id = $1)"
        )
        .bind(discord_user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    /// 獲取用戶餘額
    pub async fn get_balance(&self, discord_user_id: i64) -> Result<Option<BigDecimal>> {
        let row = sqlx::query("SELECT balance FROM users WHERE discord_user_id = $1")
            .bind(discord_user_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(row.get("balance"))),
            None => Ok(None),
        }
    }
}

impl UserRepository {
    // 為 trait 提供的內部方法
    async fn create_user_for_trait(&self, discord_user_id: i64, username: &str) -> Result<User> {
        let request = CreateUserRequest {
            discord_user_id,
            username: username.to_string(),
            initial_balance: None, // 使用預設值 1000
        };
        self.create_user(request).await
    }

    async fn find_by_user_id_for_trait(&self, user_id: i64) -> Result<Option<User>> {
        self.get_user_by_discord_id(user_id).await
    }

    async fn update_balance_for_trait(&self, user_id: i64, new_balance: &BigDecimal) -> Result<User> {
        self.update_balance(user_id, new_balance).await
    }

    async fn user_exists_for_trait(&self, user_id: i64) -> Result<bool> {
        self.user_exists(user_id).await
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create_user(&self, discord_user_id: i64, username: &str) -> Result<User> {
        self.create_user_for_trait(discord_user_id, username).await
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Option<User>> {
        self.find_by_user_id_for_trait(user_id).await
    }

    async fn update_balance(&self, user_id: i64, new_balance: &BigDecimal) -> Result<User> {
        self.update_balance_for_trait(user_id, new_balance).await
    }

    async fn user_exists(&self, user_id: i64) -> Result<bool> {
        self.user_exists_for_trait(user_id).await
    }
}