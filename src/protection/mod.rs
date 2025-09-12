//! # Protection Module
//! 
//! 群組防護功能模組，提供訊息檢測、規則引擎和防護動作執行。
//! 
//! ## 主要功能
//! - 垃圾訊息檢測
//! - 重複訊息識別
//! - 連結安全檢測
//! - 洗版行為檢測
//! - 自動禁言和防護動作執行

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::ProtectionLevel;

pub mod inspector;
pub mod rules_engine;
pub mod pattern_recognition;
pub mod action_executor;

// Re-exports
pub use inspector::{MessageInspector, InspectorConfig};
pub use rules_engine::{RulesEngine, RuleDecision};
pub use pattern_recognition::{PatternRecognizer, SpamScore, SafetyResult};
pub use action_executor::{ActionExecutor};

/// Protection 模組錯誤類型
#[derive(Error, Debug)]
pub enum ProtectionError {
    #[error("訊息檢測失敗: {0}")]
    InspectionFailed(String),
    
    #[error("規則引擎錯誤: {0}")]
    RulesEngineError(String),
    
    #[error("動作執行失敗: {0}")]
    ActionExecutionFailed(String),
    
    #[error("配置錯誤: {0}")]
    ConfigurationError(String),
    
    #[error("權限不足: {0}")]
    InsufficientPermissions(String),
    
    #[error("API 限制: {0}")]
    RateLimited(String),
}

pub type Result<T> = std::result::Result<T, ProtectionError>;

/// Discord 訊息結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub author_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<String>,
    pub embeds: Vec<String>,
    pub mentions: Vec<String>,
}

/// 訊息上下文
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub message: Message,
    pub author_history: Vec<Message>,
    pub channel_recent_messages: Vec<Message>,
    pub author_violation_count: usize,
    pub guild_protection_level: ProtectionLevel,
}

/// 檢測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub message_id: String,
    pub violations: Vec<Violation>,
    pub risk_score: f32,
    pub suggested_actions: Vec<ProtectionAction>,
    pub confidence: f32,
}

/// 違規類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationType {
    Spam,
    Flooding,
    DuplicateMessage,
    UnsafeLink,
    Harassment,
    Other(String),
}

/// 違規記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub violation_type: ViolationType,
    pub severity: Severity,
    pub description: String,
    pub evidence: String,
}

/// 嚴重程度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// 防護動作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtectionAction {
    DeleteMessage {
        message_id: String,
        reason: String,
    },
    Mute {
        user_id: String,
        duration_seconds: u64,
        reason: String,
    },
    Warn {
        user_id: String,
        reason: String,
    },
    Ban {
        user_id: String,
        reason: String,
        delete_message_days: u8,
    },
    Kick {
        user_id: String,
        reason: String,
    },
}

/// Protection Service - 主要服務介面
#[async_trait]
pub trait ProtectionService: Send + Sync {
    /// 檢測訊息
    async fn inspect_message(&self, context: &MessageContext) -> Result<InspectionResult>;
    
    /// 執行防護動作
    async fn execute_action(&self, action: &ProtectionAction) -> Result<()>;
    
    /// 更新防護等級
    async fn update_protection_level(&self, guild_id: &str, level: ProtectionLevel) -> Result<()>;
    
    /// 獲取統計資料
    async fn get_statistics(&self, guild_id: &str) -> Result<ProtectionStatistics>;
}

/// 防護統計資料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionStatistics {
    pub guild_id: String,
    pub total_messages_inspected: u64,
    pub violations_detected: u64,
    pub actions_taken: u64,
    pub false_positives: u64,
    pub current_protection_level: ProtectionLevel,
    pub last_update: DateTime<Utc>,
}

/// Protection Manager - 整合所有防護組件
pub struct ProtectionManager {
    inspector: Arc<dyn MessageInspector>,
    rules_engine: Arc<RwLock<dyn RulesEngine>>,
    pattern_recognizer: Arc<dyn PatternRecognizer>,
    action_executor: Arc<dyn ActionExecutor>,
    statistics: Arc<RwLock<dashmap::DashMap<String, ProtectionStatistics>>>,
}

impl ProtectionManager {
    /// 創建新的 Protection Manager
    pub fn new(
        inspector: Arc<dyn MessageInspector>,
        rules_engine: Arc<RwLock<dyn RulesEngine>>,
        pattern_recognizer: Arc<dyn PatternRecognizer>,
        action_executor: Arc<dyn ActionExecutor>,
    ) -> Self {
        Self {
            inspector,
            rules_engine,
            pattern_recognizer,
            action_executor,
            statistics: Arc::new(RwLock::new(dashmap::DashMap::new())),
        }
    }
    
    /// 初始化防護系統
    pub async fn initialize(&self) -> Result<()> {
        // 初始化各個組件
        tracing::info!("初始化群組防護系統");
        Ok(())
    }
    
    /// 關閉防護系統
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("關閉群組防護系統");
        Ok(())
    }
}

#[async_trait]
impl ProtectionService for ProtectionManager {
    async fn inspect_message(&self, context: &MessageContext) -> Result<InspectionResult> {
        // 使用 inspector 進行訊息檢測
        self.inspector.inspect(context).await
    }
    
    async fn execute_action(&self, action: &ProtectionAction) -> Result<()> {
        // 使用 action_executor 執行防護動作
        self.action_executor.execute(action).await
    }
    
    async fn update_protection_level(&self, guild_id: &str, level: ProtectionLevel) -> Result<()> {
        // 更新規則引擎的防護等級
        let mut engine = self.rules_engine.write().await;
        engine.update_protection_level(guild_id, level).await
    }
    
    async fn get_statistics(&self, guild_id: &str) -> Result<ProtectionStatistics> {
        let stats = self.statistics.read().await;
        stats.get(guild_id)
            .map(|entry| entry.clone())
            .ok_or_else(|| ProtectionError::ConfigurationError(
                format!("找不到群組 {} 的統計資料", guild_id)
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
    
    #[test]
    fn test_violation_type_equality() {
        assert_eq!(ViolationType::Spam, ViolationType::Spam);
        assert_ne!(ViolationType::Spam, ViolationType::Flooding);
    }
}
