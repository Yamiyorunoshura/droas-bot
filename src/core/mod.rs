pub mod types;
pub mod bot_manager;

// Re-export main types for convenience
pub use types::{
    BotId, BotConfig, BotInstance, BotState, HealthStatus, BotManagerError,
    LlmConfig, ProtectionLevel, RestartPolicy
};

pub use bot_manager::{
    BotManager, ProcessSupervisor, ServiceRegistry
};
