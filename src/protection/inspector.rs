//! Message Inspector Service
//!
//! 整合 Pattern Recognition 和 Rules Engine，提供高效能的訊息檢測服務。
//! 使用 Event-Driven Architecture 和 tokio channels 進行非同步處理。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, Instant};
use crate::protection::{
    MessageContext, InspectionResult, Result, ProtectionError,
    Violation, Severity, ViolationType, ProtectionAction,
    pattern_recognition::{PatternRecognizer, DefaultPatternRecognizer},
    rules_engine::{RulesEngine, DefaultRulesEngine},
};
use regex::Regex;

/// Inspector 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorConfig {
    pub max_concurrent_inspections: usize,
    pub inspection_timeout_ms: u64,
    pub cache_size: usize,
    pub enable_caching: bool,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_inspections: 100,
            inspection_timeout_ms: 100,
            cache_size: 1000,
            enable_caching: true,
        }
    }
}

/// Message Inspector trait
#[async_trait]
pub trait MessageInspector: Send + Sync {
    /// 檢測訊息
    async fn inspect(&self, context: &MessageContext) -> Result<InspectionResult>;
    
    /// 批次檢測訊息
    async fn inspect_batch(&self, contexts: Vec<MessageContext>) -> Result<Vec<InspectionResult>>;
    
    /// 獲取性能統計
    async fn get_performance_stats(&self) -> Result<PerformanceStats>;
}

/// 性能統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_inspections: u64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub messages_per_second: f64,
}

/// 檢測事件
#[derive(Debug)]
struct InspectionEvent {
    context: MessageContext,
    response_tx: tokio::sync::oneshot::Sender<Result<InspectionResult>>,
}

/// 默認的 Message Inspector 實現
pub struct DefaultMessageInspector {
    config: Arc<InspectorConfig>,
    pattern_recognizer: Arc<dyn PatternRecognizer>,
    inspection_tx: mpsc::Sender<InspectionEvent>,
    stats: Arc<RwLock<InspectorStats>>,
    cache: Arc<RwLock<lru::LruCache<String, InspectionResult>>>,
}

/// 內部統計數據
struct InspectorStats {
    total_inspections: u64,
    total_latency_ms: u64,
    latencies: Vec<u64>,
    cache_hits: u64,
    cache_misses: u64,
    start_time: Instant,
}

impl DefaultMessageInspector {
    /// 創建新的 Message Inspector
    pub fn new(config: InspectorConfig) -> Self {
        let pattern_recognizer: Arc<dyn PatternRecognizer> = Arc::new(DefaultPatternRecognizer::new());
        let (inspection_tx, mut inspection_rx) = mpsc::channel::<InspectionEvent>(1000);
        let stats = Arc::new(RwLock::new(InspectorStats {
            total_inspections: 0,
            total_latency_ms: 0,
            latencies: Vec::new(),
            cache_hits: 0,
            cache_misses: 0,
            start_time: Instant::now(),
        }));
        let cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap()
        )));
        
        let inspector = Self {
            config: Arc::new(config.clone()),
            pattern_recognizer: pattern_recognizer.clone(),
            inspection_tx,
            stats: stats.clone(),
            cache: cache.clone(),
        };
        
        // 啟動單一工作線程處理所有檢測請求
        {
            let pattern_recognizer = pattern_recognizer.clone();
            let stats = stats.clone();
            let cache = cache.clone();
            let config = Arc::new(config.clone());
            
            tokio::spawn(async move {
                while let Some(event) = inspection_rx.recv().await {
                    let start = Instant::now();
                    let result = Self::process_inspection(
                        &event.context,
                        &pattern_recognizer,
                        &cache,
                        &config,
                    ).await;
                    
                    // 更新統計
                    let latency = start.elapsed().as_millis() as u64;
                    {
                        let mut stats = stats.write().await;
                        stats.total_inspections += 1;
                        stats.total_latency_ms += latency;
                        stats.latencies.push(latency);
                        if stats.latencies.len() > 1000 {
                            stats.latencies.remove(0);
                        }
                    }
                    
                    let _ = event.response_tx.send(result);
                }
            });
        }
        
        inspector
    }
    
    /// 處理單個檢測請求
    async fn process_inspection(
        context: &MessageContext,
        pattern_recognizer: &Arc<dyn PatternRecognizer>,
        cache: &Arc<RwLock<lru::LruCache<String, InspectionResult>>>,
        config: &Arc<InspectorConfig>,
    ) -> Result<InspectionResult> {
        // 檢查快取
        if config.enable_caching {
            let cache_key = format!("{}:{}", context.message.id, context.message.content);
            let mut cache = cache.write().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        let mut violations = Vec::new();
        let mut risk_score: f32 = 0.0;
        let mut suggested_actions = Vec::new();
        
        // 檢測垃圾訊息
        let spam_result = pattern_recognizer
            .detect_spam_with_level(&context.message.content, context.guild_protection_level)
            .await
            .map_err(|e| ProtectionError::InspectionFailed(e.to_string()))?;
        
        if let Some(violation_type) = spam_result.violation_type {
            violations.push(Violation {
                violation_type,
                severity: if spam_result.score > 0.9 {
                    Severity::Critical
                } else if spam_result.score > 0.7 {
                    Severity::High
                } else if spam_result.score > 0.5 {
                    Severity::Medium
                } else {
                    Severity::Low
                },
                description: "垃圾訊息檢測".to_string(),
                evidence: format!("垃圾訊息分數: {:.2}", spam_result.score),
            });
            risk_score = risk_score.max(spam_result.score);
        }
        
        // 檢測重複訊息
        if !context.author_history.is_empty() {
            let duplicate_result = pattern_recognizer
                .detect_duplicates(&context.author_history)
                .await
                .map_err(|e| ProtectionError::InspectionFailed(e.to_string()))?;
            
            if duplicate_result.has_duplicates {
                violations.push(Violation {
                    violation_type: ViolationType::DuplicateMessage,
                    severity: if duplicate_result.duplicate_count > 5 {
                        Severity::High
                    } else if duplicate_result.duplicate_count > 3 {
                        Severity::Medium
                    } else {
                        Severity::Low
                    },
                    description: "重複訊息".to_string(),
                    evidence: format!(
                        "重複 {} 次，相似度 {:.2}",
                        duplicate_result.duplicate_count,
                        duplicate_result.similarity_score
                    ),
                });
                risk_score = risk_score.max(duplicate_result.similarity_score);
            }
        }
        
        // 檢測洗版行為
        if !context.channel_recent_messages.is_empty() {
            let user_messages: Vec<_> = context.channel_recent_messages
                .iter()
                .filter(|m| m.author_id == context.message.author_id)
                .cloned()
                .collect();
            
            if user_messages.len() >= 3 {
                let flooding_result = pattern_recognizer
                    .detect_flooding(&user_messages)
                    .await
                    .map_err(|e| ProtectionError::InspectionFailed(e.to_string()))?;
                
                if flooding_result.is_flooding {
                    violations.push(Violation {
                        violation_type: ViolationType::Flooding,
                        severity: if flooding_result.messages_per_second > 10.0 {
                            Severity::Critical
                        } else if flooding_result.messages_per_second > 5.0 {
                            Severity::High
                        } else if flooding_result.messages_per_second > 2.0 {
                            Severity::Medium
                        } else {
                            Severity::Low
                        },
                        description: "洗版行為".to_string(),
                        evidence: format!(
                            "訊息速率: {:.2} 訊息/秒",
                            flooding_result.messages_per_second
                        ),
                    });
                    risk_score = risk_score.max(flooding_result.messages_per_second / 10.0);
                }
            }
        }
        
        // 檢測不安全連結
        let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
        let links: Vec<String> = url_pattern
            .find_iter(&context.message.content)
            .map(|m| m.as_str().to_string())
            .collect();
        
        if !links.is_empty() {
            let safety_result = pattern_recognizer
                .check_link_safety(&links)
                .await
                .map_err(|e| ProtectionError::InspectionFailed(e.to_string()))?;
            
            if safety_result.status == crate::protection::pattern_recognition::LinkSafetyStatus::Unsafe {
                violations.push(Violation {
                    violation_type: ViolationType::UnsafeLink,
                    severity: if safety_result.risk_score > 0.8 {
                        Severity::Critical
                    } else if safety_result.risk_score > 0.6 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    description: "不安全連結".to_string(),
                    evidence: safety_result.details.join("; "),
                });
                risk_score = risk_score.max(safety_result.risk_score);
                
                // 不安全連結建議立即刪除
                suggested_actions.push(ProtectionAction::DeleteMessage {
                    message_id: context.message.id.clone(),
                    reason: "包含不安全連結".to_string(),
                });
            }
        }
        
        // 計算信心分數
        let confidence = if violations.is_empty() {
            1.0
        } else {
            0.7 + (violations.len() as f32 * 0.1).min(0.3)
        };
        
        let result = InspectionResult {
            message_id: context.message.id.clone(),
            violations,
            risk_score: risk_score.min(1.0),
            suggested_actions,
            confidence,
        };
        
        // 更新快取
        if config.enable_caching {
            let cache_key = format!("{}:{}", context.message.id, context.message.content);
            let mut cache = cache.write().await;
            cache.put(cache_key, result.clone());
        }
        
        Ok(result)
    }
}

#[async_trait]
impl MessageInspector for DefaultMessageInspector {
    async fn inspect(&self, context: &MessageContext) -> Result<InspectionResult> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        let event = InspectionEvent {
            context: context.clone(),
            response_tx,
        };
        
        self.inspection_tx
            .send(event)
            .await
            .map_err(|_| ProtectionError::InspectionFailed("檢測服務已關閉".to_string()))?;
        
        tokio::time::timeout(
            Duration::from_millis(self.config.inspection_timeout_ms),
            response_rx,
        )
        .await
        .map_err(|_| ProtectionError::InspectionFailed("檢測超時".to_string()))?
        .map_err(|_| ProtectionError::InspectionFailed("檢測失敗".to_string()))?
    }
    
    async fn inspect_batch(&self, contexts: Vec<MessageContext>) -> Result<Vec<InspectionResult>> {
        let mut results = Vec::new();
        let mut handles = Vec::new();
        
        for context in contexts {
            let inspector = self.clone();
            let handle = tokio::spawn(async move {
                inspector.inspect(&context).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(_) => return Err(ProtectionError::InspectionFailed("批次檢測失敗".to_string())),
            }
        }
        
        Ok(results)
    }
    
    async fn get_performance_stats(&self) -> Result<PerformanceStats> {
        let stats = self.stats.read().await;
        
        let mut sorted_latencies = stats.latencies.clone();
        sorted_latencies.sort_unstable();
        
        let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;
        
        let elapsed_seconds = stats.start_time.elapsed().as_secs_f64();
        
        Ok(PerformanceStats {
            total_inspections: stats.total_inspections,
            average_latency_ms: if stats.total_inspections > 0 {
                stats.total_latency_ms as f64 / stats.total_inspections as f64
            } else {
                0.0
            },
            p95_latency_ms: sorted_latencies.get(p95_index).copied().unwrap_or(0) as f64,
            p99_latency_ms: sorted_latencies.get(p99_index).copied().unwrap_or(0) as f64,
            cache_hit_rate: if stats.cache_hits + stats.cache_misses > 0 {
                stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
            } else {
                0.0
            },
            messages_per_second: if elapsed_seconds > 0.0 {
                stats.total_inspections as f64 / elapsed_seconds
            } else {
                0.0
            },
        })
    }
}

impl Clone for DefaultMessageInspector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pattern_recognizer: self.pattern_recognizer.clone(),
            inspection_tx: self.inspection_tx.clone(),
            stats: self.stats.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtectionLevel;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_inspector_creation() {
        let config = InspectorConfig::default();
        let _inspector = DefaultMessageInspector::new(config);
    }
    
    #[tokio::test]
    async fn test_performance_stats() {
        let config = InspectorConfig::default();
        let inspector = DefaultMessageInspector::new(config);
        
        let stats = inspector.get_performance_stats().await.unwrap();
        assert_eq!(stats.total_inspections, 0);
    }
}
