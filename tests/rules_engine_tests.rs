//! Rules Engine 測試案例
//! 
//! 測試規則引擎的配置、評估邏輯和動態載入功能

use droas_bot::protection::{
    rules_engine::{RulesEngine, RuleDecision, RuleConfig, RuleAction},
    MessageContext, Message, ProtectionLevel, ViolationType, 
    InspectionResult, Violation, Severity, ProtectionAction,
};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 創建測試用訊息上下文
fn create_test_context(protection_level: ProtectionLevel) -> MessageContext {
    MessageContext {
        message: Message {
            id: "test_msg_1".to_string(),
            author_id: "test_user".to_string(),
            guild_id: "test_guild".to_string(),
            channel_id: "test_channel".to_string(),
            content: "Test message".to_string(),
            timestamp: Utc::now(),
            attachments: vec![],
            embeds: vec![],
            mentions: vec![],
        },
        author_history: vec![],
        channel_recent_messages: vec![],
        author_violation_count: 0,
        guild_protection_level: protection_level,
    }
}

/// 創建測試用檢測結果
fn create_inspection_result(violations: Vec<Violation>) -> InspectionResult {
    let risk_score = violations.iter()
        .map(|v| match v.severity {
            Severity::Critical => 1.0,
            Severity::High => 0.75,
            Severity::Medium => 0.5,
            Severity::Low => 0.25,
        })
        .sum::<f32>() / violations.len().max(1) as f32;
    
    InspectionResult {
        message_id: "test_msg_1".to_string(),
        violations,
        risk_score,
        suggested_actions: vec![],
        confidence: 0.85,
    }
}

#[tokio::test]
async fn test_rule_evaluation_basic() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    let context = create_test_context(ProtectionLevel::Medium);
    
    // 測試單一違規
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::Spam,
            severity: Severity::High,
            description: "垃圾訊息檢測".to_string(),
            evidence: "包含多個垃圾關鍵詞".to_string(),
        }
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    assert!(decision.should_take_action);
    assert!(!decision.actions.is_empty());
    assert_eq!(decision.confidence, 0.85);
}

#[tokio::test]
async fn test_protection_level_loose() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    let context = create_test_context(ProtectionLevel::Loose);
    
    // 寬鬆模式下，低嚴重度違規不應觸發動作
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::Spam,
            severity: Severity::Low,
            description: "輕微垃圾訊息".to_string(),
            evidence: "包含一個推廣連結".to_string(),
        }
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    assert!(!decision.should_take_action);
    assert!(decision.actions.is_empty());
}

#[tokio::test]
async fn test_protection_level_strict() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    let context = create_test_context(ProtectionLevel::Strict);
    
    // 嚴格模式下，即使低嚴重度違規也應觸發動作
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::DuplicateMessage,
            severity: Severity::Low,
            description: "重複訊息".to_string(),
            evidence: "相似度 70%".to_string(),
        }
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    assert!(decision.should_take_action);
    assert!(!decision.actions.is_empty());
}

#[tokio::test]
async fn test_multiple_violations() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    let context = create_test_context(ProtectionLevel::Medium);
    
    // 多重違規應該觸發更嚴厲的動作
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::Spam,
            severity: Severity::Medium,
            description: "垃圾訊息".to_string(),
            evidence: "包含垃圾關鍵詞".to_string(),
        },
        Violation {
            violation_type: ViolationType::Flooding,
            severity: Severity::High,
            description: "洗版行為".to_string(),
            evidence: "5秒內發送10條訊息".to_string(),
        },
        Violation {
            violation_type: ViolationType::UnsafeLink,
            severity: Severity::Critical,
            description: "惡意連結".to_string(),
            evidence: "包含已知釣魚網站".to_string(),
        },
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    assert!(decision.should_take_action);
    assert!(decision.actions.len() >= 2); // 應該有多個動作
    assert!(decision.confidence > 0.9);
}

#[tokio::test]
async fn test_author_violation_history() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    
    // 測試有違規歷史的用戶
    let mut context = create_test_context(ProtectionLevel::Medium);
    context.author_violation_count = 5;
    
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::Spam,
            severity: Severity::Low,
            description: "輕微違規".to_string(),
            evidence: "包含一個連結".to_string(),
        }
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    // 有違規歷史的用戶應該受到更嚴格的處理
    assert!(decision.should_take_action);
    assert!(decision.escalated);
}

#[tokio::test]
async fn test_rule_reload() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    
    // 測試規則重載
    let new_config = RuleConfig {
        spam_threshold: 0.5,
        flooding_threshold: 3.0,
        duplicate_threshold: 0.7,
        link_safety_threshold: 0.6,
        escalation_threshold: 3,
    };
    
    engine.write().await
        .reload_rules(new_config)
        .await
        .unwrap();
    
    // 驗證新規則已生效
    let config = engine.read().await.get_current_config().await.unwrap();
    assert_eq!(config.spam_threshold, 0.5);
}

#[tokio::test]
async fn test_update_protection_level() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    
    // 測試更新防護等級
    engine.write().await
        .update_protection_level("test_guild", ProtectionLevel::Strict)
        .await
        .unwrap();
    
    let level = engine.read().await
        .get_protection_level("test_guild")
        .await
        .unwrap();
    
    assert_eq!(level, ProtectionLevel::Strict);
}

#[tokio::test]
async fn test_action_priority() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    let context = create_test_context(ProtectionLevel::Medium);
    
    // Critical 違規應該產生最高優先級的動作
    let inspection = create_inspection_result(vec![
        Violation {
            violation_type: ViolationType::UnsafeLink,
            severity: Severity::Critical,
            description: "惡意連結".to_string(),
            evidence: "已確認的釣魚網站".to_string(),
        }
    ]);
    
    let decision = engine.read().await
        .evaluate(&context, &inspection)
        .await
        .unwrap();
    
    // 應該包含立即刪除和禁言動作
    assert!(decision.actions.iter().any(|a| matches!(a, ProtectionAction::DeleteMessage { .. })));
    assert!(decision.actions.iter().any(|a| matches!(a, ProtectionAction::Mute { .. })));
}

#[tokio::test]
async fn test_custom_rules() {
    let engine = Arc::new(RwLock::new(MockRulesEngine::new()));
    
    // 測試自定義規則
    engine.write().await
        .add_custom_rule("no_caps", r"^[A-Z\s]+$", RuleAction::Warn)
        .await
        .unwrap();
    
    let rules = engine.read().await.get_custom_rules().await.unwrap();
    assert!(rules.contains_key("no_caps"));
}

// Mock 實現
struct MockRulesEngine {
    config: RuleConfig,
    protection_levels: std::collections::HashMap<String, ProtectionLevel>,
    custom_rules: std::collections::HashMap<String, (String, RuleAction)>,
}

impl MockRulesEngine {
    fn new() -> Self {
        Self {
            config: RuleConfig::default(),
            protection_levels: std::collections::HashMap::new(),
            custom_rules: std::collections::HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl RulesEngine for MockRulesEngine {
    async fn evaluate(
        &self, 
        context: &MessageContext, 
        inspection: &InspectionResult
    ) -> Result<RuleDecision, Box<dyn std::error::Error>> {
        let mut actions = vec![];
        let mut should_take_action = false;
        let mut escalated = false;
        
        // 根據防護等級和違規嚴重度決定動作
        for violation in &inspection.violations {
            let severity_score = match violation.severity {
                Severity::Critical => 1.0,
                Severity::High => 0.75,
                Severity::Medium => 0.5,
                Severity::Low => 0.25,
            };
            
            let threshold = match context.guild_protection_level {
                ProtectionLevel::Strict => 0.2,
                ProtectionLevel::Medium => 0.5,
                ProtectionLevel::Loose => 0.75,
            };
            
            if severity_score >= threshold || context.author_violation_count >= 3 {
                should_take_action = true;
                
                if context.author_violation_count >= 3 {
                    escalated = true;
                }
                
                // 根據違規類型添加動作
                match violation.severity {
                    Severity::Critical => {
                        actions.push(ProtectionAction::DeleteMessage {
                            message_id: context.message.id.clone(),
                            reason: violation.description.clone(),
                        });
                        actions.push(ProtectionAction::Mute {
                            user_id: context.message.author_id.clone(),
                            duration_seconds: 3600,
                            reason: violation.description.clone(),
                        });
                    },
                    Severity::High => {
                        actions.push(ProtectionAction::DeleteMessage {
                            message_id: context.message.id.clone(),
                            reason: violation.description.clone(),
                        });
                        if escalated {
                            actions.push(ProtectionAction::Mute {
                                user_id: context.message.author_id.clone(),
                                duration_seconds: 600,
                                reason: "多次違規".to_string(),
                            });
                        }
                    },
                    Severity::Medium => {
                        if context.guild_protection_level != ProtectionLevel::Loose {
                            actions.push(ProtectionAction::Warn {
                                user_id: context.message.author_id.clone(),
                                reason: violation.description.clone(),
                            });
                        }
                    },
                    Severity::Low => {
                        if context.guild_protection_level == ProtectionLevel::Strict {
                            actions.push(ProtectionAction::Warn {
                                user_id: context.message.author_id.clone(),
                                reason: violation.description.clone(),
                            });
                        }
                    },
                }
            }
        }
        
        Ok(RuleDecision {
            should_take_action,
            actions,
            confidence: inspection.confidence,
            reasoning: "基於違規嚴重度和防護等級的決策".to_string(),
            escalated,
        })
    }
    
    async fn reload_rules(&mut self, config: RuleConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config = config;
        Ok(())
    }
    
    async fn update_protection_level(
        &mut self, 
        guild_id: &str, 
        level: ProtectionLevel
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.protection_levels.insert(guild_id.to_string(), level);
        Ok(())
    }
    
    async fn get_protection_level(&self, guild_id: &str) -> Result<ProtectionLevel, Box<dyn std::error::Error>> {
        Ok(self.protection_levels.get(guild_id)
            .copied()
            .unwrap_or(ProtectionLevel::Medium))
    }
    
    async fn get_current_config(&self) -> Result<RuleConfig, Box<dyn std::error::Error>> {
        Ok(self.config.clone())
    }
    
    async fn add_custom_rule(
        &mut self,
        name: &str,
        pattern: &str,
        action: RuleAction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.custom_rules.insert(
            name.to_string(), 
            (pattern.to_string(), action)
        );
        Ok(())
    }
    
    async fn get_custom_rules(&self) -> Result<std::collections::HashMap<String, (String, RuleAction)>, Box<dyn std::error::Error>> {
        Ok(self.custom_rules.clone())
    }
}
