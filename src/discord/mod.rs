//! Discord API 連接處理模組
//!
//! 此模組負責管理與 Discord API 的連接，包括：
//! - Gateway WebSocket 連接管理
//! - 速率限制處理和自動重試
//! - 連接監控和自動恢復
//! - API 請求的可靠處理
//! - REST API 客戶端和訊息發送
//! - 熔斷器保護機制
//! - 系統監控和指標收集

pub mod api_client;
pub mod circuit_breaker;
pub mod client;
pub mod commands;
pub mod event_handler;
pub mod gateway;
pub mod monitoring;
pub mod monitoring_server;
pub mod prometheus_metrics;
pub mod rate_limit;

pub use api_client::DiscordApiClient;
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, SharedCircuitBreaker,
};
pub use client::DiscordClient;
pub use event_handler::{EventHandler, EventResult, TestMemberJoinEvent};
pub use gateway::{GatewayManager, GatewayStatus};
pub use monitoring::{
    ApiMetrics, DiscordMonitor, EventMetrics, HealthStatus, RateLimitMetrics, SystemHealth,
};
pub use prometheus_metrics::{
    PrometheusMetrics, init_global_metrics, get_global_metrics, update_global_system_metrics,
};
pub use monitoring_server::{MonitoringServer, MonitoringServerConfig};
pub use rate_limit::{RateLimit, RateLimiter};
