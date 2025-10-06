pub mod discord_gateway;
pub mod command_router;
pub mod config;
pub mod database;
pub mod error;
pub mod logging;
pub mod health;
pub mod metrics;
pub mod services;
pub mod cache;
pub mod styles;

// 重新導出監控服務相關類型
pub use services::monitoring_service::{
    MonitoringService, ExtendedHealthStatus,
    create_health_routes, create_metrics_routes, start_monitoring_server
};

/// 創建並配置監控服務
pub async fn create_monitoring_service(
    database_pool: sqlx::PgPool,
    _discord_gateway: Option<discord_gateway::DiscordGateway>,
    _redis_client: Option<redis::Client>,
) -> MonitoringService {
    let health_checker = health::HealthChecker::new();
    let metrics_collector = metrics::MetricsCollector::new();

    // 創建基礎監控服務
    let monitoring_service = MonitoringService::new(health_checker, metrics_collector, database_pool);

    // 注意：當前實現需要在創建後手動設置可選組件
    // 未來可以擴展 MonitoringService 來支持動態設置這些組件

    monitoring_service
}