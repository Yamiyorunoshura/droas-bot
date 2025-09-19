//! Prometheus 指標收集器模組
//!
//! 此模組提供 Prometheus 格式的指標收集和導出功能，包括：
//! - HTTP 請求指標
//! - Gateway 連接指標
//! - 事件處理指標
//! - 系統指標
//! - 自定義業務指標

use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder,
};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

/// Discord 機器人的 Prometheus 指標收集器
pub struct PrometheusMetrics {
    /// Registry for all metrics
    registry: Registry,

    /// HTTP 請求相關指標
    pub http_requests_total: Counter,
    pub http_request_duration_seconds: Histogram,
    pub http_active_requests: Gauge,

    /// Gateway 連接相關指標
    pub gateway_connections_total: Counter,
    pub gateway_connection_duration_seconds: Histogram,
    pub gateway_reconnects_total: Counter,
    pub gateway_heartbeat_total: Counter,
    pub gateway_connection_status: Gauge,

    /// 事件處理相關指標
    pub events_processed_total: Counter,
    pub event_processing_duration_seconds: Histogram,
    pub events_failed_total: Counter,

    /// 速率限制相關指標
    pub rate_limit_hits_total: Counter,
    pub rate_limit_wait_duration_seconds: Histogram,

    /// 系統指標
    pub system_memory_usage_bytes: Gauge,
    pub system_cpu_usage_percent: Gauge,
    pub bot_uptime_seconds: Gauge,
}

impl PrometheusMetrics {
    /// 創建新的 Prometheus 指標收集器
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // HTTP 請求指標
        let http_requests_total = Counter::with_opts(Opts::new(
            "droas_http_requests_total",
            "Total number of HTTP requests made to Discord API"
        ))?;
        let http_request_duration_seconds = Histogram::with_opts(HistogramOpts::new(
            "droas_http_request_duration_seconds",
            "Duration of HTTP requests to Discord API in seconds"
        ))?;
        let http_active_requests = Gauge::with_opts(Opts::new(
            "droas_http_active_requests",
            "Number of active HTTP requests"
        ))?;

        // Gateway 連接指標
        let gateway_connections_total = Counter::with_opts(Opts::new(
            "droas_gateway_connections_total",
            "Total number of Gateway connections established"
        ))?;
        let gateway_connection_duration_seconds = Histogram::with_opts(HistogramOpts::new(
            "droas_gateway_connection_duration_seconds",
            "Duration of Gateway connections in seconds"
        ))?;
        let gateway_reconnects_total = Counter::with_opts(Opts::new(
            "droas_gateway_reconnects_total",
            "Total number of Gateway reconnections"
        ))?;
        let gateway_heartbeat_total = Counter::with_opts(Opts::new(
            "droas_gateway_heartbeat_total",
            "Total number of Gateway heartbeats sent"
        ))?;
        let gateway_connection_status = Gauge::with_opts(Opts::new(
            "droas_gateway_connection_status",
            "Current Gateway connection status (1=connected, 0=disconnected, 2=error)"
        ))?;

        // 事件處理指標
        let events_processed_total = Counter::with_opts(Opts::new(
            "droas_events_processed_total",
            "Total number of Discord events processed"
        ))?;
        let event_processing_duration_seconds = Histogram::with_opts(HistogramOpts::new(
            "droas_event_processing_duration_seconds",
            "Duration of event processing in seconds"
        ))?;
        let events_failed_total = Counter::with_opts(Opts::new(
            "droas_events_failed_total",
            "Total number of failed event processing"
        ))?;

        // 速率限制指標
        let rate_limit_hits_total = Counter::with_opts(Opts::new(
            "droas_rate_limit_hits_total",
            "Total number of rate limit hits"
        ))?;
        let rate_limit_wait_duration_seconds = Histogram::with_opts(HistogramOpts::new(
            "droas_rate_limit_wait_duration_seconds",
            "Duration spent waiting for rate limits in seconds"
        ))?;

        // 系統指標
        let system_memory_usage_bytes = Gauge::with_opts(Opts::new(
            "droas_system_memory_usage_bytes",
            "Current memory usage in bytes"
        ))?;
        let system_cpu_usage_percent = Gauge::with_opts(Opts::new(
            "droas_system_cpu_usage_percent",
            "Current CPU usage percentage"
        ))?;
        let bot_uptime_seconds = Gauge::with_opts(Opts::new(
            "droas_bot_uptime_seconds",
            "Bot uptime in seconds"
        ))?;

        // 註冊所有指標
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_active_requests.clone()))?;
        registry.register(Box::new(gateway_connections_total.clone()))?;
        registry.register(Box::new(gateway_connection_duration_seconds.clone()))?;
        registry.register(Box::new(gateway_reconnects_total.clone()))?;
        registry.register(Box::new(gateway_heartbeat_total.clone()))?;
        registry.register(Box::new(gateway_connection_status.clone()))?;
        registry.register(Box::new(events_processed_total.clone()))?;
        registry.register(Box::new(event_processing_duration_seconds.clone()))?;
        registry.register(Box::new(events_failed_total.clone()))?;
        registry.register(Box::new(rate_limit_hits_total.clone()))?;
        registry.register(Box::new(rate_limit_wait_duration_seconds.clone()))?;
        registry.register(Box::new(system_memory_usage_bytes.clone()))?;
        registry.register(Box::new(system_cpu_usage_percent.clone()))?;
        registry.register(Box::new(bot_uptime_seconds.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            http_active_requests,
            gateway_connections_total,
            gateway_connection_duration_seconds,
            gateway_reconnects_total,
            gateway_heartbeat_total,
            gateway_connection_status,
            events_processed_total,
            event_processing_duration_seconds,
            events_failed_total,
            rate_limit_hits_total,
            rate_limit_wait_duration_seconds,
            system_memory_usage_bytes,
            system_cpu_usage_percent,
            bot_uptime_seconds,
        })
    }

    /// 獲取指標的 Prometheus 格式輸出
    pub fn gather_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let buffer = encoder.encode_to_string(&metric_families)?;
        Ok(buffer)
    }

    /// 更新系統指標
    pub fn update_system_metrics(&self) {
        // 更新運行時間
        if let Ok(uptime) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
        {
            self.bot_uptime_seconds.set(uptime);
        }

        // 更新記憶體使用量
        if let Ok(memory_usage) = self.get_memory_usage() {
            self.system_memory_usage_bytes.set(memory_usage as f64);
        }

        // 更新 CPU 使用率
        if let Ok(cpu_usage) = self.get_cpu_usage() {
            self.system_cpu_usage_percent.set(cpu_usage);
        }
    }

    /// 獲取記憶體使用量
    fn get_memory_usage(&self) -> Result<u64> {
        // 簡化的記憶體使用量獲取
        #[cfg(target_os = "linux")]
        {
            if let Ok(stats) = std::fs::read_to_string("/proc/self/statm") {
                let parts: Vec<&str> = stats.split_whitespace().collect();
                if let Some(pages) = parts.get(1) {
                    if let Ok(pages) = pages.parse::<u64>() {
                        // 假設頁面大小為 4KB
                        return Ok(pages * 4096);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 使用 sysctl 獲取記憶體使用量
            use std::process::Command;
            if let Ok(output) = Command::new("ps").args(&["-o", "rss=", "-p", &std::process::id().to_string()]).output() {
                if let Ok(rss_kb) = String::from_utf8_lossy(&output.stdout).trim().parse::<u64>() {
                    return Ok(rss_kb * 1024); // KB 轉換為 bytes
                }
            }
        }

        // 默認返回 0
        Ok(0)
    }

    /// 獲取 CPU 使用率
    fn get_cpu_usage(&self) -> Result<f64> {
        // 簡化的 CPU 使用率獲取
        #[cfg(target_os = "linux")]
        {
            if let Ok(stats) = std::fs::read_to_string("/proc/self/stat") {
                let parts: Vec<&str> = stats.split_whitespace().collect();
                if let (Some(utime), Some(stime)) = (parts.get(13), parts.get(14)) {
                    if let (Ok(utime), Ok(stime)) = (utime.parse::<u64>(), stime.parse::<u64>()) {
                        let total_time = utime + stime;
                        let uptime = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if uptime > 0 {
                            return Ok((total_time as f64) / (uptime as f64 * 100.0));
                        }
                    }
                }
            }
        }

        // 默認返回 0.0
        Ok(0.0)
    }

    /// 記錄 HTTP 請求
    pub fn record_http_request(&self, duration_seconds: f64, _status_code: u16) {
        self.http_requests_total.inc();
        self.http_request_duration_seconds.observe(duration_seconds);
    }

    /// 設置 Gateway 連接狀態
    pub fn set_gateway_connection_status(&self, status: u8) {
        self.gateway_connection_status.set(status as f64);
    }

    /// 記錄 Gateway 連接
    pub fn record_gateway_connection(&self) {
        self.gateway_connections_total.inc();
    }

    /// 記錄 Gateway 重連
    pub fn record_gateway_reconnect(&self) {
        self.gateway_reconnects_total.inc();
    }

    /// 記錄 Gateway 心跳
    pub fn record_gateway_heartbeat(&self) {
        self.gateway_heartbeat_total.inc();
    }

    /// 記錄事件處理
    pub fn record_event_processing(&self, duration_seconds: f64) {
        self.events_processed_total.inc();
        self.event_processing_duration_seconds.observe(duration_seconds);
    }

    /// 記錄事件處理失敗
    pub fn record_event_failure(&self) {
        self.events_failed_total.inc();
    }

    /// 記錄速率限制命中
    pub fn record_rate_limit_hit(&self, wait_duration_seconds: f64) {
        self.rate_limit_hits_total.inc();
        self.rate_limit_wait_duration_seconds.observe(wait_duration_seconds);
    }

    /// 增加活躍 HTTP 請求數
    pub fn inc_active_requests(&self) {
        self.http_active_requests.inc();
    }

    /// 減少活躍 HTTP 請求數
    pub fn dec_active_requests(&self) {
        self.http_active_requests.dec();
    }
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create Prometheus metrics")
    }
}

/// 全局 Prometheus 指標收集器
pub static GLOBAL_PROMETHEUS_METRICS: once_cell::sync::Lazy<Arc<RwLock<PrometheusMetrics>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(RwLock::new(PrometheusMetrics::new().expect("Failed to create global Prometheus metrics")))
    });

/// 初始化全局 Prometheus 指標
pub async fn init_global_metrics() -> Result<()> {
    let metrics = GLOBAL_PROMETHEUS_METRICS.write().await;
    // 初始化系統指標
    metrics.update_system_metrics();
    Ok(())
}

/// 獲取全局 Prometheus 指標
pub async fn get_global_metrics() -> String {
    let metrics = GLOBAL_PROMETHEUS_METRICS.read().await;
    metrics.gather_metrics().unwrap_or_else(|_| String::from("# Error gathering metrics"))
}

/// 更新全局系統指標
pub async fn update_global_system_metrics() {
    let metrics = GLOBAL_PROMETHEUS_METRICS.write().await;
    metrics.update_system_metrics();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_metrics_creation() {
        let metrics = PrometheusMetrics::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_metrics_gathering() {
        let metrics = PrometheusMetrics::new().unwrap();
        let output = metrics.gather_metrics();
        assert!(output.is_ok());
        let output_str = output.unwrap();
        assert!(output_str.contains("droas_"));
    }

    #[tokio::test]
    async fn test_global_metrics() {
        let result = init_global_metrics().await;
        assert!(result.is_ok());

        let metrics_output = get_global_metrics().await;
        assert!(!metrics_output.is_empty());
        assert!(metrics_output.contains("droas_"));
    }
}