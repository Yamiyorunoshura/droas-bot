pub mod schema;
pub mod service;
pub mod events;
pub mod watcher;
pub mod hot_reload;

// Re-export schema types
pub use schema::{BotConfig, BotConfigInner, LlmConfig, ConfigError, ConfigVersion};

// Re-export service types
pub use service::{ConfigService, ValidationEngine};

// Re-export event types
pub use events::{ConfigEvent, EventBus, EventBusError};

// Re-export watcher types
pub use watcher::{FileWatcher, WatchHandle, WatcherError};

// Re-export hot reload types
pub use hot_reload::{HotReloadService, HotReloadError};
