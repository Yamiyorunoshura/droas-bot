use tracing::{info, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "droas_bot=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn log_connection_success() {
    info!("Discord API connection established successfully");
}

pub fn log_connection_error(error: &str) {
    error!("Discord API connection failed: {}", error);
}

pub fn log_command_received(command: &str) {
    debug!("Received command: {}", command);
}

pub fn log_command_processed(command: &str, response_time_ms: u64) {
    info!("Command '{}' processed in {}ms", command, response_time_ms);
}

pub fn log_event_received(event_type: &str) {
    debug!("Discord event received: {}", event_type);
}