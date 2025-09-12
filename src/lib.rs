pub mod core;
pub mod config;
pub mod bot_manager;
pub mod protection;
pub mod audit;
pub mod commands;

// Re-export main types at the crate level
pub use core::{
    BotManager, BotId, BotInstance, BotState, 
    BotManagerError, ProcessSupervisor, ServiceRegistry,
    ProtectionLevel
};

// Re-export bot_manager types
pub use bot_manager::{
    lifecycle::LifecycleManager,
    health::{HealthMonitor, HealthStatus, BotStatus},
    restart_policy::{AutoRestartSupervisor, RestartPolicy, RestartReporter, RestartEvent},
};

// Re-export config types
pub use config::{BotConfig, ConfigService, ConfigError, ValidationEngine, LlmConfig};
