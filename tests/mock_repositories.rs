// Mock 倉儲服務 - 用於轉帳服務測試
// 解決測試環境依賴外部資料庫的問題

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::str::FromStr;
use droas_bot::database::{user_repository::User, transaction_repository::{Transaction, CreateTransactionRequest}};
use droas_bot::error::{DiscordError, Result};

// Mock Transaction ID 計數器
static mut TRANSACTION_COUNTER: i64 = 1;

/// Mock 用戶資料庫倉儲
/// 提供記憶體中的用戶數據管理，用於測試環境
#[derive(Debug, Clone)]
pub struct MockUserRepository {
    users: Arc<Mutex<HashMap<i64, User>>>,
}

impl MockUserRepository {
    /// 創建新的 Mock 用戶倉儲
    pub fn new() -> Self {
        let mut users = HashMap::new();

        // 預先創建一些測試用戶
        users.insert(123, User {
            discord_user_id: 123,
            username: "Alice".to_string(),
            balance: BigDecimal::from_str("500.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(456, User {
            discord_user_id: 456,
            username: "Bob".to_string(),
            balance: BigDecimal::from_str("200.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(789, User {
            discord_user_id: 789,
            username: "Charlie".to_string(),
            balance: BigDecimal::from_str("50.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(101, User {
            discord_user_id: 101,
            username: "David".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(111, User {
            discord_user_id: 111,
            username: "Eve".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(222, User {
            discord_user_id: 222,
            username: "Frank".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(333, User {
            discord_user_id: 333,
            username: "Grace".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(444, User {
            discord_user_id: 444,
            username: "Hank".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(555, User {
            discord_user_id: 555,
            username: "Ivy".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        users.insert(666, User {
            discord_user_id: 666,
            username: "SelfTester".to_string(),
            balance: BigDecimal::from_str("100.00").unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        Self {
            users: Arc::new(Mutex::new(users)),
        }
    }

    /// 添加測試用戶（用於動態測試場景）
    pub fn add_test_user(&self, user: User) {
        let mut users = self.users.lock().unwrap();
        users.insert(user.discord_user_id, user);
    }

    /// 獲取所有用戶（用於測試驗證）
    pub fn get_all_users(&self) -> HashMap<i64, User> {
        let users = self.users.lock().unwrap();
        users.clone()
    }
}

#[async_trait]
impl droas_bot::database::user_repository::UserRepositoryTrait for MockUserRepository {
    async fn create_user(&self, discord_user_id: i64, username: &str) -> Result<User> {
        let user = User {
            discord_user_id,
            username: username.to_string(),
            balance: BigDecimal::from(1000), // 預設餘額 1000
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut users = self.users.lock().unwrap();
        users.insert(discord_user_id, user.clone());

        Ok(user)
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&user_id).cloned())
    }

    async fn update_balance(&self, user_id: i64, new_balance: &BigDecimal) -> Result<User> {
        let mut users = self.users.lock().unwrap();

        if let Some(user) = users.get_mut(&user_id) {
            user.balance = new_balance.clone();
            user.updated_at = Utc::now();
            Ok(user.clone())
        } else {
            Err(DiscordError::UserNotFound("用戶不存在".to_string()))
        }
    }

    async fn user_exists(&self, user_id: i64) -> Result<bool> {
        let users = self.users.lock().unwrap();
        Ok(users.contains_key(&user_id))
    }
}

/// Mock 交易資料庫倉儲
/// 提供記憶體中的交易數據管理，用於測試環境
#[derive(Debug, Clone)]
pub struct MockTransactionRepository {
    transactions: Arc<Mutex<Vec<Transaction>>>,
}

impl MockTransactionRepository {
    /// 創建新的 Mock 交易倉儲
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 獲取所有交易（用於測試驗證）
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions.clone()
    }

    /// 清空所有交易（用於測試隔離）
    pub fn clear_transactions(&self) {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.clear();
        unsafe {
            TRANSACTION_COUNTER = 1;
        }
    }
}

#[async_trait]
impl droas_bot::database::transaction_repository::TransactionRepositoryTrait for MockTransactionRepository {
    async fn create_transaction(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        let transaction = Transaction {
            id: unsafe {
                let id = TRANSACTION_COUNTER;
                TRANSACTION_COUNTER += 1;
                id
            },
            from_user_id: request.from_user_id,
            to_user_id: request.to_user_id,
            amount: request.amount,
            transaction_type: request.transaction_type,
            created_at: Utc::now(),
            metadata: request.metadata,
        };

        let mut transactions = self.transactions.lock().unwrap();
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    async fn get_user_transactions(&self, user_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Transaction>> {
        let transactions = self.transactions.lock().unwrap();

        let mut filtered: Vec<Transaction> = transactions
            .iter()
            .filter(|tx| tx.from_user_id == Some(user_id) || tx.to_user_id == Some(user_id))
            .cloned()
            .collect();

        // 排序（最新的在前）
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // 應用分頁
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(10) as usize;

        let end = std::cmp::min(offset + limit, filtered.len());

        if offset < filtered.len() {
            Ok(filtered[offset..end].to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    async fn create_admin_audit(&self, request: CreateTransactionRequest) -> Result<Transaction> {
        let transaction = Transaction {
            id: unsafe {
                let id = TRANSACTION_COUNTER;
                TRANSACTION_COUNTER += 1;
                id
            },
            from_user_id: request.from_user_id,
            to_user_id: request.to_user_id,
            amount: request.amount,
            transaction_type: request.transaction_type,
            created_at: Utc::now(),
            metadata: request.metadata,
        };

        let mut transactions = self.transactions.lock().unwrap();
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    async fn query_admin_audit(
        &self,
        admin_id: Option<i64>,
        operation_type: Option<&str>,
        target_user_id: Option<i64>,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>
    ) -> Result<Vec<Transaction>> {
        let transactions = self.transactions.lock().unwrap();

        let mut filtered: Vec<Transaction> = transactions
            .iter()
            .filter(|tx| {
                // 過濾管理員ID
                if let Some(admin) = admin_id {
                    if tx.from_user_id != Some(admin) {
                        return false;
                    }
                }

                // 過濾操作類型
                if let Some(op_type) = operation_type {
                    if tx.transaction_type != op_type {
                        return false;
                    }
                }

                // 過濾目標用戶
                if let Some(target) = target_user_id {
                    if tx.to_user_id != Some(target) {
                        return false;
                    }
                }

                // 過濾時間範圍
                if let Some(start) = start_time {
                    if tx.created_at < start {
                        return false;
                    }
                }

                if let Some(end) = end_time {
                    if tx.created_at > end {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 排序（最新的在前）
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // 應用分頁
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(10) as usize;

        let end = std::cmp::min(offset + limit, filtered.len());

        if offset < filtered.len() {
            Ok(filtered[offset..end].to_vec())
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use droas_bot::database::user_repository::UserRepositoryTrait;
    use droas_bot::database::transaction_repository::TransactionRepositoryTrait;

    #[tokio::test]
    async fn test_mock_user_repository() {
        let repo = MockUserRepository::new();

        // 測試查找存在的用戶
        let user = repo.find_by_user_id(123).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "Alice");

        // 測試查找不存在的用戶
        let user = repo.find_by_user_id(999999).await.unwrap();
        assert!(user.is_none());

        // 測試更新餘額
        let new_balance = BigDecimal::from_str("600.00").unwrap();
        let updated_user = repo.update_balance(123, &new_balance).await.unwrap();
        assert_eq!(updated_user.balance, new_balance);

        // 測試用戶存在檢查
        assert!(repo.user_exists(123).await.unwrap());
        assert!(!repo.user_exists(999999).await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_transaction_repository() {
        let repo = MockTransactionRepository::new();

        // 測試創建交易
        let request = CreateTransactionRequest {
            from_user_id: Some(123),
            to_user_id: Some(456),
            amount: BigDecimal::from_str("100.00").unwrap(),
            transaction_type: "transfer".to_string(),
            metadata: None,
        };

        let transaction = repo.create_transaction(request).await.unwrap();
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.from_user_id, Some(123));
        assert_eq!(transaction.to_user_id, Some(456));

        // 測試查詢用戶交易
        let transactions = repo.get_user_transactions(123, Some(10), Some(0)).await.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, 1);
    }
}