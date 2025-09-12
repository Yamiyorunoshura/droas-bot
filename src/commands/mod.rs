//! Admin Commands Module
//!
//! Discord slash commands 實現，提供管理員控制介面。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::ProtectionLevel;

/// 命令類型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    SetProtectionLevel,
    SetMuteDuration,
    ViewConfig,
    ViewStats,
    AddCustomRule,
    RemoveCustomRule,
    ListViolations,
    UnmuteUser,
    ClearViolations,
}

/// 命令執行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// 管理員命令處理器
#[async_trait]
pub trait AdminCommandHandler: Send + Sync {
    /// 執行命令
    async fn execute(
        &self,
        command: AdminCommand,
        context: CommandContext,
    ) -> Result<CommandResult, CommandError>;
    
    /// 驗證權限
    async fn validate_permission(
        &self,
        user_id: &str,
        command: &AdminCommand,
    ) -> Result<bool, CommandError>;
}

/// 管理員命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCommand {
    pub command_type: CommandType,
    pub args: serde_json::Value,
    pub issuer: String,
}

/// 命令上下文
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub guild_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub user_roles: Vec<String>,
}

/// 命令錯誤
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("權限不足")]
    InsufficientPermissions,
    
    #[error("無效參數: {0}")]
    InvalidArguments(String),
    
    #[error("執行失敗: {0}")]
    ExecutionFailed(String),
    
    #[error("找不到資源: {0}")]
    NotFound(String),
}

/// 默認的管理員命令處理器
pub struct DefaultAdminCommandHandler {
    protection_service: Arc<dyn crate::protection::ProtectionService>,
    audit_logger: Arc<crate::audit::AuditLogger>,
    config: Arc<RwLock<AdminConfig>>,
}

/// 管理員配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub admin_role_ids: Vec<String>,
    pub moderator_role_ids: Vec<String>,
    pub default_mute_duration: u64,
    pub max_mute_duration: u64,
    pub enable_audit: bool,
}

impl DefaultAdminCommandHandler {
    /// 創建新的命令處理器
    pub fn new(
        protection_service: Arc<dyn crate::protection::ProtectionService>,
        audit_logger: Arc<crate::audit::AuditLogger>,
    ) -> Self {
        Self {
            protection_service,
            audit_logger,
            config: Arc::new(RwLock::new(AdminConfig::default())),
        }
    }
}

#[async_trait]
impl AdminCommandHandler for DefaultAdminCommandHandler {
    async fn execute(
        &self,
        command: AdminCommand,
        context: CommandContext,
    ) -> Result<CommandResult, CommandError> {
        // 驗證權限
        if !self.validate_permission(&context.user_id, &command).await? {
            return Err(CommandError::InsufficientPermissions);
        }
        
        // 記錄審計日誌
        let audit_entry = crate::audit::AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: crate::audit::AuditEventType::AdminCommand,
            actor: context.user_id.clone(),
            target: Some(context.guild_id.clone()),
            action: format!("{:?}", command.command_type),
            details: command.args.clone(),
            guild_id: context.guild_id.clone(),
            success: true,
            ip_address: None,
        };
        self.audit_logger.log(audit_entry).await;
        
        // 執行命令
        match command.command_type {
            CommandType::SetProtectionLevel => {
                let level: ProtectionLevel = serde_json::from_value(command.args)
                    .map_err(|e| CommandError::InvalidArguments(e.to_string()))?;
                
                self.protection_service
                    .update_protection_level(&context.guild_id, level)
                    .await
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                
                Ok(CommandResult {
                    success: true,
                    message: format!("防護等級已更新為 {:?}", level),
                    data: None,
                })
            },
            
            CommandType::ViewStats => {
                let stats = self.protection_service
                    .get_statistics(&context.guild_id)
                    .await
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                
                Ok(CommandResult {
                    success: true,
                    message: "統計資料".to_string(),
                    data: Some(serde_json::to_value(stats).unwrap()),
                })
            },
            
            CommandType::SetMuteDuration => {
                let duration: u64 = serde_json::from_value(command.args)
                    .map_err(|e| CommandError::InvalidArguments(e.to_string()))?;
                
                let mut config = self.config.write().await;
                config.default_mute_duration = duration;
                
                Ok(CommandResult {
                    success: true,
                    message: format!("默認禁言時長已設置為 {} 秒", duration),
                    data: None,
                })
            },
            
            _ => Err(CommandError::ExecutionFailed("未實現的命令".to_string())),
        }
    }
    
    async fn validate_permission(
        &self,
        user_id: &str,
        command: &AdminCommand,
    ) -> Result<bool, CommandError> {
        // 簡單的權限驗證（實際應檢查 Discord 角色）
        Ok(true)
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            admin_role_ids: vec![],
            moderator_role_ids: vec![],
            default_mute_duration: 600,
            max_mute_duration: 86400,
            enable_audit: true,
        }
    }
}
