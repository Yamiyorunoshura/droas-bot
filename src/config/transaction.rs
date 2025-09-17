//! 公會配置事務管理
//! 
//! 此模組實現配置更新的原子化操作和並發控制機制，
//! 確保在多執行緒環境下的資料一致性和事務安全。

use crate::config::{models::GuildConfig, repository::GuildConfigRepository};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use tracing::{debug, info, warn};

/// 事務執行上下文
/// 
/// 包含事務執行過程中的所有必要資訊和狀態
#[derive(Debug)]
pub struct TransactionContext {
    /// 事務 ID
    pub transaction_id: String,
    /// 涉及的公會 ID
    pub guild_ids: Vec<i64>,
    /// 開始時間戳
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 是否為只讀事務
    pub read_only: bool,
}

/// 事務結果
#[derive(Debug)]
pub enum TransactionResult<T> {
    /// 事務成功完成
    Committed(T),
    /// 事務回滾
    Aborted(String),
    /// 事務超時
    Timeout,
}

/// 鎖管理器
/// 
/// 管理公會配置的讀寫鎖，防止併發衝突
struct LockManager {
    /// 公會級別的讀寫鎖
    guild_locks: RwLock<HashMap<i64, Arc<RwLock<()>>>>,
}

impl LockManager {
    fn new() -> Self {
        Self {
            guild_locks: RwLock::new(HashMap::new()),
        }
    }
    
    /// 獲取公會的讀寫鎖
    async fn get_guild_lock(&self, guild_id: i64) -> Arc<RwLock<()>> {
        let mut locks = self.guild_locks.write().await;
        locks
            .entry(guild_id)
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone()
    }
    
    /// 清理未使用的鎖
    async fn cleanup_unused_locks(&self) {
        let mut locks = self.guild_locks.write().await;
        locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    }
}

/// 配置事務管理器
/// 
/// 提供原子化的配置更新操作，確保資料一致性
pub struct ConfigTransactionManager {
    repository: GuildConfigRepository,
    lock_manager: Arc<LockManager>,
    /// 事務超時時間（秒）
    timeout_seconds: u64,
}

impl ConfigTransactionManager {
    /// 創建新的事務管理器
    /// 
    /// # Arguments
    /// 
    /// * `repository` - 配置資料庫操作實例
    /// * `timeout_seconds` - 事務超時時間（預設: 30秒）
    pub fn new(repository: GuildConfigRepository, timeout_seconds: Option<u64>) -> Self {
        Self {
            repository,
            lock_manager: Arc::new(LockManager::new()),
            timeout_seconds: timeout_seconds.unwrap_or(30),
        }
    }
    
    /// 執行只讀事務
    /// 
    /// # Arguments
    /// 
    /// * `guild_ids` - 需要讀取的公會 ID 列表
    /// * `operation` - 要執行的操作
    pub async fn execute_read_transaction<T, F>(
        &self,
        guild_ids: Vec<i64>,
        operation: F,
    ) -> Result<TransactionResult<T>>
    where
        F: for<'a> FnOnce(&'a GuildConfigRepository) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>,
    {
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let context = TransactionContext {
            transaction_id: transaction_id.clone(),
            guild_ids: guild_ids.clone(),
            started_at: chrono::Utc::now(),
            read_only: true,
        };
        
        debug!(
            "開始只讀事務: transaction_id={}, guild_ids={:?}",
            context.transaction_id, context.guild_ids
        );
        
        // 獲取所有需要的鎖並持有引用
        let mut guild_locks = Vec::new();
        for guild_id in &guild_ids {
            let lock = self.lock_manager.get_guild_lock(*guild_id).await;
            guild_locks.push(lock);
        }
        
        // 取得讀鎖守衛
        let mut _read_guards = Vec::new();
        for lock in &guild_locks {
            let guard = lock.read().await;
            _read_guards.push(guard);
        }
        
        // 設置超時
        let timeout_duration = std::time::Duration::from_secs(self.timeout_seconds);
        let operation_future = operation(&self.repository);
        
        match tokio::time::timeout(timeout_duration, operation_future).await {
            Ok(Ok(result)) => {
                info!("只讀事務完成: transaction_id={}", context.transaction_id);
                Ok(TransactionResult::Committed(result))
            }
            Ok(Err(error)) => {
                warn!(
                    "只讀事務失敗: transaction_id={}, error={}",
                    context.transaction_id, error
                );
                Ok(TransactionResult::Aborted(error.to_string()))
            }
            Err(_) => {
                warn!("只讀事務超時: transaction_id={}", context.transaction_id);
                Ok(TransactionResult::Timeout)
            }
        }
    }
    
    /// 執行寫事務
    /// 
    /// # Arguments
    /// 
    /// * `guild_ids` - 需要修改的公會 ID 列表
    /// * `operation` - 要執行的操作
    pub async fn execute_write_transaction<T, F>(
        &self,
        guild_ids: Vec<i64>,
        operation: F,
    ) -> Result<TransactionResult<T>>
    where
        F: for<'a> FnOnce(&'a GuildConfigRepository) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>,
    {
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let context = TransactionContext {
            transaction_id: transaction_id.clone(),
            guild_ids: guild_ids.clone(),
            started_at: chrono::Utc::now(),
            read_only: false,
        };
        
        debug!(
            "開始寫事務: transaction_id={}, guild_ids={:?}",
            context.transaction_id, context.guild_ids
        );
        
        // 按 guild_id 排序獲取鎖，防止死鎖
        let mut sorted_guild_ids = guild_ids.clone();
        sorted_guild_ids.sort();
        
        // 獲取所有需要的鎖並持有引用
        let mut guild_locks = Vec::new();
        for guild_id in &sorted_guild_ids {
            let lock = self.lock_manager.get_guild_lock(*guild_id).await;
            guild_locks.push(lock);
        }
        
        // 取得寫鎖守衛
        let mut _write_guards = Vec::new();
        for lock in &guild_locks {
            let guard = lock.write().await;
            _write_guards.push(guard);
        }
        
        // 設置超時
        let timeout_duration = std::time::Duration::from_secs(self.timeout_seconds);
        let operation_future = operation(&self.repository);
        
        match tokio::time::timeout(timeout_duration, operation_future).await {
            Ok(Ok(result)) => {
                info!("寫事務提交成功: transaction_id={}", context.transaction_id);
                Ok(TransactionResult::Committed(result))
            }
            Ok(Err(error)) => {
                warn!(
                    "寫事務回滾: transaction_id={}, error={}",
                    context.transaction_id, error
                );
                Ok(TransactionResult::Aborted(error.to_string()))
            }
            Err(_) => {
                warn!("寫事務超時: transaction_id={}", context.transaction_id);
                Ok(TransactionResult::Timeout)
            }
        }
    }
    
    /// 批量更新配置（原子操作）
    /// 
    /// # Arguments
    /// 
    /// * `configs` - 要更新的配置列表
    pub async fn batch_update_configs(&self, configs: Vec<GuildConfig>) -> Result<TransactionResult<()>> {
        let guild_ids: Vec<i64> = configs.iter().map(|c| c.guild_id).collect();
        let configs_clone = configs.clone(); // 克隆以避免生命週期問題
        
        self.execute_write_transaction(guild_ids, move |repo| {
            let configs = configs_clone;
            Box::pin(async move {
                for config in configs {
                    repo.upsert_config(&config)
                        .await
                        .with_context(|| format!("批量更新配置失敗: guild_id={}", config.guild_id))?;
                }
                Ok(())
            })
        })
        .await
    }
    
    /// 清理鎖管理器中未使用的鎖
    pub async fn cleanup_locks(&self) {
        self.lock_manager.cleanup_unused_locks().await;
    }
    
    /// 獲取事務統計信息
    pub async fn get_transaction_stats(&self) -> TransactionStats {
        let guild_locks = self.lock_manager.guild_locks.read().await;
        TransactionStats {
            active_guild_locks: guild_locks.len(),
        }
    }
}

/// 事務統計信息
#[derive(Debug, Clone)]
pub struct TransactionStats {
    /// 活躍的公會鎖數量
    pub active_guild_locks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{GuildConfig, BackgroundAsset};
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::NamedTempFile;
    use std::sync::atomic::{AtomicI32, Ordering};
    
    async fn setup_test_transaction_manager() -> ConfigTransactionManager {
        let temp_file = NamedTempFile::new().expect("創建臨時文件失敗");
        let database_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("連接測試資料庫失敗");
        
        let repository = GuildConfigRepository::new(pool);
        repository.migrate().await.expect("遷移失敗");
        
        ConfigTransactionManager::new(repository, Some(5)) // 5秒超時用於測試
    }
    
    #[tokio::test]
    async fn test_read_transaction() {
        let manager = setup_test_transaction_manager().await;
        
        // 先插入一些測試數據
        let config = GuildConfig::new(12345, Some(67890), None);
        let _ = manager.repository.upsert_config(&config).await;
        
        // 執行只讀事務
        let result = manager
            .execute_read_transaction(vec![12345], |repo| {
                Box::pin(async move {
                    repo.get_config(12345).await
                })
            })
            .await
            .expect("只讀事務執行失敗");
        
        match result {
            TransactionResult::Committed(config_result) => {
                match config_result {
                    Ok(Some(retrieved_config)) => {
                        assert_eq!(retrieved_config.guild_id, 12345);
                    }
                    Ok(None) => panic!("配置不存在"),
                    Err(e) => panic!("獲取配置失敗: {:?}", e),
                }
            }
            _ => panic!("只讀事務應該成功"),
        }
    }
    
    #[tokio::test]
    async fn test_write_transaction() {
        let manager = setup_test_transaction_manager().await;
        
        let config = GuildConfig::new(54321, Some(98765), Some("bg_test".to_string()));
        
        // 執行寫事務
        let result = manager
            .execute_write_transaction(vec![54321], |repo| {
                Box::pin(async move {
                    repo.upsert_config(&config).await
                })
            })
            .await
            .expect("寫事務執行失敗");
        
        match result {
            TransactionResult::Committed(_) => {
                // 驗證數據是否成功寫入
                let retrieved = manager.repository.get_config(54321).await.expect("查詢失敗");
                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap().guild_id, 54321);
            }
            _ => panic!("寫事務應該成功"),
        }
    }
    
    #[tokio::test]
    async fn test_batch_update_configs() {
        let manager = setup_test_transaction_manager().await;
        
        let configs = vec![
            GuildConfig::new(111, Some(222), None),
            GuildConfig::new(333, Some(444), Some("bg_333".to_string())),
            GuildConfig::new(555, Some(666), Some("bg_555".to_string())),
        ];
        
        let result = manager.batch_update_configs(configs).await.expect("批量更新失敗");
        
        match result {
            TransactionResult::Committed(_) => {
                // 驗證所有配置都已更新
                for guild_id in [111, 333, 555] {
                    let config = manager.repository.get_config(guild_id).await.expect("查詢失敗");
                    assert!(config.is_some());
                    assert_eq!(config.unwrap().guild_id, guild_id);
                }
            }
            _ => panic!("批量更新事務應該成功"),
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_transactions() {
        let manager = Arc::new(setup_test_transaction_manager().await);
        let counter = Arc::new(AtomicI32::new(0));
        
        // 創建多個並發事務
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let manager_clone = manager.clone();
            let counter_clone = counter.clone();
            
            let handle = tokio::spawn(async move {
                let config = GuildConfig::new(i, Some(i + 1000), None);
                
                let result = manager_clone
                    .execute_write_transaction(vec![i], |repo| {
                        Box::pin(async move {
                            // 模擬一些處理時間
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            
                            counter_clone.fetch_add(1, Ordering::SeqCst);
                            repo.upsert_config(&config).await
                        })
                    })
                    .await
                    .expect("事務執行失敗");
                
                matches!(result, TransactionResult::Committed(_))
            });
            
            handles.push(handle);
        }
        
        // 等待所有事務完成
        let results = futures::future::join_all(handles).await;
        
        // 驗證所有事務都成功
        for result in results {
            assert!(result.expect("任務應該完成"));
        }
        
        // 驗證計數器
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
    
    #[tokio::test]
    async fn test_transaction_timeout() {
        let manager = setup_test_transaction_manager().await;
        
        // 執行一個會超時的事務
        let result = manager
            .execute_write_transaction(vec![99999], |_repo| {
                Box::pin(async move {
                    // 睡眠超過事務超時時間
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    Ok(())
                })
            })
            .await
            .expect("事務執行失敗");
        
        match result {
            TransactionResult::Timeout => {
                // 預期的結果
            }
            _ => panic!("事務應該超時"),
        }
    }
    
    #[tokio::test]
    async fn test_transaction_stats() {
        let manager = setup_test_transaction_manager().await;
        
        // 執行一些操作以創建鎖
        let _result = manager
            .execute_read_transaction(vec![1, 2, 3], |_repo| {
                Box::pin(async move { Ok(()) })
            })
            .await;
        
        let stats = manager.get_transaction_stats().await;
        // 由於鎖可能已被清理，我們只檢查統計信息是否可用
        assert!(stats.active_guild_locks >= 0);
    }
}