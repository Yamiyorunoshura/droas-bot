use crate::database::schema::GuildConfigService;
use crate::handlers::welcome::WelcomeHandler;
use anyhow::Result;
use std::collections::HashMap;
use std::option::Option;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// 事件處理結果
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// 處理成功
    Success,
    /// 被跳過（如重複事件）
    Skipped(String),
    /// 處理失敗但不是致命錯誤
    Failed(String),
}

/// 測試用的簡化 Discord 事件結構
#[derive(Debug, Clone)]
pub struct TestMemberJoinEvent {
    pub guild_id: u64,
    pub user_id: u64,
    pub username: String,
    pub timestamp: Instant,
}

/// 事件去重緩存項目
#[derive(Debug, Clone)]
struct DeduplicationEntry {
    timestamp: Instant,
    processed: bool,
}

/// Discord 事件處理器
///
/// 負責處理所有 Discord 事件，包括 GUILD_MEMBER_ADD 事件的接收、
/// 驗證、去重和路由到相應的處理程序。
#[derive(Clone)]
pub struct EventHandler {
    pub guild_service: GuildConfigService,
    welcome_handler: WelcomeHandler,
    /// 事件去重緩存 (guild_id:user_id -> 處理記錄)
    deduplication_cache: Arc<Mutex<HashMap<String, DeduplicationEntry>>>,
    /// 去重緩存的有效期限（5分鐘）
    dedup_ttl: Duration,
    /// 監控器（可選）
    monitor: Option<Arc<crate::discord::monitoring::DiscordMonitor>>,
}

impl EventHandler {
    /// 創建新的事件處理器
    pub fn new(guild_service: GuildConfigService, welcome_handler: WelcomeHandler) -> Self {
        Self {
            guild_service,
            welcome_handler,
            deduplication_cache: Arc::new(Mutex::new(HashMap::new())),
            dedup_ttl: Duration::from_secs(300), // 5分鐘
            monitor: None,
        }
    }

    /// 創建帶監控器的事件處理器
    pub fn with_monitor(
        guild_service: GuildConfigService,
        welcome_handler: WelcomeHandler,
        monitor: Arc<crate::discord::monitoring::DiscordMonitor>,
    ) -> Self {
        Self {
            guild_service,
            welcome_handler,
            deduplication_cache: Arc::new(Mutex::new(HashMap::new())),
            dedup_ttl: Duration::from_secs(300), // 5分鐘
            monitor: Some(monitor),
        }
    }

    /// 處理 GUILD_MEMBER_ADD 事件
    ///
    /// 此方法實現完整的事件處理流水線：
    /// 1. 事件驗證
    /// 2. 去重檢查
    /// 3. 委託給歡迎處理器
    /// 4. 性能監控
    pub async fn handle_member_join_event(
        &self,
        event: &TestMemberJoinEvent,
    ) -> Result<EventResult> {
        let start_time = Instant::now();
        debug!(
            "開始處理成員加入事件: guild_id={}, user_id={}",
            event.guild_id, event.user_id
        );

        // 1. 事件驗證
        if !self.validate_event(event) {
            warn!(
                "事件驗證失敗: guild_id={}, user_id={}",
                event.guild_id, event.user_id
            );

            // 記錄失敗事件
            let processing_time = start_time.elapsed();
            if let Some(ref monitor) = self.monitor {
                monitor
                    .record_event_processing(
                        "member_join",
                        processing_time.as_millis() as u64,
                        "failed",
                    )
                    .await;
            }

            return Ok(EventResult::Failed("事件驗證失敗".to_string()));
        }

        // 2. 去重檢查
        if let Some(reason) = self.check_duplication(event).await {
            debug!("事件被去重跳過: {}", reason);

            // 記錄重複事件
            let processing_time = start_time.elapsed();
            if let Some(ref monitor) = self.monitor {
                monitor
                    .record_event_processing(
                        "member_join",
                        processing_time.as_millis() as u64,
                        "duplicate",
                    )
                    .await;
            }

            return Ok(EventResult::Skipped(reason));
        }

        // 3. 使用 timeout 確保處理時間不超過 500ms
        let processing_result = timeout(
            Duration::from_millis(500),
            self.process_member_join_internal(event),
        )
        .await;

        let processing_time = start_time.elapsed();

        match processing_result {
            Ok(Ok(_)) => {
                info!(
                    "成員加入事件處理完成: guild_id={}, user_id={}, 處理時間={}ms",
                    event.guild_id,
                    event.user_id,
                    processing_time.as_millis()
                );

                // 記錄到去重緩存
                self.record_processed_event(event).await;

                // 記錄成功事件
                if let Some(ref monitor) = self.monitor {
                    monitor
                        .record_event_processing(
                            "member_join",
                            processing_time.as_millis() as u64,
                            "success",
                        )
                        .await;
                }

                Ok(EventResult::Success)
            }
            Ok(Err(e)) => {
                error!(
                    "成員加入事件處理失敗: guild_id={}, user_id={}, error={}, 處理時間={}ms",
                    event.guild_id,
                    event.user_id,
                    e,
                    processing_time.as_millis()
                );

                // 記錄失敗事件
                if let Some(ref monitor) = self.monitor {
                    monitor
                        .record_event_processing(
                            "member_join",
                            processing_time.as_millis() as u64,
                            "failed",
                        )
                        .await;
                }

                Ok(EventResult::Failed(e.to_string()))
            }
            Err(_timeout_error) => {
                error!(
                    "成員加入事件處理超時: guild_id={}, user_id={}, 處理時間={}ms",
                    event.guild_id,
                    event.user_id,
                    processing_time.as_millis()
                );

                // 記錄超時事件
                if let Some(ref monitor) = self.monitor {
                    monitor
                        .record_event_processing(
                            "member_join",
                            processing_time.as_millis() as u64,
                            "failed",
                        )
                        .await;
                }

                Ok(EventResult::Failed("事件處理超時".to_string()))
            }
        }
    }

    /// 驗證事件的有效性
    ///
    /// 檢查事件的基本格式和內容是否有效
    pub fn validate_event(&self, event: &TestMemberJoinEvent) -> bool {
        // 檢查 Guild ID 有效性
        if event.guild_id == 0 {
            debug!("無效的 Guild ID: 0");
            return false;
        }

        // 檢查 User ID 有效性
        if event.user_id == 0 {
            debug!("無效的 User ID: 0");
            return false;
        }

        // 檢查用戶名有效性
        if event.username.is_empty() {
            debug!("無效的用戶名: 空字符串");
            return false;
        }

        // 檢查用戶名長度（Discord 用戶名限制）
        if event.username.len() > 32 {
            debug!("無效的用戶名: 超過32個字符");
            return false;
        }

        // 檢查時間戳有效性（不能是未來時間）
        if event.timestamp > Instant::now() {
            debug!("無效的時間戳: 未來時間");
            return false;
        }

        // 檢查時間戳不能太舊（超過24小時）
        if event.timestamp.elapsed() > Duration::from_secs(24 * 60 * 60) {
            debug!("無效的時間戳: 超過24小時");
            return false;
        }

        true
    }

    /// 檢查事件是否重複
    async fn check_duplication(&self, event: &TestMemberJoinEvent) -> Option<String> {
        let cache_key = format!("{}:{}", event.guild_id, event.user_id);

        let mut cache = self.deduplication_cache.lock().unwrap();

        // 清理過期的緩存條目
        self.cleanup_expired_cache(&mut cache);

        // 檢查是否存在相同的事件
        if let Some(entry) = cache.get(&cache_key) {
            if entry.processed && entry.timestamp.elapsed() < self.dedup_ttl {
                return Some(format!(
                    "重複事件: guild_id={}, user_id={}",
                    event.guild_id, event.user_id
                ));
            }
        }

        // 記錄事件開始處理（防止並發重複）
        cache.insert(
            cache_key,
            DeduplicationEntry {
                timestamp: event.timestamp,
                processed: false,
            },
        );

        None
    }

    /// 記錄事件已成功處理
    async fn record_processed_event(&self, event: &TestMemberJoinEvent) {
        let cache_key = format!("{}:{}", event.guild_id, event.user_id);

        let mut cache = self.deduplication_cache.lock().unwrap();
        cache.insert(
            cache_key,
            DeduplicationEntry {
                timestamp: event.timestamp,
                processed: true,
            },
        );
    }

    /// 清理過期的緩存條目
    fn cleanup_expired_cache(&self, cache: &mut HashMap<String, DeduplicationEntry>) {
        let now = Instant::now();
        cache.retain(|_key, entry| now.duration_since(entry.timestamp) < self.dedup_ttl);
    }

    /// 內部成員加入處理邏輯
    async fn process_member_join_internal(&self, event: &TestMemberJoinEvent) -> Result<()> {
        debug!(
            "處理成員加入事件: guild_id={}, user_id={}",
            event.guild_id, event.user_id
        );

        // 查詢公會配置
        let guild_config = self
            .guild_service
            .get_guild_config(&event.guild_id.to_string())
            .await?;

        if let Some(config) = guild_config {
            debug!(
                "找到公會配置: welcome_channel_id={}",
                config.welcome_channel_id
            );

            // 委託給歡迎處理器
            self.welcome_handler
                .handle_member_join(event.guild_id, event.user_id)
                .await?;

            info!(
                "歡迎訊息處理完成: guild_id={}, user_id={}",
                event.guild_id, event.user_id
            );
        } else {
            warn!("未找到公會配置，跳過歡迎訊息: guild_id={}", event.guild_id);
        }

        Ok(())
    }

    /// 獲取去重緩存統計信息（用於監控和調試）
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.deduplication_cache.lock().unwrap();
        let total_entries = cache.len();
        let processed_entries = cache.values().filter(|entry| entry.processed).count();

        (total_entries, processed_entries)
    }

    /// 清空去重緩存（主要用於測試）
    pub fn clear_cache(&self) {
        let mut cache = self.deduplication_cache.lock().unwrap();
        cache.clear();
    }

    /// 手動觸發緩存清理（用於定期維護）
    pub fn cleanup_cache(&self) {
        let mut cache = self.deduplication_cache.lock().unwrap();
        self.cleanup_expired_cache(&mut cache);
        debug!("緩存清理完成，剩餘條目數: {}", cache.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema::{GuildConfig, GuildConfigService};
    use chrono::Utc;
    use tempfile::NamedTempFile;

    async fn create_test_services() -> (GuildConfigService, NamedTempFile) {
        let temp_file = NamedTempFile::new().expect("無法創建臨時檔案");
        let database_url = format!("sqlite://{}", temp_file.path().display());

        let pool = sqlx::SqlitePool::connect(&database_url)
            .await
            .expect("無法連接測試資料庫");

        // 創建測試表格
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS guild_config (
                guild_id TEXT PRIMARY KEY,
                welcome_channel_id TEXT NOT NULL,
                background_ref TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("無法創建測試表格");

        let guild_service = GuildConfigService::new(pool);

        (guild_service, temp_file)
    }

    #[tokio::test]
    async fn test_event_validation() {
        let (guild_service, _temp_file) = create_test_services().await;
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // 測試有效事件
        let valid_event = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "ValidUser".to_string(),
            timestamp: Instant::now(),
        };
        assert!(event_handler.validate_event(&valid_event));

        // 測試無效事件
        let invalid_events = vec![
            TestMemberJoinEvent {
                guild_id: 0,
                user_id: 555666777,
                username: "InvalidUser".to_string(),
                timestamp: Instant::now(),
            },
            TestMemberJoinEvent {
                guild_id: 123456789,
                user_id: 0,
                username: "InvalidUser".to_string(),
                timestamp: Instant::now(),
            },
            TestMemberJoinEvent {
                guild_id: 123456789,
                user_id: 555666777,
                username: "".to_string(),
                timestamp: Instant::now(),
            },
        ];

        for invalid_event in invalid_events {
            assert!(!event_handler.validate_event(&invalid_event));
        }
    }

    #[tokio::test]
    async fn test_deduplication() {
        let (guild_service, _temp_file) = create_test_services().await;
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        let test_event = TestMemberJoinEvent {
            guild_id: 123456789,
            user_id: 555666777,
            username: "TestUser".to_string(),
            timestamp: Instant::now(),
        };

        // 第一次檢查應該沒有重複
        let dup_check1 = event_handler.check_duplication(&test_event).await;
        assert!(dup_check1.is_none());

        // 標記為已處理
        event_handler.record_processed_event(&test_event).await;

        // 再次檢查應該檢測到重複
        let dup_check2 = event_handler.check_duplication(&test_event).await;
        assert!(dup_check2.is_some());
        assert!(dup_check2.unwrap().contains("重複事件"));
    }

    #[tokio::test]
    async fn test_cache_management() {
        let (guild_service, _temp_file) = create_test_services().await;
        let welcome_handler = WelcomeHandler::new();
        let event_handler = EventHandler::new(guild_service, welcome_handler);

        // 添加一些測試事件到緩存
        for i in 0..3 {
            let test_event = TestMemberJoinEvent {
                guild_id: 123456789,
                user_id: 555666777 + i,
                username: format!("TestUser{}", i),
                timestamp: Instant::now(),
            };

            event_handler.check_duplication(&test_event).await;
            event_handler.record_processed_event(&test_event).await;
        }

        // 檢查緩存統計
        let (total, processed) = event_handler.get_cache_stats();
        assert_eq!(total, 3);
        assert_eq!(processed, 3);

        // 測試緩存清理
        event_handler.clear_cache();
        let (total_after_clear, processed_after_clear) = event_handler.get_cache_stats();
        assert_eq!(total_after_clear, 0);
        assert_eq!(processed_after_clear, 0);
    }
}
