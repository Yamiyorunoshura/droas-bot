//! Discord API 監控和指標模組
//!
//! 此模組提供全面的監控功能，包括：
//! - 速率限制指標監控
//! - API 調用統計
//! - 事件處理指標
//! - 系統健康檢查
//! - 性能監控

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 速率限制指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMetrics {
    /// 總請求數
    pub total_requests: u64,
    /// 速率限制觸發次數
    pub rate_limit_hits: u64,
    /// 全域速率限制觸發次數
    pub global_rate_limit_hits: u64,
    /// 平均等待時間（毫秒）
    pub average_wait_time_ms: f64,
    /// 最大等待時間（毫秒）
    pub max_wait_time_ms: u64,
    /// 最近更新時間
    pub last_updated: std::time::SystemTime,
}

impl Default for RateLimitMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            rate_limit_hits: 0,
            global_rate_limit_hits: 0,
            average_wait_time_ms: 0.0,
            max_wait_time_ms: 0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

/// API 調用指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    /// 成功調用數
    pub successful_calls: u64,
    /// 失敗調用數
    pub failed_calls: u64,
    /// 超時調用數
    pub timeout_calls: u64,
    /// 平均響應時間（毫秒）
    pub average_response_time_ms: f64,
    /// 成功率（百分比）
    pub success_rate: f64,
    /// 最近更新時間
    pub last_updated: std::time::SystemTime,
}

impl Default for ApiMetrics {
    fn default() -> Self {
        Self {
            successful_calls: 0,
            failed_calls: 0,
            timeout_calls: 0,
            average_response_time_ms: 0.0,
            success_rate: 100.0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

/// 事件處理指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    /// 總事件數
    pub total_events: u64,
    /// 成功處理事件數
    pub processed_events: u64,
    /// 重複事件數
    pub duplicate_events: u64,
    /// 失敗事件數
    pub failed_events: u64,
    /// 平均處理時間（毫秒）
    pub average_processing_time_ms: f64,
    /// 吞吐量（事件/秒）
    pub throughput: f64,
    /// 最近更新時間
    pub last_updated: std::time::SystemTime,
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            total_events: 0,
            processed_events: 0,
            duplicate_events: 0,
            failed_events: 0,
            average_processing_time_ms: 0.0,
            throughput: 0.0,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

/// 系統健康狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// 總體健康狀態
    pub overall_health: HealthStatus,
    /// 速率限制健康狀態
    pub rate_limit_health: HealthStatus,
    /// API 健康狀態
    pub api_health: HealthStatus,
    /// 事件處理健康狀態
    pub event_processing_health: HealthStatus,
    /// 檢查時間
    pub check_time: std::time::SystemTime,
}

/// 健康狀態枚舉
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 警告
    Warning,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

impl HealthStatus {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            HealthStatus::Healthy
        } else if score >= 0.7 {
            HealthStatus::Warning
        } else if score > 0.0 {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Unknown
        }
    }
}

/// Discord API 監控器
///
/// 負責收集和維護所有 Discord API 相關的監控指標
pub struct DiscordMonitor {
    /// 速率限制指標
    rate_limit_metrics: Arc<RwLock<RateLimitMetrics>>,
    /// API 指標
    api_metrics: Arc<RwLock<ApiMetrics>>,
    /// 事件處理指標
    event_metrics: Arc<RwLock<EventMetrics>>,
    /// 指標收集開始時間
    start_time: Instant,
    /// 總等待時間（用於計算平均值）
    total_wait_time_ms: AtomicU64,
    /// 總響應時間（用於計算平均值）
    total_response_time_ms: AtomicU64,
    /// 總處理時間（用於計算平均值）
    total_processing_time_ms: AtomicU64,
}

impl DiscordMonitor {
    /// 創建新的監控器
    pub fn new() -> Self {
        Self {
            rate_limit_metrics: Arc::new(RwLock::new(RateLimitMetrics::default())),
            api_metrics: Arc::new(RwLock::new(ApiMetrics::default())),
            event_metrics: Arc::new(RwLock::new(EventMetrics::default())),
            start_time: Instant::now(),
            total_wait_time_ms: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
        }
    }

    /// 記錄速率限制事件
    ///
    /// # Arguments
    /// * `route` - API 路由
    /// * `wait_time_ms` - 等待時間（毫秒）
    /// * `is_global` - 是否為全域限制
    pub async fn record_rate_limit_event(&self, route: &str, wait_time_ms: u64, is_global: bool) {
        let mut metrics = self.rate_limit_metrics.write().await;

        metrics.total_requests += 1;
        metrics.rate_limit_hits += 1;

        if is_global {
            metrics.global_rate_limit_hits += 1;
        }

        // 更新最大等待時間
        if wait_time_ms > metrics.max_wait_time_ms {
            metrics.max_wait_time_ms = wait_time_ms;
        }

        // 更新總等待時間
        self.total_wait_time_ms.fetch_add(wait_time_ms, Ordering::Relaxed);

        // 重新計算平均等待時間
        metrics.average_wait_time_ms = self.total_wait_time_ms.load(Ordering::Relaxed) as f64 / metrics.rate_limit_hits as f64;

        metrics.last_updated = std::time::SystemTime::now();

        tracing::debug!(
            "記錄速率限制事件: route={}, wait_time={}ms, is_global={}, total_hits={}",
            route, wait_time_ms, is_global, metrics.rate_limit_hits
        );
    }

    /// 記錄 API 調用
    ///
    /// # Arguments
    /// * `endpoint` - API 端點
    /// * `response_time_ms` - 響應時間（毫秒）
    /// * `success` - 是否成功
    pub async fn record_api_call(&self, endpoint: &str, response_time_ms: u64, success: bool) {
        let mut metrics = self.api_metrics.write().await;

        metrics.successful_calls += u64::from(success);
        metrics.failed_calls += u64::from(!success);

        // 更新總響應時間
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);

        // 重新計算平均響應時間和成功率
        let total_calls = metrics.successful_calls + metrics.failed_calls;
        if total_calls > 0 {
            metrics.average_response_time_ms = self.total_response_time_ms.load(Ordering::Relaxed) as f64 / total_calls as f64;
            metrics.success_rate = (metrics.successful_calls as f64 / total_calls as f64) * 100.0;
        }

        metrics.last_updated = std::time::SystemTime::now();

        tracing::debug!(
            "記錄 API 調用: endpoint={}, response_time={}ms, success={}, success_rate={:.2}%",
            endpoint, response_time_ms, success, metrics.success_rate
        );
    }

    /// 記錄 API 超時
    pub async fn record_api_timeout(&self, endpoint: &str) {
        let mut metrics = self.api_metrics.write().await;
        metrics.timeout_calls += 1;
        metrics.last_updated = std::time::SystemTime::now();

        tracing::warn!("記錄 API 超時: endpoint={}, total_timeouts={}", endpoint, metrics.timeout_calls);
    }

    /// 記錄事件處理
    ///
    /// # Arguments
    /// * `event_type` - 事件類型
    /// * `processing_time_ms` - 處理時間（毫秒）
    /// * `result` - 處理結果（"success", "duplicate", "failed"）
    pub async fn record_event_processing(&self, event_type: &str, processing_time_ms: u64, result: &str) {
        let mut metrics = self.event_metrics.write().await;

        metrics.total_events += 1;

        match result {
            "success" => metrics.processed_events += 1,
            "duplicate" => metrics.duplicate_events += 1,
            "failed" => metrics.failed_events += 1,
            _ => {}
        }

        // 更新總處理時間
        self.total_processing_time_ms.fetch_add(processing_time_ms, Ordering::Relaxed);

        // 重新計算平均處理時間和吞吐量
        if metrics.processed_events > 0 {
            metrics.average_processing_time_ms = self.total_processing_time_ms.load(Ordering::Relaxed) as f64 / metrics.processed_events as f64;
        }

        // 計算吞吐量（事件/秒）
        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        if elapsed_seconds > 0.0 {
            metrics.throughput = metrics.total_events as f64 / elapsed_seconds;
        }

        metrics.last_updated = std::time::SystemTime::now();

        tracing::debug!(
            "記錄事件處理: event_type={}, processing_time={}ms, result={}, throughput={:.2} events/sec",
            event_type, processing_time_ms, result, metrics.throughput
        );
    }

    /// 獲取速率限制指標
    pub async fn get_rate_limit_metrics(&self) -> RateLimitMetrics {
        self.rate_limit_metrics.read().await.clone()
    }

    /// 獲取 API 指標
    pub async fn get_api_metrics(&self) -> ApiMetrics {
        self.api_metrics.read().await.clone()
    }

    /// 獲取事件處理指標
    pub async fn get_event_metrics(&self) -> EventMetrics {
        self.event_metrics.read().await.clone()
    }

    /// 獲取系統健康狀態
    pub async fn get_system_health(&self) -> SystemHealth {
        let rate_limit_metrics = self.get_rate_limit_metrics().await;
        let api_metrics = self.get_api_metrics().await;
        let event_metrics = self.get_event_metrics().await;

        // 計算各個子系統的健康分數
        let rate_limit_health_score = self.calculate_rate_limit_health_score(&rate_limit_metrics);
        let api_health_score = self.calculate_api_health_score(&api_metrics);
        let event_health_score = self.calculate_event_health_score(&event_metrics);

        // 計算總體健康分數（加權平均）
        let overall_health_score = (
            rate_limit_health_score * 0.3 +
            api_health_score * 0.4 +
            event_health_score * 0.3
        ).min(1.0).max(0.0);

        SystemHealth {
            overall_health: HealthStatus::from_score(overall_health_score),
            rate_limit_health: HealthStatus::from_score(rate_limit_health_score),
            api_health: HealthStatus::from_score(api_health_score),
            event_processing_health: HealthStatus::from_score(event_health_score),
            check_time: std::time::SystemTime::now(),
        }
    }

    /// 計算速率限制健康分數
    fn calculate_rate_limit_health_score(&self, metrics: &RateLimitMetrics) -> f64 {
        if metrics.total_requests == 0 {
            return 1.0; // 沒有請求時認為是健康的
        }

        // 速率限制觸發率越低越好
        let rate_limit_ratio = metrics.rate_limit_hits as f64 / metrics.total_requests as f64;
        let rate_limit_score = 1.0 - (rate_limit_ratio * 2.0).min(1.0);

        // 平均等待時間越短越好
        let wait_time_score = if metrics.average_wait_time_ms < 100.0 {
            1.0
        } else if metrics.average_wait_time_ms < 1000.0 {
            1.0 - ((metrics.average_wait_time_ms - 100.0) / 900.0)
        } else {
            0.0
        };

        (rate_limit_score + wait_time_score) / 2.0
    }

    /// 計算 API 健康分數
    fn calculate_api_health_score(&self, metrics: &ApiMetrics) -> f64 {
        let total_calls = metrics.successful_calls + metrics.failed_calls;
        if total_calls == 0 {
            return 1.0;
        }

        // 成功率是主要指標
        let success_rate_score = metrics.success_rate / 100.0;

        // 響應時間分數
        let response_time_score = if metrics.average_response_time_ms < 500.0 {
            1.0
        } else if metrics.average_response_time_ms < 2000.0 {
            1.0 - ((metrics.average_response_time_ms - 500.0) / 1500.0)
        } else {
            0.0
        };

        (success_rate_score * 0.7 + response_time_score * 0.3)
    }

    /// 計算事件處理健康分數
    fn calculate_event_health_score(&self, metrics: &EventMetrics) -> f64 {
        if metrics.total_events == 0 {
            return 1.0;
        }

        // 處理成功率
        let processed_ratio = metrics.processed_events as f64 / metrics.total_events as f64;
        let processing_score = processed_ratio;

        // 處理時間分數
        let processing_time_score = if metrics.average_processing_time_ms < 100.0 {
            1.0
        } else if metrics.average_processing_time_ms < 500.0 {
            1.0 - ((metrics.average_processing_time_ms - 100.0) / 400.0)
        } else {
            0.0
        };

        (processing_score * 0.8 + processing_time_score * 0.2)
    }

    /// 重設所有指標
    pub async fn reset_metrics(&self) {
        let mut rate_limit_metrics = self.rate_limit_metrics.write().await;
        *rate_limit_metrics = RateLimitMetrics::default();

        let mut api_metrics = self.api_metrics.write().await;
        *api_metrics = ApiMetrics::default();

        let mut event_metrics = self.event_metrics.write().await;
        *event_metrics = EventMetrics::default();

        self.total_wait_time_ms.store(0, Ordering::Relaxed);
        self.total_response_time_ms.store(0, Ordering::Relaxed);
        self.total_processing_time_ms.store(0, Ordering::Relaxed);

        tracing::info!("所有監控指標已重設");
    }

    /// 導出指標為 JSON 字符串
    pub async fn export_metrics_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct MetricsExport {
            rate_limit: RateLimitMetrics,
            api: ApiMetrics,
            events: EventMetrics,
            health: SystemHealth,
            uptime_seconds: f64,
        }

        let export = MetricsExport {
            rate_limit: self.get_rate_limit_metrics().await,
            api: self.get_api_metrics().await,
            events: self.get_event_metrics().await,
            health: self.get_system_health().await,
            uptime_seconds: self.start_time.elapsed().as_secs_f64(),
        };

        Ok(serde_json::to_string_pretty(&export)?)
    }

    /// 獲取運行時間
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for DiscordMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        let monitor = DiscordMonitor::new();

        let rate_limit_metrics = monitor.get_rate_limit_metrics().await;
        assert_eq!(rate_limit_metrics.total_requests, 0);

        let api_metrics = monitor.get_api_metrics().await;
        assert_eq!(api_metrics.successful_calls, 0);

        let event_metrics = monitor.get_event_metrics().await;
        assert_eq!(event_metrics.total_events, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_metrics_recording() {
        let monitor = DiscordMonitor::new();

        monitor.record_rate_limit_event("test_route", 100, false).await;
        monitor.record_rate_limit_event("test_route", 200, true).await;

        let metrics = monitor.get_rate_limit_metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.rate_limit_hits, 2);
        assert_eq!(metrics.global_rate_limit_hits, 1);
        assert_eq!(metrics.max_wait_time_ms, 200);
    }

    #[tokio::test]
    async fn test_api_metrics_recording() {
        let monitor = DiscordMonitor::new();

        monitor.record_api_call("test_endpoint", 150, true).await;
        monitor.record_api_call("test_endpoint", 200, false).await;
        monitor.record_api_timeout("test_endpoint").await;

        let metrics = monitor.get_api_metrics().await;
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.timeout_calls, 1);
        assert_eq!(metrics.success_rate, 50.0);
    }

    #[tokio::test]
    async fn test_event_metrics_recording() {
        let monitor = DiscordMonitor::new();

        monitor.record_event_processing("member_join", 50, "success").await;
        monitor.record_event_processing("member_join", 30, "duplicate").await;
        monitor.record_event_processing("member_join", 100, "failed").await;

        let metrics = monitor.get_event_metrics().await;
        assert_eq!(metrics.total_events, 3);
        assert_eq!(metrics.processed_events, 1);
        assert_eq!(metrics.duplicate_events, 1);
        assert_eq!(metrics.failed_events, 1);
    }

    #[tokio::test]
    async fn test_system_health_calculation() {
        let monitor = DiscordMonitor::new();

        // 記錄一些健康數據
        monitor.record_rate_limit_event("test", 50, false).await;
        monitor.record_api_call("test", 100, true).await;
        monitor.record_event_processing("test", 75, "success").await;

        let health = monitor.get_system_health().await;

        // 應該是健康的，因為所有指標都很好
        assert_eq!(health.overall_health, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_metrics_reset() {
        let monitor = DiscordMonitor::new();

        // 記錄一些數據
        monitor.record_rate_limit_event("test", 100, false).await;
        monitor.record_api_call("test", 150, true).await;
        monitor.record_event_processing("test", 50, "success").await;

        // 重設
        monitor.reset_metrics().await;

        // 驗證重設
        let rate_limit_metrics = monitor.get_rate_limit_metrics().await;
        assert_eq!(rate_limit_metrics.total_requests, 0);

        let api_metrics = monitor.get_api_metrics().await;
        assert_eq!(api_metrics.successful_calls, 0);

        let event_metrics = monitor.get_event_metrics().await;
        assert_eq!(event_metrics.total_events, 0);
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let monitor = DiscordMonitor::new();

        // 記錄一些數據
        monitor.record_rate_limit_event("test", 100, false).await;
        monitor.record_api_call("test", 150, true).await;
        monitor.record_event_processing("test", 50, "success").await;

        // 導出 JSON
        let json = monitor.export_metrics_json().await.unwrap();

        // 驗證 JSON 包含預期字段
        assert!(json.contains("rate_limit"));
        assert!(json.contains("api"));
        assert!(json.contains("events"));
        assert!(json.contains("health"));
        assert!(json.contains("uptime_seconds"));

        // 解析 JSON 驗證結構
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("rate_limit").is_some());
        assert!(value.get("api").is_some());
        assert!(value.get("events").is_some());
        assert!(value.get("health").is_some());
        assert!(value.get("uptime_seconds").is_some());
    }

    #[test]
    fn test_health_status_from_score() {
        assert_eq!(HealthStatus::from_score(0.95), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from_score(0.8), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from_score(0.75), HealthStatus::Warning);
        assert_eq!(HealthStatus::from_score(0.5), HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::from_score(0.1), HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::from_score(0.0), HealthStatus::Unknown);
    }

    #[test]
    fn test_uptime() {
        let monitor = DiscordMonitor::new();
        let uptime = monitor.get_uptime();

        // 應該有一個小的運行時間
        assert!(uptime > Duration::from_secs(0));
        assert!(uptime < Duration::from_secs(1));
    }
}