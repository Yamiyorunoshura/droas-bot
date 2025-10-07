use sqlx::{PgPool, Row};
use bigdecimal::BigDecimal;
use crate::error::{DiscordError, Result};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: i64,
    pub from_user_id: Option<i64>,
    pub to_user_id: Option<i64>,
    pub amount: BigDecimal,
    pub transaction_type: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<Value>,
}

#[derive(Debug)]
pub struct CreateTransactionRequest {
    pub from_user_id: Option<i64>,
    pub to_user_id: Option<i64>,
    pub amount: BigDecimal,
    pub transaction_type: String,
    pub metadata: Option<Value>,
}

#[async_trait]
pub trait TransactionRepositoryTrait {
    /// 創建新的交易記錄
    async fn create_transaction(&self, request: CreateTransactionRequest) -> Result<Transaction>;

    /// 獲取用戶的交易歷史
    async fn get_user_transactions(&self, user_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Transaction>>;

    /// 創建管理員審計記錄
    async fn create_admin_audit(&self, request: CreateTransactionRequest) -> Result<Transaction>;

    /// 查詢管理員審計記錄
    async fn query_admin_audit(
        &self,
        admin_id: Option<i64>,
        operation_type: Option<&str>,
        target_user_id: Option<i64>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>
    ) -> Result<Vec<Transaction>>;
}

pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 創建新的交易記錄
///
/// # Arguments
///
/// * `request` - 包含交易詳情的請求結構體
///
/// # Returns
///
/// 返回創建成功的 `Transaction` 物件，包含交易 ID 和時間戳
///
/// # Errors
///
/// 當資料庫連接失敗或插入操作失敗時返回錯誤
///
/// # Example
///
/// ```rust
/// let request = CreateTransactionRequest {
///     from_user_id: Some(12345),
///     to_user_id: Some(67890),
///     amount: BigDecimal::from_str("100.00").unwrap(),
///     transaction_type: "transfer".to_string(),
/// };
/// let transaction = repository.create_transaction(request).await?;
/// ```
    pub async fn create_transaction(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, from_user_id, to_user_id, amount, transaction_type, created_at
            "#
        )
        .bind(request.from_user_id)
        .bind(request.to_user_id)
        .bind(&request.amount)
        .bind(&request.transaction_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(Transaction {
            id: row.get("id"),
            from_user_id: row.get("from_user_id"),
            to_user_id: row.get("to_user_id"),
            amount: row.get("amount"),
            transaction_type: row.get("transaction_type"),
            created_at: row.get("created_at"),
            metadata: None, // 簡化查詢不包含 metadata
        })
    }

    /// 執行原子轉帳事務
///
/// 此方法實現了完整的 ACID 事務，確保轉帳操作的原子性和一致性。
/// 會檢查發送者餘額是否足夠，如果不足則返回錯誤。
///
/// # Arguments
///
/// * `from_user_id` - 發送者的 Discord 用戶 ID
/// * `to_user_id` - 接收者的 Discord 用戶 ID
/// * `amount` - 轉帳金額
///
/// # Returns
///
/// 返回創建的轉帳交易記錄
///
/// # Errors
///
/// * `InsufficientBalance` - 當發送者餘額不足時
/// * `DatabaseQueryError` - 當資料庫操作失敗時
/// * `TransactionError` - 當事務提交失敗時
///
/// # Example
///
/// ```rust
/// let amount = BigDecimal::from_str("500.00").unwrap();
/// let transaction = repository.execute_transfer(12345, 67890, &amount).await?;
/// println!("轉帳成功，交易 ID: {}", transaction.id);
/// ```
    pub async fn execute_transfer(
        &self,
        from_user_id: i64,
        to_user_id: i64,
        amount: &BigDecimal,
    ) -> Result<Transaction> {
        let mut tx = self.pool.begin().await?;

        // 檢查發送者餘額是否足夠
        let sender_balance: BigDecimal = sqlx::query("SELECT balance FROM users WHERE discord_user_id = $1")
            .bind(from_user_id)
            .fetch_one(&mut *tx)
            .await?
            .get("balance");

        if sender_balance < *amount {
            return Err(DiscordError::InsufficientBalance(from_user_id));
        }

        // 扣除發送者餘額
        sqlx::query("UPDATE users SET balance = balance - $1, updated_at = CURRENT_TIMESTAMP WHERE discord_user_id = $2")
            .bind(amount)
            .bind(from_user_id)
            .execute(&mut *tx)
            .await?;

        // 增加接收者餘額
        sqlx::query("UPDATE users SET balance = balance + $1, updated_at = CURRENT_TIMESTAMP WHERE discord_user_id = $2")
            .bind(amount)
            .bind(to_user_id)
            .execute(&mut *tx)
            .await?;

        // 記錄交易
        let row = sqlx::query(
            r#"
            INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, from_user_id, to_user_id, amount, transaction_type, created_at
            "#
        )
        .bind(from_user_id)
        .bind(to_user_id)
        .bind(amount)
        .bind("transfer")
        .fetch_one(&mut *tx)
        .await?;

        // 提交事務
        tx.commit().await?;

        Ok(Transaction {
            id: row.get("id"),
            from_user_id: row.get("from_user_id"),
            to_user_id: row.get("to_user_id"),
            amount: row.get("amount"),
            transaction_type: row.get("transaction_type"),
            created_at: row.get("created_at"),
            metadata: None, // 簡化查詢不包含 metadata
        })
    }

    /// 獲取指定用戶的交易歷史記錄
///
/// 返回用戶作為發送者或接收者的所有交易記錄，按時間倒序排列。
///
/// # Arguments
///
/// * `discord_user_id` - Discord 用戶 ID
/// * `limit` - 返回記錄的最大數量，預設為 50
/// * `offset` - 分頁偏移量，預設為 0
///
/// # Returns
///
/// 返回交易記錄的向量，按創建時間倒序排列
///
/// # Errors
///
/// 當資料庫查詢失敗時返回錯誤
///
/// # Example
///
/// ```rust
/// let transactions = repository.get_user_transactions(12345, Some(20), Some(0)).await?;
/// for transaction in transactions {
///     println!("交易 ID: {}, 金額: {}", transaction.id, transaction.amount);
/// }
/// ```
    pub async fn get_user_transactions(
        &self,
        discord_user_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Transaction>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, amount, transaction_type, created_at
            FROM transactions
            WHERE from_user_id = $1 OR to_user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(discord_user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Transaction {
                id: row.get("id"),
                from_user_id: row.get("from_user_id"),
                to_user_id: row.get("to_user_id"),
                amount: row.get("amount"),
                transaction_type: row.get("transaction_type"),
                created_at: row.get("created_at"),
                metadata: None, // 簡化查詢不包含 metadata
            });
        }

        Ok(transactions)
    }

    /// 根據 ID 獲取交易
    pub async fn get_transaction_by_id(&self, transaction_id: i64) -> Result<Option<Transaction>> {
        let row = sqlx::query(
            "SELECT id, from_user_id, to_user_id, amount, transaction_type, created_at FROM transactions WHERE id = $1"
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Transaction {
                id: row.get("id"),
                from_user_id: row.get("from_user_id"),
                to_user_id: row.get("to_user_id"),
                amount: row.get("amount"),
                transaction_type: row.get("transaction_type"),
                created_at: row.get("created_at"),
                metadata: None, // 簡化查詢不包含 metadata
            })),
            None => Ok(None),
        }
    }

    /// 獲取指定時間範圍內的交易
    pub async fn get_transactions_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<Transaction>> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, amount, transaction_type, created_at
            FROM transactions
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at DESC
            LIMIT $3
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Transaction {
                id: row.get("id"),
                from_user_id: row.get("from_user_id"),
                to_user_id: row.get("to_user_id"),
                amount: row.get("amount"),
                transaction_type: row.get("transaction_type"),
                created_at: row.get("created_at"),
                metadata: None, // 簡化查詢不包含 metadata
            });
        }

        Ok(transactions)
    }
}

// 注意：TransactionRepository 的現有方法已經滿足 trait 要求
// 直接使用 impl 塊來實現 trait，但需要處理名稱衝突
impl TransactionRepository {
    // 為 trait 提供的內部方法
    async fn create_transaction_for_trait(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        // 調用現有實現，使用不同的調用方式
        let row = sqlx::query(
            r#"
            INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, from_user_id, to_user_id, amount, transaction_type, created_at
            "#
        )
        .bind(request.from_user_id)
        .bind(request.to_user_id)
        .bind(&request.amount)
        .bind(&request.transaction_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DiscordError::DatabaseQueryError(e.to_string()))?;

        Ok(Transaction {
            id: row.get("id"),
            from_user_id: row.get("from_user_id"),
            to_user_id: row.get("to_user_id"),
            amount: row.get("amount"),
            transaction_type: row.get("transaction_type"),
            created_at: row.get("created_at"),
            metadata: None, // 簡化查詢不包含 metadata
        })
    }

    async fn get_user_transactions_for_trait(&self, user_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Transaction>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, amount, transaction_type, created_at
            FROM transactions
            WHERE from_user_id = $1 OR to_user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DiscordError::DatabaseQueryError(e.to_string()))?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Transaction {
                id: row.get("id"),
                from_user_id: row.get("from_user_id"),
                to_user_id: row.get("to_user_id"),
                amount: row.get("amount"),
                transaction_type: row.get("transaction_type"),
                created_at: row.get("created_at"),
                metadata: None, // 現有查詢不包含 metadata
            });
        }

        Ok(transactions)
    }

    /// 創建管理員審計記錄的內部實現
    async fn create_admin_audit_for_trait(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        // 從 metadata 中提取審計相關信息
        let reason = request.metadata
            .as_ref()
            .and_then(|m| m.get("reason"))
            .and_then(|r| r.as_str())
            .unwrap_or("管理員操作");

        let ip_address = request.metadata
            .as_ref()
            .and_then(|m| m.get("ip_address"))
            .and_then(|ip| ip.as_str());

        let user_agent = request.metadata
            .as_ref()
            .and_then(|m| m.get("user_agent"))
            .and_then(|ua| ua.as_str());

        // 插入到 admin_audit 表
        let _row = sqlx::query(
            r#"
            INSERT INTO admin_audit (admin_id, operation_type, target_user_id, amount, reason, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, admin_id, operation_type, target_user_id, amount, reason, timestamp, ip_address, user_agent, created_at
            "#
        )
        .bind(request.from_user_id) // admin_id
        .bind(&request.transaction_type) // operation_type
        .bind(request.to_user_id) // target_user_id
        .bind(&request.amount)
        .bind(reason)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DiscordError::DatabaseQueryError(e.to_string()))?;

        // 同時插入到 transactions 表以保持一致性
        let transaction_row = sqlx::query(
            r#"
            INSERT INTO transactions (from_user_id, to_user_id, amount, transaction_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, from_user_id, to_user_id, amount, transaction_type, created_at
            "#
        )
        .bind(request.from_user_id)
        .bind(request.to_user_id)
        .bind(&request.amount)
        .bind(&request.transaction_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DiscordError::DatabaseQueryError(e.to_string()))?;

        Ok(Transaction {
            id: transaction_row.get("id"),
            from_user_id: transaction_row.get("from_user_id"),
            to_user_id: transaction_row.get("to_user_id"),
            amount: transaction_row.get("amount"),
            transaction_type: transaction_row.get("transaction_type"),
            created_at: transaction_row.get("created_at"),
            metadata: request.metadata,
        })
    }

    /// 查詢管理員審計記錄的內部實現
    async fn query_admin_audit_for_trait(
        &self,
        admin_id: Option<i64>,
        operation_type: Option<&str>,
        target_user_id: Option<i64>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64
    ) -> Result<Vec<Transaction>> {
        // 構建動態查詢
        let mut query = String::from(
            r#"
            SELECT t.id, t.from_user_id, t.to_user_id, t.amount, t.transaction_type, t.created_at,
                   aa.reason, aa.ip_address, aa.user_agent
            FROM transactions t
            INNER JOIN admin_audit aa ON t.id = aa.id
            WHERE 1=1
            "#
        );

        let mut bind_count = 0;
        let mut params = Vec::new();

        if let Some(admin) = admin_id {
            query.push_str(&format!(" AND t.from_user_id = ${}", bind_count + 1));
            params.push(admin.to_string());
            bind_count += 1;
        }

        if let Some(op_type) = operation_type {
            query.push_str(&format!(" AND t.transaction_type = ${}", bind_count + 1));
            params.push(op_type.to_string());
            bind_count += 1;
        }

        if let Some(target) = target_user_id {
            query.push_str(&format!(" AND t.to_user_id = ${}", bind_count + 1));
            params.push(target.to_string());
            bind_count += 1;
        }

        if let Some(start) = start_time {
            query.push_str(&format!(" AND t.created_at >= ${}", bind_count + 1));
            params.push(start.to_rfc3339());
            bind_count += 1;
        }

        if let Some(end) = end_time {
            query.push_str(&format!(" AND t.created_at <= ${}", bind_count + 1));
            params.push(end.to_rfc3339());
            bind_count += 1;
        }

        query.push_str(&format!(" ORDER BY t.created_at DESC LIMIT ${} OFFSET ${}", bind_count + 1, bind_count + 2));
        params.push(limit.to_string());
        params.push(offset.to_string());

        // 執行查詢（這裡簡化實現，實際應該使用 sqlx 的動態查詢功能）
        // 為了簡化，先使用固定的查詢
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.from_user_id, t.to_user_id, t.amount, t.transaction_type, t.created_at,
                   aa.reason, aa.ip_address, aa.user_agent
            FROM transactions t
            INNER JOIN admin_audit aa ON t.id = aa.id
            WHERE ($1::bigint IS NULL OR t.from_user_id = $1)
              AND ($2::text IS NULL OR t.transaction_type = $2)
              AND ($3::bigint IS NULL OR t.to_user_id = $3)
              AND ($4::timestamptz IS NULL OR t.created_at >= $4)
              AND ($5::timestamptz IS NULL OR t.created_at <= $5)
            ORDER BY t.created_at DESC
            LIMIT $6 OFFSET $7
            "#
        )
        .bind(admin_id)
        .bind(operation_type)
        .bind(target_user_id)
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DiscordError::DatabaseQueryError(e.to_string()))?;

        let mut transactions = Vec::new();
        for row in rows {
            let metadata = Value::Object(serde_json::json!({
                "reason": row.get::<Option<String>, _>("reason").unwrap_or_default(),
                "ip_address": row.get::<Option<String>, _>("ip_address"),
                "user_agent": row.get::<Option<String>, _>("user_agent"),
                "audit_type": "admin_operation"
            }).as_object().unwrap().clone());

            transactions.push(Transaction {
                id: row.get("id"),
                from_user_id: row.get("from_user_id"),
                to_user_id: row.get("to_user_id"),
                amount: row.get("amount"),
                transaction_type: row.get("transaction_type"),
                created_at: row.get("created_at"),
                metadata: Some(metadata),
            });
        }

        Ok(transactions)
    }
}

#[async_trait]
impl TransactionRepositoryTrait for TransactionRepository {
    async fn create_transaction(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        self.create_transaction_for_trait(request).await
    }

    async fn get_user_transactions(&self, user_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Transaction>> {
        self.get_user_transactions_for_trait(user_id, limit, offset).await
    }

    async fn create_admin_audit(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        self.create_admin_audit_for_trait(request).await
    }

    async fn query_admin_audit(
        &self,
        admin_id: Option<i64>,
        operation_type: Option<&str>,
        target_user_id: Option<i64>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>
    ) -> Result<Vec<Transaction>> {
        self.query_admin_audit_for_trait(
            admin_id,
            operation_type,
            target_user_id,
            start_time,
            end_time,
            limit.unwrap_or(100),
            offset.unwrap_or(0)
        ).await
    }
}