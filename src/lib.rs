pub mod core;

// Re-export main types at the crate level
pub use core::{
    BotManager, BotId, BotConfig, BotInstance, BotState, 
    HealthStatus, BotManagerError, ProcessSupervisor, ServiceRegistry,
    LlmConfig, ProtectionLevel, RestartPolicy
};
