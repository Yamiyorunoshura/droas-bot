use crate::metrics::MetricsCollector;
use crate::services::monitoring_config::MonitoringConfig;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};

/// 異步指標收集器
///
/// 在背景中定期收集和處理指標，避免阻斷主要業務邏輯
pub struct AsyncMetricsCollector {
    /// 基礎指標收集器
    metrics_collector: Arc<MetricsCollector>,

    /// 監控配置
    config: Arc<MonitoringConfig>,

    /// 收集器狀態
    state: Arc<RwLock<CollectorState>>,

    /// 運行標誌
    running: Arc<RwLock<bool>>,
}

/// 收集器狀態
#[derive(Debug)]
pub struct CollectorState {
    /// 最後一次收集時間
    last_collection_time: Option<std::time::Instant>,

    /// 總收集次數
    total_collections: u64,

    /// 成功收集次數
    successful_collections: u64,

    /// 失敗收集次數
    failed_collections: u64,

    /// 平均收集時間
    average_collection_time_ms: f64,
}

impl AsyncMetricsCollector {
    /// 創建新的異步指標收集器
    pub fn new(metrics_collector: Arc<MetricsCollector>, config: MonitoringConfig) -> Self {
        Self {
            metrics_collector,
            config: Arc::new(config),
            state: Arc::new(RwLock::new(CollectorState {
                last_collection_time: None,
                total_collections: 0,
                successful_collections: 0,
                failed_collections: 0,
                average_collection_time_ms: 0.0,
            })),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 啟動異步指標收集
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut running = self.running.write().await;
        if *running {
            warn!("異步指標收集器已經在運行中");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("啟動異步指標收集器，間隔: {:?}", self.config.metrics_collection_interval);

        let metrics_collector = Arc::clone(&self.metrics_collector);
        let config = Arc::clone(&self.config);
        let state = Arc::clone(&self.state);
        let running_flag = Arc::clone(&self.running);

        let mut interval = interval(self.config.metrics_collection_interval);
        interval.tick().await; // 跳過第一次立即觸發

        tokio::spawn(async move {
            while *running_flag.read().await {
                interval.tick().await;

                let start_time = std::time::Instant::now();

                match Self::collect_metrics(&metrics_collector, &config).await {
                    Ok(_) => {
                        let elapsed = start_time.elapsed().as_millis() as f64;

                        let mut state_guard = state.write().await;
                        state_guard.last_collection_time = Some(std::time::Instant::now());
                        state_guard.total_collections += 1;
                        state_guard.successful_collections += 1;

                        // 更新平均收集時間
                        let total = state_guard.total_collections as f64;
                        let current_avg = state_guard.average_collection_time_ms;
                        state_guard.average_collection_time_ms =
                            (current_avg * (total - 1.0) + elapsed) / total;

                        info!("指標收集完成，耗時: {}ms", elapsed);
                    }
                    Err(e) => {
                        error!("指標收集失敗: {:?}", e);

                        let mut state_guard = state.write().await;
                        state_guard.total_collections += 1;
                        state_guard.failed_collections += 1;
                    }
                }
            }

            info!("異步指標收集器已停止");
        });

        Ok(())
    }

    /// 停止異步指標收集
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("異步指標收集器停止信號已發送");
    }

    /// 獲取收集器狀態
    pub async fn get_state(&self) -> CollectorState {
        self.state.read().await.clone()
    }

    /// 檢查收集器是否正在運行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 執行一次指標收集
    async fn collect_metrics(
        metrics_collector: &MetricsCollector,
        config: &MonitoringConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 更新系統指標
        metrics_collector.update_system_metrics().await;

        // 如果啟用了詳細指標，收集更多數據
        if config.enable_detailed_metrics {
            // 這裡可以添加更多詳細指標的收集邏輯
            // 例如：系統資源使用情況、記憶體使用量等
        }

        // 如果啟用了性能監控，收集性能相關指標
        if config.enable_performance_monitoring {
            // 這裡可以添加性能監控相關的指標收集
            // 例如：CPU 使用率、記憶體使用率、響應時間統計等
        }

        Ok(())
    }

    /// 批量記錄指標
    ///
    /// 用於一次性記錄多個指標，提高性能
    pub async fn record_batch_metrics(&self, metrics: Vec<BatchMetric>) {
        let start_time = std::time::Instant::now();
        let metrics_count = metrics.len();

        for metric in metrics {
            match metric {
                BatchMetric::Command { command, response_time_ms, success } => {
                    self.metrics_collector.record_command(&command, response_time_ms, success).await;
                }
                BatchMetric::DatabaseQuery { query_time_ms, success } => {
                    self.metrics_collector.record_database_query(query_time_ms, success).await;
                }
                BatchMetric::Transfer { amount, transfer_time_ms, success, error_type } => {
                    self.metrics_collector.record_transfer(amount, transfer_time_ms, success, error_type.as_deref()).await;
                }
                BatchMetric::AccountCreation { creation_time_ms, success } => {
                    self.metrics_collector.record_account_creation(creation_time_ms, success).await;
                }
            }
        }

        let elapsed = start_time.elapsed();
        if elapsed > Duration::from_millis(100) {
            warn!("批量指標記錄耗時較長: {}ms, 指標數量: {}", elapsed.as_millis(), metrics_count);
        }
    }

    /// 獲取指標收集器的引用
    pub fn metrics_collector(&self) -> &Arc<MetricsCollector> {
        &self.metrics_collector
    }

    /// 獲取監控配置的引用
    pub fn config(&self) -> &Arc<MonitoringConfig> {
        &self.config
    }
}

/// 批量指標枚舉
#[derive(Debug, Clone)]
pub enum BatchMetric {
    /// 命令指標
    Command {
        command: String,
        response_time_ms: u64,
        success: bool,
    },

    /// 資料庫查詢指標
    DatabaseQuery {
        query_time_ms: u64,
        success: bool,
    },

    /// 轉帳指標
    Transfer {
        amount: f64,
        transfer_time_ms: u64,
        success: bool,
        error_type: Option<String>,
    },

    /// 帳戶創建指標
    AccountCreation {
        creation_time_ms: u64,
        success: bool,
    },
}

impl Clone for CollectorState {
    fn clone(&self) -> Self {
        Self {
            last_collection_time: self.last_collection_time,
            total_collections: self.total_collections,
            successful_collections: self.successful_collections,
            failed_collections: self.failed_collections,
            average_collection_time_ms: self.average_collection_time_ms,
        }
    }
}

/// 指標收集器統計信息
#[derive(Debug, Clone)]
pub struct CollectorStats {
    pub is_running: bool,
    pub total_collections: u64,
    pub successful_collections: u64,
    pub failed_collections: u64,
    pub success_rate: f64,
    pub average_collection_time_ms: f64,
    pub last_collection_time: Option<std::time::Instant>,
}

impl AsyncMetricsCollector {
    /// 獲取收集器統計信息
    pub async fn get_stats(&self) -> CollectorStats {
        let state = self.get_state().await;
        let is_running = self.is_running().await;

        let success_rate = if state.total_collections > 0 {
            (state.successful_collections as f64 / state.total_collections as f64) * 100.0
        } else {
            0.0
        };

        CollectorStats {
            is_running,
            total_collections: state.total_collections,
            successful_collections: state.successful_collections,
            failed_collections: state.failed_collections,
            success_rate,
            average_collection_time_ms: state.average_collection_time_ms,
            last_collection_time: state.last_collection_time,
        }
    }
}