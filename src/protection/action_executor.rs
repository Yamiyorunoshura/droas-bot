//! Action Executor
//!
//! 使用 Command Pattern 執行防護動作，包括刪除訊息、禁言、警告等。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use crate::protection::{ProtectionAction, Result, ProtectionError};

/// Action Executor trait
#[async_trait]
pub trait ActionExecutor: Send + Sync {
    /// 執行防護動作
    async fn execute(&self, action: &ProtectionAction) -> Result<()>;
    
    /// 批次執行動作
    async fn execute_batch(&self, actions: Vec<ProtectionAction>) -> Result<Vec<ActionResult>>;
    
    /// 撤銷動作（如果支援）
    async fn undo(&self, action_id: &str) -> Result<()>;
    
    /// 獲取執行歷史
    async fn get_history(&self, limit: usize) -> Result<Vec<ActionRecord>>;
}

/// 動作執行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

/// 動作記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: String,
    pub action: ProtectionAction,
    pub result: ActionResult,
    pub can_undo: bool,
}

/// 默認的 Action Executor 實現
pub struct DefaultActionExecutor {
    history: Arc<RwLock<Vec<ActionRecord>>>,
    max_history_size: usize,
}

impl DefaultActionExecutor {
    /// 創建新的 Action Executor
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }
    
    /// 生成動作 ID
    fn generate_action_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    /// 記錄動作
    async fn record_action(&self, action: ProtectionAction, result: ActionResult, can_undo: bool) {
        let mut history = self.history.write().await;
        
        history.push(ActionRecord {
            id: result.action_id.clone(),
            action,
            result,
            can_undo,
        });
        
        // 限制歷史記錄大小
        if history.len() > self.max_history_size {
            let drain_count = history.len() - self.max_history_size;
            history.drain(0..drain_count);
        }
    }
}

#[async_trait]
impl ActionExecutor for DefaultActionExecutor {
    async fn execute(&self, action: &ProtectionAction) -> Result<()> {
        let action_id = Self::generate_action_id();
        let start_time = Utc::now();
        
        // 模擬執行動作（實際應整合 Discord API）
        let (success, error_message, can_undo) = match action {
            ProtectionAction::DeleteMessage { message_id, reason } => {
                tracing::info!("刪除訊息 {}: {}", message_id, reason);
                // TODO: 調用 Discord API 刪除訊息
                (true, None, false)
            },
            ProtectionAction::Mute { user_id, duration_seconds, reason } => {
                tracing::info!("禁言用戶 {} {} 秒: {}", user_id, duration_seconds, reason);
                // TODO: 調用 Discord API 禁言用戶
                (true, None, true)
            },
            ProtectionAction::Warn { user_id, reason } => {
                tracing::info!("警告用戶 {}: {}", user_id, reason);
                // TODO: 發送警告訊息
                (true, None, false)
            },
            ProtectionAction::Ban { user_id, reason, delete_message_days } => {
                tracing::info!("封禁用戶 {} (刪除 {} 天訊息): {}", user_id, delete_message_days, reason);
                // TODO: 調用 Discord API 封禁用戶
                (true, None, true)
            },
            ProtectionAction::Kick { user_id, reason } => {
                tracing::info!("踢出用戶 {}: {}", user_id, reason);
                // TODO: 調用 Discord API 踢出用戶
                (true, None, false)
            },
        };
        
        let result = ActionResult {
            action_id: action_id.clone(),
            success,
            error_message: error_message.clone(),
            executed_at: start_time,
        };
        
        // 記錄動作
        self.record_action(action.clone(), result, can_undo).await;
        
        if success {
            Ok(())
        } else {
            Err(ProtectionError::ActionExecutionFailed(
                error_message.unwrap_or_else(|| "未知錯誤".to_string())
            ))
        }
    }
    
    async fn execute_batch(&self, actions: Vec<ProtectionAction>) -> Result<Vec<ActionResult>> {
        let mut results = Vec::new();
        
        for action in actions {
            let action_id = Self::generate_action_id();
            let start_time = Utc::now();
            
            match self.execute(&action).await {
                Ok(_) => {
                    results.push(ActionResult {
                        action_id,
                        success: true,
                        error_message: None,
                        executed_at: start_time,
                    });
                },
                Err(e) => {
                    results.push(ActionResult {
                        action_id,
                        success: false,
                        error_message: Some(e.to_string()),
                        executed_at: start_time,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    async fn undo(&self, action_id: &str) -> Result<()> {
        let history = self.history.read().await;
        
        let record = history
            .iter()
            .find(|r| r.id == action_id)
            .ok_or_else(|| ProtectionError::ActionExecutionFailed(
                format!("找不到動作記錄: {}", action_id)
            ))?;
        
        if !record.can_undo {
            return Err(ProtectionError::ActionExecutionFailed(
                "此動作無法撤銷".to_string()
            ));
        }
        
        // 模擬撤銷動作
        match &record.action {
            ProtectionAction::Mute { user_id, .. } => {
                tracing::info!("撤銷禁言: {}", user_id);
                // TODO: 調用 Discord API 解除禁言
            },
            ProtectionAction::Ban { user_id, .. } => {
                tracing::info!("撤銷封禁: {}", user_id);
                // TODO: 調用 Discord API 解除封禁
            },
            _ => {
                return Err(ProtectionError::ActionExecutionFailed(
                    "不支援撤銷此類型動作".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    async fn get_history(&self, limit: usize) -> Result<Vec<ActionRecord>> {
        let history = self.history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        
        Ok(history[start..].to_vec())
    }
}

impl Default for DefaultActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_action_execution() {
        let executor = DefaultActionExecutor::new();
        
        let action = ProtectionAction::Warn {
            user_id: "test_user".to_string(),
            reason: "測試警告".to_string(),
        };
        
        let result = executor.execute(&action).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_batch_execution() {
        let executor = DefaultActionExecutor::new();
        
        let actions = vec![
            ProtectionAction::DeleteMessage {
                message_id: "msg_1".to_string(),
                reason: "垃圾訊息".to_string(),
            },
            ProtectionAction::Warn {
                user_id: "user_1".to_string(),
                reason: "違規行為".to_string(),
            },
        ];
        
        let results = executor.execute_batch(actions).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }
    
    #[tokio::test]
    async fn test_history() {
        let executor = DefaultActionExecutor::new();
        
        // 執行一些動作
        for i in 0..5 {
            let action = ProtectionAction::Warn {
                user_id: format!("user_{}", i),
                reason: "測試".to_string(),
            };
            executor.execute(&action).await.unwrap();
        }
        
        let history = executor.get_history(10).await.unwrap();
        assert_eq!(history.len(), 5);
    }
}
