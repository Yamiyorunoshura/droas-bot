//! Pattern Recognition Service
//!
//! 提供各種訊息模式識別功能，包括垃圾訊息檢測、重複訊息識別、
//! 洗版行為檢測和連結安全檢測。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use chrono::{DateTime, Utc, Duration};
use crate::protection::{Message, ViolationType};
use crate::ProtectionLevel;

/// 垃圾訊息分數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamScore {
    pub score: f32,
    pub violation_type: Option<ViolationType>,
    pub confidence: f32,
}

/// 連結安全狀態
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinkSafetyStatus {
    Safe,
    Unsafe,
    Unknown,
}

/// 安全檢測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyResult {
    pub status: LinkSafetyStatus,
    pub risk_score: f32,
    pub details: Vec<String>,
}

/// 洗版檢測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloodingResult {
    pub is_flooding: bool,
    pub messages_per_second: f32,
    pub violation_type: ViolationType,
}

/// 重複訊息檢測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionResult {
    pub has_duplicates: bool,
    pub duplicate_count: usize,
    pub similarity_score: f32,
}

/// Pattern Recognizer trait
#[async_trait]
pub trait PatternRecognizer: Send + Sync {
    /// 檢測垃圾訊息
    async fn detect_spam(&self, content: &str) -> Result<SpamScore, Box<dyn std::error::Error>>;
    
    /// 檢測重複訊息
    async fn detect_duplicates(&self, messages: &[Message]) -> Result<DuplicateDetectionResult, Box<dyn std::error::Error>>;
    
    /// 檢測洗版行為
    async fn detect_flooding(&self, messages: &[Message]) -> Result<FloodingResult, Box<dyn std::error::Error>>;
    
    /// 檢查連結安全性
    async fn check_link_safety(&self, links: &[String]) -> Result<SafetyResult, Box<dyn std::error::Error>>;
    
    /// 根據防護等級檢測垃圾訊息
    async fn detect_spam_with_level(
        &self,
        content: &str,
        level: ProtectionLevel,
    ) -> Result<SpamScore, Box<dyn std::error::Error>>;
}

/// 默認的 Pattern Recognizer 實現
pub struct DefaultPatternRecognizer {
    spam_keywords: HashSet<String>,
    safe_domains: HashSet<String>,
    unsafe_domains: HashSet<String>,
    url_shorteners: HashSet<String>,
    emoji_spam_patterns: Vec<Regex>,
}

impl DefaultPatternRecognizer {
    /// 創建新的 Pattern Recognizer
    pub fn new() -> Self {
        let mut spam_keywords = HashSet::new();
        spam_keywords.insert("free".to_string());
        spam_keywords.insert("money".to_string());
        spam_keywords.insert("prize".to_string());
        spam_keywords.insert("winner".to_string());
        spam_keywords.insert("click here".to_string());
        spam_keywords.insert("limited offer".to_string());
        spam_keywords.insert("buy now".to_string());
        spam_keywords.insert("discount".to_string());
        spam_keywords.insert("crypto".to_string());
        spam_keywords.insert("giveaway".to_string());
        spam_keywords.insert("earn".to_string());
        spam_keywords.insert("investment".to_string());
        
        let mut safe_domains = HashSet::new();
        safe_domains.insert("discord.com".to_string());
        safe_domains.insert("github.com".to_string());
        safe_domains.insert("youtube.com".to_string());
        safe_domains.insert("wikipedia.org".to_string());
        safe_domains.insert("google.com".to_string());
        safe_domains.insert("twitter.com".to_string());
        safe_domains.insert("reddit.com".to_string());
        
        let mut unsafe_domains = HashSet::new();
        unsafe_domains.insert("phishing-site.tk".to_string());
        unsafe_domains.insert("scam.site".to_string());
        unsafe_domains.insert("suspicious-download.exe".to_string());
        
        let mut url_shorteners = HashSet::new();
        url_shorteners.insert("bit.ly".to_string());
        url_shorteners.insert("tinyurl.com".to_string());
        url_shorteners.insert("short.link".to_string());
        url_shorteners.insert("t.co".to_string());
        
        let emoji_spam_patterns = vec![
            Regex::new(r"💰{3,}").unwrap(),
            Regex::new(r"🔥{3,}").unwrap(),
            Regex::new(r"₿{3,}").unwrap(),
            Regex::new(r"([💰🔥💎💸]{2,}.*){3,}").unwrap(),
        ];
        
        Self {
            spam_keywords,
            safe_domains,
            unsafe_domains,
            url_shorteners,
            emoji_spam_patterns,
        }
    }
    
    /// 計算 Levenshtein 距離
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = *[
                    matrix[i][j + 1] + 1,     // 刪除
                    matrix[i + 1][j] + 1,     // 插入
                    matrix[i][j] + cost,      // 替換
                ].iter().min().unwrap();
            }
        }
        
        matrix[len1][len2]
    }
    
    /// 計算相似度分數 (0.0 到 1.0)
    fn similarity_score(&self, s1: &str, s2: &str) -> f32 {
        let distance = self.levenshtein_distance(s1, s2);
        let max_len = s1.len().max(s2.len()) as f32;
        if max_len == 0.0 {
            return 1.0;
        }
        1.0 - (distance as f32 / max_len)
    }
    
    /// 從 URL 提取域名
    fn extract_domain(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.host_str().map(|h| h.to_string())
        } else {
            // 嘗試簡單的域名提取
            url.split('/').nth(2).map(|s| s.to_string())
        }
    }
}

#[async_trait]
impl PatternRecognizer for DefaultPatternRecognizer {
    async fn detect_spam(&self, content: &str) -> Result<SpamScore, Box<dyn std::error::Error>> {
        let mut score: f32 = 0.0;
        let content_lower = content.to_lowercase();
        
        // 檢查垃圾關鍵詞
        let mut keyword_count = 0;
        for keyword in &self.spam_keywords {
            if content_lower.contains(keyword) {
                keyword_count += 1;
                score += 0.15;
            }
        }
        
        // 檢查大寫字母比例
        let uppercase_count = content.chars().filter(|c| c.is_uppercase()).count();
        let total_alpha = content.chars().filter(|c| c.is_alphabetic()).count();
        if total_alpha > 0 {
            let uppercase_ratio = uppercase_count as f32 / total_alpha as f32;
            if uppercase_ratio > 0.5 {
                score += 0.25;
            } else if uppercase_ratio > 0.3 {
                score += 0.15;
            }
        }
        
        // 檢查感嘆號數量
        let exclamation_count = content.matches('!').count();
        if exclamation_count > 5 {
            score += 0.3;
        } else if exclamation_count > 3 {
            score += 0.15;
        }
        
        // 檢查 emoji 垃圾模式
        for pattern in &self.emoji_spam_patterns {
            if pattern.is_match(content) {
                score += 0.25;
                break;
            }
        }
        
        // 檢查重複字符
        if let Some(caps) = Regex::new(r"(.)\1{4,}").unwrap().captures(content) {
            score += 0.2;
        }
        
        // 檢查全形字符垃圾訊息
        if content.contains("ＦＲ") || content.contains("ＭＯＮＥＹ") || 
           content.contains("ＣＬＩＣＫ") || content.contains("ＧＩＶＥＡＷＡＹ") {
            score += 0.3;
        }
        
        // 檢查多個連結
        let link_count = Regex::new(r"https?://[^\s]+").unwrap()
            .find_iter(content)
            .count();
        if link_count > 3 {
            score += 0.2;
        }
        
        // 檢查 Discord 邀請連結重複
        let discord_invites = Regex::new(r"discord\.gg/\w+").unwrap()
            .find_iter(content)
            .count();
        if discord_invites > 2 {
            score += 0.25;
        }
        
        score = score.min(1.0f32);
        
        Ok(SpamScore {
            score,
            violation_type: if score > 0.7 { 
                Some(ViolationType::Spam) 
            } else { 
                None 
            },
            confidence: if keyword_count > 2 { 0.9 } else { 0.75 },
        })
    }
    
    async fn detect_duplicates(&self, messages: &[Message]) -> Result<DuplicateDetectionResult, Box<dyn std::error::Error>> {
        if messages.is_empty() {
            return Ok(DuplicateDetectionResult {
                has_duplicates: false,
                duplicate_count: 0,
                similarity_score: 0.0,
            });
        }
        
        let mut content_map: HashMap<String, usize> = HashMap::new();
        let mut max_similarity = 0.0f32;
        let mut duplicate_count = 0;
        
        // 計算完全相同的訊息
        for message in messages {
            *content_map.entry(message.content.clone()).or_insert(0) += 1;
        }
        
        for count in content_map.values() {
            if *count > 1 {
                duplicate_count = duplicate_count.max(*count);
            }
        }
        
        // 計算相似訊息
        for i in 0..messages.len() {
            for j in i + 1..messages.len() {
                let similarity = self.similarity_score(
                    &messages[i].content,
                    &messages[j].content
                );
                max_similarity = max_similarity.max(similarity);
            }
        }
        
        Ok(DuplicateDetectionResult {
            has_duplicates: duplicate_count > 1 || max_similarity > 0.8,
            duplicate_count,
            similarity_score: if duplicate_count > 1 { 1.0 } else { max_similarity },
        })
    }
    
    async fn detect_flooding(&self, messages: &[Message]) -> Result<FloodingResult, Box<dyn std::error::Error>> {
        if messages.len() < 2 {
            return Ok(FloodingResult {
                is_flooding: false,
                messages_per_second: 0.0,
                violation_type: ViolationType::Flooding,
            });
        }
        
        // 按時間排序
        let mut sorted_messages = messages.to_vec();
        sorted_messages.sort_by_key(|m| m.timestamp);
        
        // 計算訊息速率
        let first_time = sorted_messages.first().unwrap().timestamp;
        let last_time = sorted_messages.last().unwrap().timestamp;
        let time_span = (last_time - first_time).num_seconds() as f32;
        
        if time_span <= 0.0 {
            // 所有訊息在同一秒內
            return Ok(FloodingResult {
                is_flooding: messages.len() > 5,
                messages_per_second: messages.len() as f32,
                violation_type: ViolationType::Flooding,
            });
        }
        
        let messages_per_second = messages.len() as f32 / time_span;
        
        // 檢查短時間窗口內的訊息數量
        let mut max_burst = 0;
        let window = Duration::seconds(5);
        
        for i in 0..sorted_messages.len() {
            let window_start = sorted_messages[i].timestamp;
            let window_end = window_start + window;
            let count = sorted_messages.iter()
                .filter(|m| m.timestamp >= window_start && m.timestamp <= window_end)
                .count();
            max_burst = max_burst.max(count);
        }
        
        let is_flooding = messages_per_second > 2.0 || max_burst > 10;
        
        Ok(FloodingResult {
            is_flooding,
            messages_per_second,
            violation_type: ViolationType::Flooding,
        })
    }
    
    async fn check_link_safety(&self, links: &[String]) -> Result<SafetyResult, Box<dyn std::error::Error>> {
        let mut risk_score: f32 = 0.0;
        let mut details = Vec::new();
        
        for link in links {
            // 檢查是否為 IP 地址
            if Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap().is_match(link) {
                risk_score += 0.3;
                details.push("包含 IP 地址連結".to_string());
            }
            
            // 檢查可執行檔案
            if link.ends_with(".exe") || link.ends_with(".bat") || 
               link.ends_with(".cmd") || link.ends_with(".scr") {
                risk_score += 0.5;
                details.push("包含可執行檔案連結".to_string());
            }
            
            // 提取域名並檢查
            if let Some(domain) = self.extract_domain(link) {
                // 檢查已知的不安全域名
                if self.unsafe_domains.contains(&domain) {
                    risk_score += 0.8;
                    details.push(format!("已知的不安全域名: {}", domain));
                }
                // 檢查 URL 縮短服務
                else if self.url_shorteners.contains(&domain) {
                    risk_score += 0.3;
                    details.push(format!("URL 縮短服務: {}", domain));
                }
                // 檢查已知的安全域名
                else if self.safe_domains.contains(&domain) {
                    risk_score -= 0.2;
                }
                // 檢查可疑的頂級域名
                else if domain.ends_with(".tk") || domain.ends_with(".ml") || 
                        domain.ends_with(".ga") || domain.ends_with(".cf") {
                    risk_score += 0.4;
                    details.push(format!("可疑的頂級域名: {}", domain));
                }
            }
            
            // 檢查 HTTP vs HTTPS
            if link.starts_with("http://") && !link.starts_with("https://") {
                risk_score += 0.1;
                details.push("使用不安全的 HTTP 協議".to_string());
            }
        }
        
        risk_score = risk_score.max(0.0f32).min(1.0f32);
        
        let status = if risk_score > 0.7 {
            LinkSafetyStatus::Unsafe
        } else if risk_score < 0.3 {
            LinkSafetyStatus::Safe
        } else {
            LinkSafetyStatus::Unknown
        };
        
        Ok(SafetyResult {
            status,
            risk_score,
            details,
        })
    }
    
    async fn detect_spam_with_level(
        &self,
        content: &str,
        level: ProtectionLevel,
    ) -> Result<SpamScore, Box<dyn std::error::Error>> {
        let mut base_score = self.detect_spam(content).await?;
        
        // 根據防護等級調整靈敏度
        let multiplier = match level {
            ProtectionLevel::High => 1.3,
            ProtectionLevel::Medium => 1.0,
            ProtectionLevel::Low => 0.7,
        };
        
        base_score.score = (base_score.score * multiplier).min(1.0);
        
        // 調整違規類型閾值
        base_score.violation_type = match level {
            ProtectionLevel::High if base_score.score > 0.5 => Some(ViolationType::Spam),
            ProtectionLevel::Medium if base_score.score > 0.7 => Some(ViolationType::Spam),
            ProtectionLevel::Low if base_score.score > 0.85 => Some(ViolationType::Spam),
            _ => None,
        };
        
        Ok(base_score)
    }
}

impl Default for DefaultPatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_levenshtein_distance() {
        let recognizer = DefaultPatternRecognizer::new();
        assert_eq!(recognizer.levenshtein_distance("hello", "hello"), 0);
        assert_eq!(recognizer.levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(recognizer.levenshtein_distance("", "hello"), 5);
    }
    
    #[tokio::test]
    async fn test_similarity_score() {
        let recognizer = DefaultPatternRecognizer::new();
        assert_eq!(recognizer.similarity_score("hello", "hello"), 1.0);
        assert!(recognizer.similarity_score("hello", "hallo") > 0.7);
        assert!(recognizer.similarity_score("hello", "world") < 0.3);
    }
}
