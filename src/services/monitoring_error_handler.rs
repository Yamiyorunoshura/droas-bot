use crate::error::{DiscordError, Result};
use crate::services::monitoring_config::MonitoringConfig;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn, info};

/// 監控錯誤處理器
///
/// 與現有 Error Handling Framework 整合，提供一致的錯誤處理體驗
pub struct MonitoringErrorHandler {
    /// 監控配置
    config: Arc<MonitoringConfig>,

    /// 錯誤統計
    error_stats: Arc<RwLock<ErrorStats>>,

    /// 錯誤警報狀態
    alert_states: Arc<RwLock<std::collections::HashMap<String, AlertState>>>,
}

/// 錯誤統計信息
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    /// 總錯誤數
    pub total_errors: u64,

    /// 各類型錯誤統計
    pub error_counts: std::collections::HashMap<String, u64>,

    /// 最後錯誤時間
    pub last_error_time: Option<std::time::Instant>,

    /// 錯誤率（每分鐘）
    pub error_rate_per_minute: f64,

    /// 連續錯誤計數器
    pub consecutive_errors: std::collections::HashMap<String, u32>,
}

/// 警報狀態
#[derive(Debug)]
pub struct AlertState {
    /// 是否處於警報狀態
    pub is_active: bool,

    /// 警報開始時間
    pub start_time: Option<std::time::Instant>,

    /// 警報觸發次數
    pub trigger_count: u32,

    /// 最後警報時間
    pub last_alert_time: Option<std::time::Instant>,

    /// 警報級別
    pub severity: AlertSeverity,
}

/// 警報級別
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    /// 信息級別
    Info,
    /// 警告級別
    Warning,
    /// 錯誤級別
    Error,
    /// 嚴重錯誤級別
    Critical,
}

impl MonitoringErrorHandler {
    /// 創建新的監控錯誤處理器
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config: Arc::new(config),
            error_stats: Arc::new(RwLock::new(ErrorStats::default())),
            alert_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 處理錯誤
    pub async fn handle_error(&self, error: &DiscordError) -> Result<()> {
        // 記錄錯誤統計
        self.record_error_stats(error).await;

        // 檢查是否需要觸發警報
        if self.should_trigger_alert(error).await {
            self.trigger_alert(error).await?;
        }

        // 根據錯誤類型決定處理策略
        match error {
            DiscordError::DatabaseConnectionError(_) => {
                self.handle_database_error(error).await
            }
            DiscordError::ConnectionError(_) => {
                self.handle_connection_error(error).await
            }
            DiscordError::ValidationError(_) => {
                self.handle_validation_error(error).await
            }
            _ => {
                info!("處理一般錯誤: {:?}", error);
                Ok(())
            }
        }
    }

    /// 記錄錯誤統計
    async fn record_error_stats(&self, error: &DiscordError) {
        let mut stats = self.error_stats.write().await;

        stats.total_errors += 1;
        stats.last_error_time = Some(std::time::Instant::now());

        let error_type = self.get_error_type(error);
        *stats.error_counts.entry(error_type.clone()).or_insert(0) += 1;
        *stats.consecutive_errors.entry(error_type.clone()).or_insert(0) += 1;

        // 計算錯誤率（簡化版本，實際應該基於時間窗口）
        stats.error_rate_per_minute = stats.total_errors as f64 / 60.0;

        drop(stats);

        tracing::error!("記錄錯誤統計: {} (類型: {})", error, error_type);
    }

    /// 檢查是否應該觸發警報
    async fn should_trigger_alert(&self, error: &DiscordError) -> bool {
        let error_type = self.get_error_type(error);
        let stats = self.error_stats.read().await;
        let alert_states = self.alert_states.read().await;

        // 檢查連續錯誤次數
        if let Some(&consecutive_count) = stats.consecutive_errors.get(&error_type) {
            let threshold = self.get_error_threshold(&error_type);
            if consecutive_count >= threshold {
                return true;
            }
        }

        // 檢查錯誤率
        if stats.error_rate_per_minute > self.config.alert_thresholds.error_rate_threshold_percent {
            return true;
        }

        // 檢查是否已經處於警報狀態
        if let Some(alert_state) = alert_states.get(&error_type) {
            if alert_state.is_active {
                return false; // 已經處於警報狀態，避免重複觸發
            }
        }

        false
    }

    /// 觸發警報
    async fn trigger_alert(&self, error: &DiscordError) -> Result<()> {
        let error_type = self.get_error_type(error);
        let severity = self.determine_alert_severity(error);

        let mut alert_states = self.alert_states.write().await;
        let alert_state = alert_states.entry(error_type.clone()).or_insert_with(|| AlertState {
            is_active: false,
            start_time: None,
            trigger_count: 0,
            last_alert_time: None,
            severity: AlertSeverity::Info,
        });

        alert_state.is_active = true;
        alert_state.start_time = Some(std::time::Instant::now());
        alert_state.trigger_count += 1;
        alert_state.last_alert_time = Some(std::time::Instant::now());
        alert_state.severity = severity.clone();

        drop(alert_states);

        // 記錄警報
        match severity {
            AlertSeverity::Info => info!("觸發資訊警報: {} - {}", error_type, error),
            AlertSeverity::Warning => warn!("觸發警告警報: {} - {}", error_type, error),
            AlertSeverity::Error => error!("觸發錯誤警報: {} - {}", error_type, error),
            AlertSeverity::Critical => error!("觸發嚴重錯誤警報: {} - {}", error_type, error),
        }

        // 這裡可以添加更多警報處理邏輯，例如：
        // - 發送通知到外部系統
        // - 記錄到專門的警報日誌
        // - 觸發自動恢復機制

        Ok(())
    }

    /// 處理資料庫錯誤
    async fn handle_database_error(&self, error: &DiscordError) -> Result<()> {
        error!("處理資料庫錯誤: {:?}", error);

        // 檢查是否需要重試連接
        if matches!(error, DiscordError::DatabaseConnectionError(_)) {
            warn!("檢測到資料庫連接問題，可能需要重試連接");
            // 這裡可以添加重試邏輯
        }

        Ok(())
    }

    /// 處理連接錯誤
    async fn handle_connection_error(&self, error: &DiscordError) -> Result<()> {
        error!("處理連接錯誤: {:?}", error);

        // 檢查是否是 Discord API 連接問題
        if matches!(error, DiscordError::ConnectionError(_)) {
            warn!("檢測到 Discord API 連接問題");
            // 這裡可以添加重連邏輯
        }

        Ok(())
    }

    /// 處理驗證錯誤
    async fn handle_validation_error(&self, error: &DiscordError) -> Result<()> {
        info!("處理驗證錯誤: {:?}", error);

        // 驗證錯誤通常不需要特殊處理，記錄即可
        Ok(())
    }

    /// 獲取錯誤類型
    fn get_error_type(&self, error: &DiscordError) -> String {
        match error {
            DiscordError::DatabaseConnectionError(_) => "database_connection".to_string(),
            DiscordError::DatabaseQueryError(_) => "database_query".to_string(),
            DiscordError::ConnectionError(_) => "discord_connection".to_string(),
            DiscordError::ValidationError(_) => "validation".to_string(),
            DiscordError::InsufficientBalance(_) => "insufficient_balance".to_string(),
            DiscordError::InvalidAmount(_) => "invalid_amount".to_string(),
            DiscordError::CommandError(_) => "command".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// 獲取錯誤閾值
    fn get_error_threshold(&self, error_type: &str) -> u32 {
        match error_type {
            "database_connection" => self.config.alert_thresholds.database_connection_failure_threshold,
            "discord_connection" => self.config.alert_thresholds.discord_api_failure_threshold,
            "database_query" => self.config.alert_thresholds.database_connection_failure_threshold,
            _ => 3, // 預設閾值
        }
    }

    /// 確定警報嚴重程度
    fn determine_alert_severity(&self, error: &DiscordError) -> AlertSeverity {
        match error {
            DiscordError::DatabaseConnectionError(_) => AlertSeverity::Critical,
            DiscordError::ConnectionError(_) => AlertSeverity::Error,
            DiscordError::ValidationError(_) => AlertSeverity::Warning,
            DiscordError::InsufficientBalance(_) => AlertSeverity::Info,
            _ => AlertSeverity::Warning,
        }
    }

    /// 獲取錯誤統計
    pub async fn get_error_stats(&self) -> ErrorStats {
        self.error_stats.read().await.clone()
    }

    /// 獲取警報狀態
    pub async fn get_alert_states(&self) -> std::collections::HashMap<String, AlertState> {
        self.alert_states.read().await.clone()
    }

    /// 重置錯誤統計
    pub async fn reset_error_stats(&self) {
        let mut stats = self.error_stats.write().await;
        *stats = ErrorStats::default();
        info!("錯誤統計已重置");
    }

    /// 重置特定類型的警報狀態
    pub async fn reset_alert_state(&self, error_type: &str) {
        let mut alert_states = self.alert_states.write().await;
        if let Some(alert_state) = alert_states.get_mut(error_type) {
            alert_state.is_active = false;
            alert_state.start_time = None;
            alert_state.trigger_count = 0;
            alert_state.last_alert_time = None;
            info!("警報狀態已重置: {}", error_type);
        }
    }

    /// 重置所有警報狀態
    pub async fn reset_all_alert_states(&self) {
        let mut alert_states = self.alert_states.write().await;
        for alert_state in alert_states.values_mut() {
            alert_state.is_active = false;
            alert_state.start_time = None;
            alert_state.trigger_count = 0;
            alert_state.last_alert_time = None;
        }
        info!("所有警報狀態已重置");
    }
}

impl Clone for AlertState {
    fn clone(&self) -> Self {
        Self {
            is_active: self.is_active,
            start_time: self.start_time,
            trigger_count: self.trigger_count,
            last_alert_time: self.last_alert_time,
            severity: self.severity.clone(),
        }
    }
}