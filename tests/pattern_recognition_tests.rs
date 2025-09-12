//! Pattern Recognition Service 測試案例
//! 
//! 測試垃圾訊息檢測、重複訊息識別、連結安全檢測、洗版行為檢測

use droas_bot::protection::{
    pattern_recognition::{
        PatternRecognizer, SpamScore, SafetyResult, FloodingResult,
        DuplicateDetectionResult, LinkSafetyStatus
    },
    Message, MessageContext, ProtectionLevel, ViolationType,
};
use chrono::{Utc, Duration};
use std::sync::Arc;
use tokio;

/// 創建測試用訊息
fn create_test_message(content: &str, author_id: &str) -> Message {
    Message {
        id: uuid::Uuid::new_v4().to_string(),
        author_id: author_id.to_string(),
        guild_id: "test_guild".to_string(),
        channel_id: "test_channel".to_string(),
        content: content.to_string(),
        timestamp: Utc::now(),
        attachments: vec![],
        embeds: vec![],
        mentions: vec![],
    }
}

/// 創建訊息歷史
fn create_message_history(messages: Vec<(&str, &str)>) -> Vec<Message> {
    messages
        .into_iter()
        .enumerate()
        .map(|(i, (content, author))| {
            let mut msg = create_test_message(content, author);
            msg.timestamp = Utc::now() - Duration::seconds(60 - i as i64);
            msg
        })
        .collect()
}

#[tokio::test]
async fn test_spam_detection_basic() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試明顯的垃圾訊息
    let spam_messages = vec![
        "💰💰💰 FREE MONEY CLICK HERE 💰💰💰",
        "VISIT WWW.SCAM-SITE.COM FOR FREE PRIZES!!!",
        "BUY NOW!!! LIMITED OFFER!!! ONLY $9.99!!!",
        "JOIN MY SERVER discord.gg/spam discord.gg/spam discord.gg/spam",
    ];
    
    for content in spam_messages {
        let msg = create_test_message(content, "spammer");
        let score = recognizer.detect_spam(&msg.content).await.unwrap();
        
        assert!(
            score.score > 0.7,
            "垃圾訊息 '{}' 應該有高分數，但得到 {}",
            content,
            score.score
        );
        assert_eq!(score.violation_type, Some(ViolationType::Spam));
    }
}

#[tokio::test]
async fn test_spam_detection_normal_messages() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試正常訊息
    let normal_messages = vec![
        "大家好，今天天氣真好",
        "有人要一起玩遊戲嗎？",
        "我覺得這個想法很不錯",
        "謝謝你的幫助！",
    ];
    
    for content in normal_messages {
        let msg = create_test_message(content, "normal_user");
        let score = recognizer.detect_spam(&msg.content).await.unwrap();
        
        assert!(
            score.score < 0.3,
            "正常訊息 '{}' 應該有低分數，但得到 {}",
            content,
            score.score
        );
        assert_eq!(score.violation_type, None);
    }
}

#[tokio::test]
async fn test_duplicate_message_detection() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試完全相同的訊息
    let messages = create_message_history(vec![
        ("這是一條測試訊息", "user1"),
        ("這是一條測試訊息", "user1"),
        ("這是一條測試訊息", "user1"),
    ]);
    
    let result = recognizer.detect_duplicates(&messages).await.unwrap();
    
    assert!(result.has_duplicates);
    assert_eq!(result.duplicate_count, 3);
    assert!(result.similarity_score > 0.95);
}

#[tokio::test]
async fn test_similar_message_detection() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試相似但不完全相同的訊息
    let messages = create_message_history(vec![
        ("大家來加入我的伺服器！", "user1"),
        ("大家快來加入我的伺服器！", "user1"),
        ("大家趕快來加入我的伺服器！", "user1"),
    ]);
    
    let result = recognizer.detect_duplicates(&messages).await.unwrap();
    
    assert!(result.has_duplicates);
    assert!(result.similarity_score > 0.7);
}

#[tokio::test]
async fn test_flooding_detection() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 創建洗版訊息（短時間內大量訊息）
    let mut messages = vec![];
    let now = Utc::now();
    
    for i in 0..10 {
        let mut msg = create_test_message(&format!("訊息 {}", i), "flooder");
        msg.timestamp = now - Duration::milliseconds(100 * i);
        messages.push(msg);
    }
    
    let result = recognizer.detect_flooding(&messages).await.unwrap();
    
    assert!(result.is_flooding);
    assert!(result.messages_per_second > 5.0);
    assert_eq!(result.violation_type, ViolationType::Flooding);
}

#[tokio::test]
async fn test_normal_message_rate() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 創建正常速率的訊息
    let mut messages = vec![];
    let now = Utc::now();
    
    for i in 0..5 {
        let mut msg = create_test_message(&format!("訊息 {}", i), "normal_user");
        msg.timestamp = now - Duration::seconds(10 * i);
        messages.push(msg);
    }
    
    let result = recognizer.detect_flooding(&messages).await.unwrap();
    
    assert!(!result.is_flooding);
    assert!(result.messages_per_second < 1.0);
}

#[tokio::test]
async fn test_unsafe_link_detection() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試可疑連結
    let unsafe_links = vec![
        "http://bit.ly/suspicious",
        "https://tinyurl.com/scam",
        "http://192.168.1.1/admin",
        "https://phishing-site.tk",
        "http://suspicious-download.exe",
    ];
    
    for link in unsafe_links {
        let result = recognizer.check_link_safety(&[link.to_string()]).await.unwrap();
        
        assert_eq!(
            result.status,
            LinkSafetyStatus::Unsafe,
            "連結 '{}' 應該被標記為不安全",
            link
        );
        assert!(result.risk_score > 0.7);
    }
}

#[tokio::test]
async fn test_safe_link_detection() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試安全連結
    let safe_links = vec![
        "https://discord.com",
        "https://github.com/user/repo",
        "https://www.youtube.com/watch?v=123",
        "https://wikipedia.org/wiki/Article",
    ];
    
    for link in safe_links {
        let result = recognizer.check_link_safety(&[link.to_string()]).await.unwrap();
        
        assert_eq!(
            result.status,
            LinkSafetyStatus::Safe,
            "連結 '{}' 應該被標記為安全",
            link
        );
        assert!(result.risk_score < 0.3);
    }
}

#[tokio::test]
async fn test_mixed_violations() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試包含多種違規的訊息
    let msg = create_test_message(
        "💰💰💰 FREE MONEY!!! CLICK HERE http://scam.site 💰💰💰",
        "scammer"
    );
    
    // 檢測垃圾訊息
    let spam_score = recognizer.detect_spam(&msg.content).await.unwrap();
    assert!(spam_score.score > 0.8);
    
    // 檢測不安全連結
    let links = vec!["http://scam.site".to_string()];
    let link_result = recognizer.check_link_safety(&links).await.unwrap();
    assert_eq!(link_result.status, LinkSafetyStatus::Unsafe);
}

#[tokio::test]
async fn test_protection_level_sensitivity() {
    // 測試不同防護等級下的檢測靈敏度
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    let borderline_msg = "Join my awesome server! discord.gg/myserver";
    
    // 寬鬆模式
    let score_loose = recognizer
        .detect_spam_with_level(&borderline_msg, ProtectionLevel::Loose)
        .await
        .unwrap();
    
    // 中等模式
    let score_medium = recognizer
        .detect_spam_with_level(&borderline_msg, ProtectionLevel::Medium)
        .await
        .unwrap();
    
    // 嚴格模式
    let score_strict = recognizer
        .detect_spam_with_level(&borderline_msg, ProtectionLevel::Strict)
        .await
        .unwrap();
    
    // 嚴格模式應該有更高的分數
    assert!(score_strict.score > score_medium.score);
    assert!(score_medium.score > score_loose.score);
}

#[tokio::test]
async fn test_unicode_and_special_chars() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 測試包含特殊字符和 Unicode 的垃圾訊息
    let unicode_spam = vec![
        "🔥🔥🔥 ＦＲ𝐄𝐄 Ｍ𝐎𝐍𝐄𝐘 🔥🔥🔥",
        "₿₿₿ ＣＲＹＰＴＯ ＧＩＶＥＡＷＡＹ ₿₿₿",
        "【【【 ＣＬＩＣＫ ＨＥＲＥ 】】】",
    ];
    
    for content in unicode_spam {
        let msg = create_test_message(content, "spammer");
        let score = recognizer.detect_spam(&msg.content).await.unwrap();
        
        assert!(
            score.score > 0.6,
            "Unicode 垃圾訊息應該被檢測到: '{}'",
            content
        );
    }
}

#[tokio::test]
async fn test_performance_large_history() {
    let recognizer = Arc::new(MockPatternRecognizer::new());
    
    // 創建大量訊息歷史測試性能
    let mut messages = vec![];
    for i in 0..1000 {
        messages.push(create_test_message(
            &format!("訊息內容 {}", i),
            &format!("user_{}", i % 10),
        ));
    }
    
    let start = tokio::time::Instant::now();
    let result = recognizer.detect_duplicates(&messages).await.unwrap();
    let elapsed = start.elapsed();
    
    // 確保處理 1000 條訊息在合理時間內完成
    assert!(
        elapsed.as_millis() < 100,
        "處理 1000 條訊息耗時 {:?}，超過 100ms",
        elapsed
    );
}

// Mock 實現用於測試
struct MockPatternRecognizer;

impl MockPatternRecognizer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PatternRecognizer for MockPatternRecognizer {
    async fn detect_spam(&self, content: &str) -> Result<SpamScore, Box<dyn std::error::Error>> {
        // 簡單的垃圾訊息檢測邏輯
        let mut score = 0.0;
        
        // 檢查關鍵詞
        let spam_keywords = ["free", "money", "click here", "limited offer", "buy now", "prize", "💰"];
        for keyword in &spam_keywords {
            if content.to_lowercase().contains(keyword) {
                score += 0.2;
            }
        }
        
        // 檢查大寫字母比例
        let uppercase_ratio = content.chars().filter(|c| c.is_uppercase()).count() as f32 
            / content.len() as f32;
        if uppercase_ratio > 0.5 {
            score += 0.3;
        }
        
        // 檢查感嘆號數量
        let exclamation_count = content.matches('!').count();
        if exclamation_count > 3 {
            score += 0.2;
        }
        
        Ok(SpamScore {
            score: score.min(1.0),
            violation_type: if score > 0.7 { Some(ViolationType::Spam) } else { None },
            confidence: 0.85,
        })
    }
    
    async fn detect_duplicates(&self, messages: &[Message]) -> Result<DuplicateDetectionResult, Box<dyn std::error::Error>> {
        // 實現重複檢測邏輯
        Ok(DuplicateDetectionResult {
            has_duplicates: false,
            duplicate_count: 0,
            similarity_score: 0.0,
        })
    }
    
    async fn detect_flooding(&self, messages: &[Message]) -> Result<FloodingResult, Box<dyn std::error::Error>> {
        // 實現洗版檢測邏輯
        Ok(FloodingResult {
            is_flooding: false,
            messages_per_second: 0.0,
            violation_type: ViolationType::Flooding,
        })
    }
    
    async fn check_link_safety(&self, links: &[String]) -> Result<SafetyResult, Box<dyn std::error::Error>> {
        // 實現連結安全檢測
        Ok(SafetyResult {
            status: LinkSafetyStatus::Unknown,
            risk_score: 0.0,
            details: vec![],
        })
    }
    
    async fn detect_spam_with_level(
        &self,
        content: &str,
        level: ProtectionLevel,
    ) -> Result<SpamScore, Box<dyn std::error::Error>> {
        let mut base_score = self.detect_spam(content).await?;
        
        // 根據防護等級調整分數
        match level {
            ProtectionLevel::Strict => base_score.score *= 1.3,
            ProtectionLevel::Medium => base_score.score *= 1.0,
            ProtectionLevel::Loose => base_score.score *= 0.7,
        }
        
        base_score.score = base_score.score.min(1.0);
        Ok(base_score)
    }
}
