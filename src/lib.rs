pub mod core;
pub mod config;

// Re-export main types at the crate level
pub use core::{
    BotManager, BotId, BotInstance, BotState, 
    HealthStatus, BotManagerError, ProcessSupervisor, ServiceRegistry,
    ProtectionLevel, RestartPolicy
};

// Re-export config types
pub use config::{BotConfig, ConfigService, ConfigError, ValidationEngine, LlmConfig};
