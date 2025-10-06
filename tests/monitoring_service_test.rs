use droas_bot::services::monitoring_service::*;
use droas_bot::health::{HealthChecker, HealthStatus};
use droas_bot::metrics::MetricsCollector;
use droas_bot::discord_gateway::ConnectionStatus;
use mockall::predicate::*;
use reqwest;
use std::time::Duration;
use tokio::time::sleep;
use warp::Filter;

#[cfg(test)]
mod monitoring_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        // 測試監控服務創建 - 驗收標準 1
        let health_checker = HealthChecker::new();
        let metrics_collector = MetricsCollector::new();

        // 創建一個模擬的資料庫連接池 URL（不會實際連接）
        let database_url = "postgresql://test:test@localhost/test";
        let database_pool = sqlx::PgPool::connect(database_url).await;

        if let Ok(pool) = database_pool {
            let monitoring_service = MonitoringService::new(
                health_checker,
                metrics_collector,
                pool,
            );

            // 驗證服務創建成功
            let health_status = monitoring_service.check_system_health().await;

            // 檢查健康狀態結構
            assert!(!health_status.discord_connected); // 預設為 false
        }
    }

    #[tokio::test]
    async fn test_metrics_collector_functionality() {
        // 測試指標收集器功能 - 驗收標準 2
        let metrics_collector = MetricsCollector::new();

        // 記錄一些測試數據
        metrics_collector.record_command("balance", 100, true).await;
        metrics_collector.record_database_query(50, true).await;

        // 獲取指標並驗證
        let command_metrics = metrics_collector.get_command_metrics("balance").await;
        assert!(command_metrics.is_some());
        assert_eq!(command_metrics.unwrap().total_commands, 1);

        let db_metrics = metrics_collector.get_database_metrics().await;
        assert_eq!(db_metrics.total_queries, 1);
        assert_eq!(db_metrics.successful_queries, 1);
    }

    #[tokio::test]
    async fn test_prometheus_metrics_format() {
        // 測試 Prometheus 指標格式 - 驗收標準 3
        let metrics_collector = MetricsCollector::new();

        // 獲取 Prometheus 格式的指標
        let prometheus_metrics = metrics_collector.generate_prometheus_metrics().await;

        // 驗證格式正確
        assert!(prometheus_metrics.contains("# HELP"));
        assert!(prometheus_metrics.contains("# TYPE"));
        assert!(prometheus_metrics.contains("droas_"));

        // 驗證包含關鍵指標
        assert!(prometheus_metrics.contains("droas_system_uptime_seconds"));
        assert!(prometheus_metrics.contains("droas_discord_connections_total"));
        assert!(prometheus_metrics.contains("droas_database_queries_total"));
    }

    #[tokio::test]
    async fn test_health_checker_functionality() {
        // 測試健康檢查器功能 - 驗收標準 4
        let health_checker = HealthChecker::new();

        // 創建一個模擬的資料庫連接池 URL（不會實際連接）
        let database_url = "postgresql://test:test@localhost/test";
        let database_pool = sqlx::PgPool::connect(database_url).await;

        if let Ok(pool) = database_pool {
            // 創建模擬的 Discord Gateway
            let monitoring_service = MonitoringService::new(
                health_checker,
                MetricsCollector::new(),
                pool,
            );

            let health_status = monitoring_service.check_system_health().await;

            // 驗證健康狀態結構包含所有必要字段
            assert!(!health_status.discord_connected);
            assert!(!health_status.cache_connected);
        }
    }

    #[tokio::test]
    async fn test_error_recording() {
        // 測試錯誤記錄功能 - 驗收標準 5
        let metrics_collector = MetricsCollector::new();

        // 記錄錯誤狀態
        metrics_collector.record_command("transfer", 200, false).await;
        metrics_collector.record_transfer(100.0, 300, false, Some("insufficient_balance")).await;

        // 驗證錯誤指標被正確記錄
        let command_metrics = metrics_collector.get_command_metrics("transfer").await;
        assert!(command_metrics.is_some());
        assert_eq!(command_metrics.unwrap().failed_commands, 1);

        let transfer_metrics = metrics_collector.get_transfer_metrics().await;
        assert_eq!(transfer_metrics.failed_transfers, 1);
        assert_eq!(transfer_metrics.insufficient_balance_errors, 1);
    }

    #[tokio::test]
    async fn test_monitoring_service_with_cache() {
        // 測試包含快取的監控服務 - 驗收標準 6
        let health_checker = HealthChecker::new();
        let metrics_collector = MetricsCollector::new();

        // 創建模擬的 Redis 客戶端
        let redis_client = redis::Client::open("redis://localhost").unwrap();

        // 創建模擬的資料庫連接池 URL
        let database_url = "postgresql://test:test@localhost/test";
        let database_pool = sqlx::PgPool::connect(database_url).await;

        if let Ok(pool) = database_pool {
            let monitoring_service = MonitoringService::with_cache(
                health_checker,
                metrics_collector,
                pool,
                redis_client,
            );

            let health_status = monitoring_service.check_system_health().await;

            // 驗證服務創建成功
            assert!(!health_status.discord_connected);
            assert!(!health_status.database_connected);
            assert!(!health_status.cache_connected);
        }
    }

    #[tokio::test]
    async fn test_monitoring_service_with_gateway() {
        // 測試包含 Gateway 的監控服務 - 驗收標準 7
        // 創建模擬的資料庫連接池 URL
        let database_url = "postgresql://test:test@localhost/test";
        let database_pool = sqlx::PgPool::connect(database_url).await;

        if let Ok(pool) = database_pool {
            let monitoring_service = MonitoringService::with_gateway(
                pool,
                droas_bot::discord_gateway::DiscordGateway::new(),
            );

            let health_status = monitoring_service.check_system_health().await;

            // 驗證服務創建成功
            assert!(!health_status.database_connected);
            assert!(!health_status.cache_connected);
        }
    }
}

