//! Audit Logger Module
//!
//! 提供結構化的審計日誌記錄，追蹤所有防護動作和敏感操作。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;

/// 審計事件類型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    ProtectionAction,
    ConfigurationChange,
    AdminCommand,
    SystemError,
    SecurityAlert,
}

/// 審計日誌項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: String,
    pub target: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub guild_id: String,
    pub success: bool,
    pub ip_address: Option<String>,
}

/// 審計日誌器
pub struct AuditLogger {
    entries: Arc<RwLock<VecDeque<AuditEntry>>>,
    max_entries: usize,
    persist_path: Option<String>,
}

impl AuditLogger {
    /// 創建新的審計日誌器
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::new())),
            max_entries,
            persist_path: None,
        }
    }
    
    /// 設置持久化路徑
    pub fn with_persist_path(mut self, path: String) -> Self {
        self.persist_path = Some(path);
        self
    }
    
    /// 記錄審計事件
    pub async fn log(&self, entry: AuditEntry) {
        let mut entries = self.entries.write().await;
        
        // 添加新項目
        entries.push_back(entry.clone());
        
        // 限制大小
        while entries.len() > self.max_entries {
            entries.pop_front();
        }
        
        // 持久化到文件
        if let Some(path) = &self.persist_path {
            self.persist_to_file(&entry, path).await;
        }
        
        // 記錄到系統日誌
        tracing::info!(
            "AUDIT: {} - {} by {} on {:?}",
            entry.event_type.as_str(),
            entry.action,
            entry.actor,
            entry.target
        );
    }
    
    /// 查詢審計日誌
    pub async fn query(
        &self,
        filter: AuditFilter,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        
        entries
            .iter()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .collect()
    }
    
    /// 持久化到文件
    async fn persist_to_file(&self, entry: &AuditEntry, path: &str) {
        use tokio::io::AsyncWriteExt;
        
        let json = serde_json::to_string(entry).unwrap();
        let line = format!("{}\n", json);
        
        if let Ok(mut file) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
        {
            let _ = file.write_all(line.as_bytes()).await;
        }
    }
}

impl AuditEventType {
    fn as_str(&self) -> &str {
        match self {
            Self::ProtectionAction => "PROTECTION",
            Self::ConfigurationChange => "CONFIG",
            Self::AdminCommand => "ADMIN",
            Self::SystemError => "ERROR",
            Self::SecurityAlert => "SECURITY",
        }
    }
}

/// 審計日誌過濾器
pub struct AuditFilter {
    pub event_type: Option<AuditEventType>,
    pub actor: Option<String>,
    pub guild_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl AuditFilter {
    pub fn new() -> Self {
        Self {
            event_type: None,
            actor: None,
            guild_id: None,
            start_time: None,
            end_time: None,
        }
    }
    
    fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(ref event_type) = self.event_type {
            if !matches!(&entry.event_type, event_type) {
                return false;
            }
        }
        
        if let Some(ref actor) = self.actor {
            if entry.actor != *actor {
                return false;
            }
        }
        
        if let Some(ref guild_id) = self.guild_id {
            if entry.guild_id != *guild_id {
                return false;
            }
        }
        
        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }
        
        true
    }
}
