//! Rules Engine
//!
//! 提供可配置的規則引擎，用於評估檢測結果並決定採取的防護動作。
//! 支援三種防護等級：寬鬆、中等、嚴格。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;
use chrono::{DateTime, Utc};
use crate::protection::{
    MessageContext, InspectionResult, ProtectionAction, 
    ProtectionError, Result, Severity, ViolationType
};
use crate::ProtectionLevel;

/// 規則決策結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDecision {
    pub should_take_action: bool,
    pub actions: Vec<ProtectionAction>,
    pub confidence: f32,
    pub reasoning: String,
    pub escalated: bool,
}

/// 規則配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub spam_threshold: f32,
    pub flooding_threshold: f32,
    pub duplicate_threshold: f32,
    pub link_safety_threshold: f32,
    pub escalation_threshold: usize,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            spam_threshold: 0.7,
            flooding_threshold: 5.0,
            duplicate_threshold: 0.8,
            link_safety_threshold: 0.7,
            escalation_threshold: 3,
        }
    }
}

/// 規則動作
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RuleAction {
    Warn,
    Delete,
    Mute,
    Ban,
    None,
}

/// 自定義規則
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub name: String,
    pub pattern: String,
    pub action: RuleAction,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

/// Rules Engine trait
#[async_trait]
pub trait RulesEngine: Send + Sync {
    /// 評估檢測結果並決定動作
    async fn evaluate(
        &self,
        context: &MessageContext,
        inspection: &InspectionResult,
    ) -> Result<RuleDecision>;
    
    /// 重新載入規則配置
    async fn reload_rules(&mut self, config: RuleConfig) -> Result<()>;
    
    /// 更新防護等級
    async fn update_protection_level(
        &mut self,
        guild_id: &str,
        level: ProtectionLevel,
    ) -> Result<()>;
    
    /// 獲取防護等級
    async fn get_protection_level(&self, guild_id: &str) -> Result<ProtectionLevel>;
    
    /// 獲取當前配置
    async fn get_current_config(&self) -> Result<RuleConfig>;
    
    /// 添加自定義規則
    async fn add_custom_rule(
        &mut self,
        name: &str,
        pattern: &str,
        action: RuleAction,
    ) -> Result<()>;
    
    /// 獲取自定義規則
    async fn get_custom_rules(&self) -> Result<HashMap<String, (String, RuleAction)>>;
}

/// 默認的 Rules Engine 實現
pub struct DefaultRulesEngine {
    config: Arc<RwLock<RuleConfig>>,
    protection_levels: Arc<RwLock<HashMap<String, ProtectionLevel>>>,
    custom_rules: Arc<RwLock<HashMap<String, CustomRule>>>,
    violation_history: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl DefaultRulesEngine {
    /// 創建新的 Rules Engine
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(RuleConfig::default())),
            protection_levels: Arc::new(RwLock::new(HashMap::new())),
            custom_rules: Arc::new(RwLock::new(HashMap::new())),
            violation_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 計算動作嚴重程度
    fn calculate_action_severity(&self, severity: &Severity, level: &ProtectionLevel) -> f32 {
        let base_score = match severity {
            Severity::Critical => 1.0,
            Severity::High => 0.75,
            Severity::Medium => 0.5,
            Severity::Low => 0.25,
        };
        
        match level {
            ProtectionLevel::High => base_score * 1.5,
            ProtectionLevel::Medium => base_score,
            ProtectionLevel::Low => base_score * 0.5,
        }
    }
    
    /// 決定採取的動作
    fn determine_actions(
        &self,
        violation: &crate::protection::Violation,
        context: &MessageContext,
        escalated: bool,
    ) -> Vec<ProtectionAction> {
        let mut actions = Vec::new();
        
        match (&violation.severity, &context.guild_protection_level) {
            (Severity::Critical, _) => {
                // Critical 違規：立即刪除並禁言
                actions.push(ProtectionAction::DeleteMessage {
                    message_id: context.message.id.clone(),
                    reason: violation.description.clone(),
                });
                actions.push(ProtectionAction::Mute {
                    user_id: context.message.author_id.clone(),
                    duration_seconds: if escalated { 7200 } else { 3600 },
                    reason: format!("Critical violation: {}", violation.description),
                });
            },
            (Severity::High, ProtectionLevel::High) |
            (Severity::High, ProtectionLevel::Medium) => {
                // High 違規：刪除訊息，可能禁言
                actions.push(ProtectionAction::DeleteMessage {
                    message_id: context.message.id.clone(),
                    reason: violation.description.clone(),
                });
                if escalated || context.guild_protection_level == ProtectionLevel::High {
                    actions.push(ProtectionAction::Mute {
                        user_id: context.message.author_id.clone(),
                        duration_seconds: 600,
                        reason: format!("High severity violation: {}", violation.description),
                    });
                }
            },
            (Severity::High, ProtectionLevel::Low) => {
                // 寬鬆模式下的 High 違規：只警告
                actions.push(ProtectionAction::Warn {
                    user_id: context.message.author_id.clone(),
                    reason: violation.description.clone(),
                });
            },
            (Severity::Medium, ProtectionLevel::High) => {
                // 嚴格模式下的 Medium 違規：刪除並警告
                actions.push(ProtectionAction::DeleteMessage {
                    message_id: context.message.id.clone(),
                    reason: violation.description.clone(),
                });
                actions.push(ProtectionAction::Warn {
                    user_id: context.message.author_id.clone(),
                    reason: violation.description.clone(),
                });
            },
            (Severity::Medium, ProtectionLevel::Medium) => {
                // 中等模式下的 Medium 違規：警告
                actions.push(ProtectionAction::Warn {
                    user_id: context.message.author_id.clone(),
                    reason: violation.description.clone(),
                });
            },
            (Severity::Low, ProtectionLevel::High) => {
                // 嚴格模式下即使是 Low 違規也要警告
                actions.push(ProtectionAction::Warn {
                    user_id: context.message.author_id.clone(),
                    reason: violation.description.clone(),
                });
            },
            _ => {
                // 其他情況不採取動作
            }
        }
        
        // 特殊處理某些違規類型
        match violation.violation_type {
            ViolationType::UnsafeLink => {
                // 不安全連結總是要刪除
                if !actions.iter().any(|a| matches!(a, ProtectionAction::DeleteMessage { .. })) {
                    actions.push(ProtectionAction::DeleteMessage {
                        message_id: context.message.id.clone(),
                        reason: "包含不安全連結".to_string(),
                    });
                }
            },
            ViolationType::Flooding if violation.severity != Severity::Low => {
                // 洗版行為需要禁言
                if !actions.iter().any(|a| matches!(a, ProtectionAction::Mute { .. })) {
                    actions.push(ProtectionAction::Mute {
                        user_id: context.message.author_id.clone(),
                        duration_seconds: 300,
                        reason: "洗版行為".to_string(),
                    });
                }
            },
            _ => {}
        }
        
        actions
    }
    
    /// 評估是否需要升級處理
    async fn check_escalation(&self, user_id: &str, threshold: usize) -> bool {
        let history = self.violation_history.read().await;
        if let Some(violations) = history.get(user_id) {
            let recent_violations = violations
                .iter()
                .filter(|&time| {
                    let age = Utc::now() - *time;
                    age.num_minutes() < 30
                })
                .count();
            recent_violations >= threshold
        } else {
            false
        }
    }
    
    /// 記錄違規歷史
    async fn record_violation(&self, user_id: &str) {
        let mut history = self.violation_history.write().await;
        let entry = history.entry(user_id.to_string()).or_insert_with(Vec::new);
        entry.push(Utc::now());
        
        // 清理超過1小時的記錄
        entry.retain(|time| {
            let age = Utc::now() - *time;
            age.num_hours() < 1
        });
    }
}

#[async_trait]
impl RulesEngine for DefaultRulesEngine {
    async fn evaluate(
        &self,
        context: &MessageContext,
        inspection: &InspectionResult,
    ) -> Result<RuleDecision> {
        let config = self.config.read().await;
        let mut all_actions = Vec::new();
        let mut should_take_action = false;
        let mut reasoning = Vec::new();
        
        // 檢查是否需要升級處理
        let escalated = self.check_escalation(
            &context.message.author_id,
            config.escalation_threshold
        ).await || context.author_violation_count >= config.escalation_threshold;
        
        if escalated {
            reasoning.push(format!(
                "用戶 {} 有多次違規記錄，升級處理",
                context.message.author_id
            ));
        }
        
        // 評估每個違規
        for violation in &inspection.violations {
            let severity_score = self.calculate_action_severity(
                &violation.severity,
                &context.guild_protection_level
            );
            
            let threshold = match context.guild_protection_level {
                ProtectionLevel::High => 0.3,
                ProtectionLevel::Medium => 0.5,
                ProtectionLevel::Low => 0.75,
            };
            
            if severity_score >= threshold || escalated {
                should_take_action = true;
                let actions = self.determine_actions(violation, context, escalated);
                
                reasoning.push(format!(
                    "{:?} 違規 (嚴重度: {:?}, 分數: {:.2})",
                    violation.violation_type,
                    violation.severity,
                    severity_score
                ));
                
                for action in actions {
                    if !all_actions.iter().any(|a| {
                        std::mem::discriminant(a) == std::mem::discriminant(&action)
                    }) {
                        all_actions.push(action);
                    }
                }
            }
        }
        
        // 如果有違規，記錄到歷史
        if should_take_action {
            self.record_violation(&context.message.author_id).await;
        }
        
        // 檢查自定義規則
        let custom_rules = self.custom_rules.read().await;
        for rule in custom_rules.values() {
            if rule.enabled {
                if let Ok(pattern) = Regex::new(&rule.pattern) {
                    if pattern.is_match(&context.message.content) {
                        should_take_action = true;
                        reasoning.push(format!("匹配自定義規則: {}", rule.name));
                        
                        match rule.action {
                            RuleAction::Delete => {
                                all_actions.push(ProtectionAction::DeleteMessage {
                                    message_id: context.message.id.clone(),
                                    reason: format!("自定義規則: {}", rule.name),
                                });
                            },
                            RuleAction::Mute => {
                                all_actions.push(ProtectionAction::Mute {
                                    user_id: context.message.author_id.clone(),
                                    duration_seconds: 600,
                                    reason: format!("自定義規則: {}", rule.name),
                                });
                            },
                            RuleAction::Warn => {
                                all_actions.push(ProtectionAction::Warn {
                                    user_id: context.message.author_id.clone(),
                                    reason: format!("自定義規則: {}", rule.name),
                                });
                            },
                            RuleAction::Ban => {
                                all_actions.push(ProtectionAction::Ban {
                                    user_id: context.message.author_id.clone(),
                                    reason: format!("自定義規則: {}", rule.name),
                                    delete_message_days: 1,
                                });
                            },
                            RuleAction::None => {},
                        }
                    }
                }
            }
        }
        
        Ok(RuleDecision {
            should_take_action,
            actions: all_actions,
            confidence: inspection.confidence,
            reasoning: reasoning.join("; "),
            escalated,
        })
    }
    
    async fn reload_rules(&mut self, config: RuleConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        tracing::info!("規則配置已重新載入");
        Ok(())
    }
    
    async fn update_protection_level(
        &mut self,
        guild_id: &str,
        level: ProtectionLevel,
    ) -> Result<()> {
        let mut levels = self.protection_levels.write().await;
        levels.insert(guild_id.to_string(), level);
        tracing::info!("群組 {} 的防護等級已更新為 {:?}", guild_id, level);
        Ok(())
    }
    
    async fn get_protection_level(&self, guild_id: &str) -> Result<ProtectionLevel> {
        let levels = self.protection_levels.read().await;
        Ok(levels.get(guild_id).copied().unwrap_or(ProtectionLevel::Medium))
    }
    
    async fn get_current_config(&self) -> Result<RuleConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }
    
    async fn add_custom_rule(
        &mut self,
        name: &str,
        pattern: &str,
        action: RuleAction,
    ) -> Result<()> {
        // 驗證正則表達式
        Regex::new(pattern)
            .map_err(|e| ProtectionError::ConfigurationError(
                format!("無效的正則表達式: {}", e)
            ))?;
        
        let mut rules = self.custom_rules.write().await;
        rules.insert(
            name.to_string(),
            CustomRule {
                name: name.to_string(),
                pattern: pattern.to_string(),
                action,
                enabled: true,
                created_at: Utc::now(),
                last_modified: Utc::now(),
            }
        );
        
        tracing::info!("已添加自定義規則: {}", name);
        Ok(())
    }
    
    async fn get_custom_rules(&self) -> Result<HashMap<String, (String, RuleAction)>> {
        let rules = self.custom_rules.read().await;
        Ok(rules
            .iter()
            .map(|(k, v)| (k.clone(), (v.pattern.clone(), v.action)))
            .collect())
    }
}

impl Default for DefaultRulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_escalation_check() {
        let engine = DefaultRulesEngine::new();
        
        // 添加違規記錄
        engine.record_violation("test_user").await;
        engine.record_violation("test_user").await;
        engine.record_violation("test_user").await;
        
        // 檢查是否需要升級
        let escalated = engine.check_escalation("test_user", 3).await;
        assert!(escalated);
    }
    
    #[tokio::test]
    async fn test_custom_rule_validation() {
        let mut engine = DefaultRulesEngine::new();
        
        // 有效的規則
        let result = engine.add_custom_rule(
            "test_rule",
            r"^[A-Z]+$",
            RuleAction::Warn
        ).await;
        assert!(result.is_ok());
        
        // 無效的規則
        let result = engine.add_custom_rule(
            "invalid_rule",
            r"[invalid(regex",
            RuleAction::Delete
        ).await;
        assert!(result.is_err());
    }
}
