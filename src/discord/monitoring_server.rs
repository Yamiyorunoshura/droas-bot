//! 監控服務器模組
//!
//! 此模組提供 HTTP 服務器來導出 Prometheus 指標和健康檢查端點

use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::discord::prometheus_metrics::get_global_metrics;

/// 監控服務器配置
#[derive(Debug, Clone)]
pub struct MonitoringServerConfig {
    /// 服務器監聽地址
    pub addr: SocketAddr,
    /// Prometheus 指標路徑
    pub metrics_path: String,
    /// 健康檢查路徑
    pub health_path: String,
}

impl Default for MonitoringServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:9090".parse().unwrap(),
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
        }
    }
}

/// 監控服務器
pub struct MonitoringServer {
    config: MonitoringServerConfig,
}

impl MonitoringServer {
    /// 創建新的監控服務器
    pub fn new(config: MonitoringServerConfig) -> Self {
        Self { config }
    }

    /// 使用默認配置創建監控服務器
    pub fn default() -> Self {
        Self::new(MonitoringServerConfig::default())
    }

    /// 啟動監控服務器
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.addr).await?;

        tracing::info!(
            "監控服務器已啟動，地址: {}",
            self.config.addr
        );

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    tracing::debug!("新連接來自: {}", addr);

                    let config = self.config.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(&mut stream, &config).await {
                            tracing::warn!("處理連接時發生錯誤: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("接受連接時發生錯誤: {}", e);
                }
            }
        }
    }

    /// 處理單個連接
    async fn handle_connection(
        stream: &mut tokio::net::TcpStream,
        config: &MonitoringServerConfig,
    ) -> Result<()> {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let request_line = request.lines().next().unwrap_or("");

        let (response, content_type) = if request_line.starts_with(&format!("GET {}", config.metrics_path)) {
            // Prometheus 指標端點
            let metrics = get_global_metrics().await;
            (metrics, "text/plain; version=0.0.4; charset=utf-8")
        } else if request_line.starts_with(&format!("GET {}", config.health_path)) {
            // 健康檢查端點
            let health_response = r#"{"status": "healthy", "timestamp": "2025-01-01T00:00:00Z"}"#;
            (health_response.to_string(), "application/json")
        } else {
            // 404 Not Found
            let response = "404 Not Found";
            (response.to_string(), "text/plain")
        };

        // 構建 HTTP 響應
        let http_response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type,
            response.len(),
            response
        );

        stream.write_all(http_response.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }

    /// 獲取服務器地址
    pub fn addr(&self) -> &SocketAddr {
        &self.config.addr
    }

    /// 獲取 Prometheus 指標 URL
    pub fn metrics_url(&self) -> String {
        format!("http://{}{}", self.config.addr, self.config.metrics_path)
    }

    /// 獲取健康檢查 URL
    pub fn health_url(&self) -> String {
        format!("http://{}{}", self.config.addr, self.config.health_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_server_config() {
        let config = MonitoringServerConfig::default();
        assert_eq!(config.addr.to_string(), "127.0.0.1:9090");
        assert_eq!(config.metrics_path, "/metrics");
        assert_eq!(config.health_path, "/health");
    }

    #[test]
    fn test_monitoring_server_creation() {
        let server = MonitoringServer::default();
        assert_eq!(server.addr().to_string(), "127.0.0.1:9090");
        assert_eq!(server.metrics_url(), "http://127.0.0.1:9090/metrics");
        assert_eq!(server.health_url(), "http://127.0.0.1:9090/health");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let config = MonitoringServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
        };

        let server = MonitoringServer::new(config);

        // 這裡我們只測試 URL 生成，不實際啟動服務器
        assert!(server.metrics_url().contains("/metrics"));
        assert!(server.health_url().contains("/health"));
    }
}