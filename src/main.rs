use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

mod assets;
mod config;
mod database;
mod discord;
mod error;
mod handlers;

use config::Config;
use discord::DiscordClient;
// Future use for error handling improvements
// use error::{GracefulError, DroasResult};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("droas_bot=info")
        .init();

    info!("Starting DROAS Bot...");

    // Load configuration with graceful error handling
    let config = match Config::load().await {
        Ok(config) => {
            info!("Configuration loaded successfully");
            Arc::new(config)
        }
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            return Ok(());
        }
    };

    // TODO: Initialize database connection

    // Initialize Discord client
    let mut discord_client = match DiscordClient::new(Arc::clone(&config)).await {
        Ok(client) => {
            info!("Discord client created successfully");
            client
        }
        Err(e) => {
            tracing::error!("Failed to create Discord client: {}", e);
            return Ok(());
        }
    };

    // Connect to Discord
    if let Err(e) = discord_client.connect().await {
        tracing::error!("Failed to connect to Discord: {}", e);
        return Ok(());
    }

    // Start Discord client (this will block until shutdown)
    info!("Starting Discord client...");
    if let Err(e) = discord_client.start().await {
        tracing::error!("Discord client error: {}", e);
    }

    info!("DROAS Bot shutting down...");

    Ok(())
}
