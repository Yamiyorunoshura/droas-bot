use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 監控配置結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 監控服務器端口
    pub server_port: u16,

    /// 健康檢查間隔
    pub health_check_interval: Duration,

    /// 指標收集間隔
    pub metrics_collection_interval: Duration,

    /// 是否啟用詳細指標
    pub enable_detailed_metrics: bool,

    /// 是否啟用性能監控
    pub enable_performance_monitoring: bool,

    /// 指標保留時間
    pub metrics_retention_period: Duration,

    /// 異常警報閾值
    pub alert_thresholds: AlertThresholds,

    /// 組件監控配置
    pub component_monitoring: ComponentMonitoringConfig,
}

/// 異常警報閾值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// 資料庫查詢響應時間閾值（毫秒）
    pub database_query_time_threshold_ms: u64,

    /// 命令響應時間閾值（毫秒）
    pub command_response_time_threshold_ms: u64,

    /// 轉帳響應時間閾值（毫秒）
    pub transfer_response_time_threshold_ms: u64,

    /// 錯誤率閾值（百分比）
    pub error_rate_threshold_percent: f64,

    /// 資料庫連接失敗閾值（連續失敗次數）
    pub database_connection_failure_threshold: u32,

    /// Discord API 連接失敗閾值（連續失敗次數）
    pub discord_api_failure_threshold: u32,

    /// Redis 連接失敗閾值（連續失敗次數）
    pub redis_connection_failure_threshold: u32,
}

/// 組件監控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMonitoringConfig {
    /// 是否監控資料庫
    pub monitor_database: bool,

    /// 是否監控快取
    pub monitor_cache: bool,

    /// 是否監控 Discord API
    pub monitor_discord_api: bool,

    /// 是否監控系統資源
    pub monitor_system_resources: bool,

    /// 是否監控業務指標
    pub monitor_business_metrics: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            health_check_interval: Duration::from_secs(30),
            metrics_collection_interval: Duration::from_secs(60),
            enable_detailed_metrics: true,
            enable_performance_monitoring: true,
            metrics_retention_period: Duration::from_secs(86400), // 24 小時
            alert_thresholds: AlertThresholds::default(),
            component_monitoring: ComponentMonitoringConfig::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            database_query_time_threshold_ms: 1000, // 1 秒
            command_response_time_threshold_ms: 2000, // 2 秒
            transfer_response_time_threshold_ms: 3000, // 3 秒
            error_rate_threshold_percent: 5.0, // 5%
            database_connection_failure_threshold: 3,
            discord_api_failure_threshold: 3,
            redis_connection_failure_threshold: 3,
        }
    }
}

impl Default for ComponentMonitoringConfig {
    fn default() -> Self {
        Self {
            monitor_database: true,
            monitor_cache: true,
            monitor_discord_api: true,
            monitor_system_resources: false, // 預設關閉以減少開銷
            monitor_business_metrics: true,
        }
    }
}

impl MonitoringConfig {
    /// 從環境變量創建配置
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // 從環境變量讀取配置
        if let Ok(port) = std::env::var("DROAS_MONITORING_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.server_port = port_num;
            }
        }

        if let Ok(interval) = std::env::var("DROAS_HEALTH_CHECK_INTERVAL") {
            if let Ok(seconds) = interval.parse::<u64>() {
                config.health_check_interval = Duration::from_secs(seconds);
            }
        }

        if let Ok(enabled) = std::env::var("DROAS_DETAILED_METRICS") {
            config.enable_detailed_metrics = enabled.to_lowercase() == "true";
        }

        if let Ok(enabled) = std::env::var("DROAS_PERFORMANCE_MONITORING") {
            config.enable_performance_monitoring = enabled.to_lowercase() == "true";
        }

        config
    }

    /// 從配置文件創建配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: MonitoringConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 驗證配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("監控服務器端口不能為 0".to_string());
        }

        if self.health_check_interval.is_zero() {
            return Err("健康檢查間隔不能為零".to_string());
        }

        if self.metrics_collection_interval.is_zero() {
            return Err("指標收集間隔不能為零".to_string());
        }

        if self.metrics_retention_period.is_zero() {
            return Err("指標保留時間不能為零".to_string());
        }

        if self.alert_thresholds.error_rate_threshold_percent < 0.0
            || self.alert_thresholds.error_rate_threshold_percent > 100.0 {
            return Err("錯誤率閾值必須在 0-100 之間".to_string());
        }

        Ok(())
    }

    /// 獲取監控服務器地址
    pub fn server_address(&self) -> String {
        format!("0.0.0.0:{}", self.server_port)
    }

    /// 獲取健康檢查端點 URL
    pub fn health_check_endpoint(&self) -> String {
        format!("http://localhost:{}/health", self.server_port)
    }

    /// 獲取指標端點 URL
    pub fn metrics_endpoint(&self) -> String {
        format!("http://localhost:{}/metrics", self.server_port)
    }
}