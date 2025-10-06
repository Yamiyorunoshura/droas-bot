use crate::discord_gateway::{DiscordGateway, ConnectionStatus};
use crate::health::HealthChecker;
use crate::metrics::MetricsCollector;
use crate::error::Result;
use std::time::Instant;
use warp::Filter;

/// 監控服務健康狀態
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtendedHealthStatus {
    pub discord_connected: bool,
    pub database_connected: bool,
    pub cache_connected: bool,
    #[serde(with = "serde_millis_as_secs")]
    pub last_check: Instant,
    pub uptime: std::time::Duration,
}

// 自定義序列化模塊用於 Instant
mod serde_millis_as_secs {
    use serde::Serializer;
    use std::time::Instant;

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = instant.elapsed().as_millis() as u64;
        serializer.serialize_u64(millis)
    }
}

/// 監控服務
///
/// 負責收集系統指標、監控各種服務健康狀態，並提供 HTTP 端點用於外部監控
pub struct MonitoringService {
    health_checker: HealthChecker,
    metrics_collector: MetricsCollector,
    database_pool: sqlx::PgPool,
    discord_gateway: Option<DiscordGateway>,
    cache_client: Option<redis::Client>,
}

impl MonitoringService {
    /// 創建新的監控服務實例
    pub fn new(
        health_checker: HealthChecker,
        metrics_collector: MetricsCollector,
        database_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            health_checker,
            metrics_collector,
            database_pool,
            discord_gateway: None,
            cache_client: None,
        }
    }

    /// 創建包含快取服務的監控服務
    pub fn with_cache(
        health_checker: HealthChecker,
        metrics_collector: MetricsCollector,
        database_pool: sqlx::PgPool,
        cache_client: redis::Client,
    ) -> Self {
        Self {
            health_checker,
            metrics_collector,
            database_pool,
            discord_gateway: None,
            cache_client: Some(cache_client),
        }
    }

    /// 創建包含 Discord Gateway 的監控服務
    pub fn with_gateway(
        database_pool: sqlx::PgPool,
        discord_gateway: DiscordGateway,
    ) -> Self {
        Self {
            health_checker: HealthChecker::new(),
            metrics_collector: MetricsCollector::new(),
            database_pool,
            discord_gateway: Some(discord_gateway),
            cache_client: None,
        }
    }

    /// 檢查系統整體健康狀態
    pub async fn check_system_health(&self) -> ExtendedHealthStatus {
        let discord_connected = if let Some(gateway) = &self.discord_gateway {
            gateway.get_status().await == ConnectionStatus::Connected
        } else {
            false
        };

        let database_connected = self.check_database_connection().await
            .unwrap_or(false);

        let cache_connected = if let Some(cache_client) = &self.cache_client {
            self.check_cache_connection(cache_client).await
                .unwrap_or(false)
        } else {
            false
        };

        ExtendedHealthStatus {
            discord_connected,
            database_connected,
            cache_connected,
            last_check: Instant::now(),
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                - std::time::Duration::from_secs(1640995200), // 2022-01-01 預設啟動時間
        }
    }

    /// 檢查資料庫連接狀態
    async fn check_database_connection(&self) -> Result<bool> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.database_pool)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 檢查快取連接狀態
    async fn check_cache_connection(&self, cache_client: &redis::Client) -> Result<bool> {
        let mut conn = cache_client.get_async_connection().await
            .map_err(|e| crate::error::DiscordError::DatabaseConnectionError(e.to_string()))?;

        let result: redis::RedisResult<String> = redis::cmd("PING")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 獲取 Prometheus 格式的指標
    pub async fn get_prometheus_metrics(&self) -> String {
        self.metrics_collector.generate_prometheus_metrics().await
    }

    /// 記錄命令指標
    pub async fn record_command(&self, command: &str, response_time_ms: u64, success: bool) {
        self.metrics_collector.record_command(command, response_time_ms, success).await;
    }

    /// 記錄資料庫查詢指標
    pub async fn record_database_query(&self, query_time_ms: u64, success: bool) {
        self.metrics_collector.record_database_query(query_time_ms, success).await;
    }

    /// 記錄轉帳指標
    pub async fn record_transfer(&self, amount: f64, transfer_time_ms: u64, success: bool, error_type: Option<&str>) {
        self.metrics_collector.record_transfer(amount, transfer_time_ms, success, error_type).await;
    }

    /// 記錄帳戶創建指標
    pub async fn record_account_creation(&self, creation_time_ms: u64, success: bool) {
        self.metrics_collector.record_account_creation(creation_time_ms, success).await;
    }

    /// 獲取健康檢查器的引用
    pub fn health_checker(&self) -> &HealthChecker {
        &self.health_checker
    }

    /// 獲取指標收集器的引用
    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// 獲取資料庫連接池的引用
    pub fn database_pool(&self) -> &sqlx::PgPool {
        &self.database_pool
    }
}

/// 創建健康檢查路由
pub fn create_health_routes(monitoring_service: std::sync::Arc<MonitoringService>) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let health_route = warp::path("health")
        .and(warp::get())
        .and(warp::any().map(move || monitoring_service.clone()))
        .and_then(|monitoring_service: std::sync::Arc<MonitoringService>| async move {
            let health_status = monitoring_service.check_system_health().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&health_status))
        });

    health_route
}

/// 創建指標路由
pub fn create_metrics_routes(monitoring_service: std::sync::Arc<MonitoringService>) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(warp::any().map(move || monitoring_service.clone()))
        .and_then(|monitoring_service: std::sync::Arc<MonitoringService>| async move {
            let metrics = monitoring_service.get_prometheus_metrics().await;
            Ok::<_, warp::Rejection>(warp::reply::with_header(metrics, "content-type", "text/plain"))
        });

    metrics_route
}

/// 啟動監控服務器
pub async fn start_monitoring_server(monitoring_service: std::sync::Arc<MonitoringService>, port: u16) -> Result<()> {
    let health_routes = create_health_routes(monitoring_service.clone());
    let metrics_routes = create_metrics_routes(monitoring_service);

    let routes = health_routes.or(metrics_routes);

    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();

    tokio::spawn(async move {
        warp::serve(routes).run(addr).await;
    });

    println!("監控服務器已啟動，端口: {}", port);
    Ok(())
}