use droas_bot::BotManager;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting DROAS Bot Manager...");
    
    let bot_manager = BotManager::new();
    
    info!("Bot Manager initialized successfully");
    
    // Keep the application running
    tokio::signal::ctrl_c().await.unwrap();
    
    info!("Shutting down...");
}
