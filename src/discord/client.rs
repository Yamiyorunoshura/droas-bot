use anyhow::{Context, Result};
use serenity::{
    async_trait,
    client::{Client, Context as SerenityContext, EventHandler},
    model::gateway::{GatewayIntents, Ready},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::gateway::{GatewayManager, GatewayStatus};
use super::rate_limit::RateLimiter;
use crate::config::Config;

/// Discord 客戶端包裝器
/// 
/// 提供高級 Discord API 互動接口，整合 Gateway 連接管理、
/// 速率限制處理和自動重試功能。
pub struct DiscordClient {
    /// Serenity 客戶端實例
    client: Option<Client>,
    /// Gateway 連接管理器
    gateway_manager: Arc<RwLock<GatewayManager>>,
    /// 速率限制管理器
    rate_limiter: Arc<RateLimiter>,
    /// 配置
    config: Arc<Config>,
}

/// Discord 事件處理器
pub struct DroasEventHandler {
    gateway_manager: Arc<RwLock<GatewayManager>>,
}

#[async_trait]
impl EventHandler for DroasEventHandler {
    async fn ready(&self, _ctx: SerenityContext, ready: Ready) {
        info!("Discord bot {} 已連接並準備就緒!", ready.user.name);
        
        // 更新 Gateway 狀態
        let mut manager = self.gateway_manager.write().await;
        manager.set_status(GatewayStatus::Connected);
    }
    
    async fn resume(&self, _ctx: SerenityContext, _: serenity::model::event::ResumedEvent) {
        info!("Discord Gateway 連接已恢復");
        
        let mut manager = self.gateway_manager.write().await;
        manager.set_status(GatewayStatus::Connected);
        manager.increment_reconnect_count();
    }
}

impl DiscordClient {
    /// 創建新的 Discord 客戶端
    /// 
    /// # Arguments
    /// * `config` - 應用程序配置
    /// 
    /// # Returns
    /// * `Ok(DiscordClient)` - 成功創建的客戶端實例
    /// * `Err(...)` - 創建失敗的錯誤信息
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let gateway_manager = Arc::new(RwLock::new(GatewayManager::new()));
        let rate_limiter = Arc::new(RateLimiter::new());
        
        info!("正在初始化 Discord 客戶端...");
        
        Ok(Self {
            client: None,
            gateway_manager,
            rate_limiter,
            config,
        })
    }
    
    /// 初始化並連接到 Discord
    /// 
    /// 建立 WebSocket Gateway 連接，設置事件處理器，並開始監聽事件。
    /// 
    /// # Returns
    /// * `Ok(())` - 成功連接
    /// * `Err(...)` - 連接失敗的錯誤信息
    pub async fn connect(&mut self) -> Result<()> {
        let intents = GatewayIntents::GUILD_MEMBERS 
            | GatewayIntents::GUILDS 
            | GatewayIntents::GUILD_MESSAGES;
            
        info!("正在建立 Discord Gateway 連接...");
        
        // 更新狀態為連接中
        {
            let mut manager = self.gateway_manager.write().await;
            manager.set_status(GatewayStatus::Connecting);
        }
        
        let client = Client::builder(&self.config.discord.token(), intents)
            .event_handler(DroasEventHandler {
                gateway_manager: Arc::clone(&self.gateway_manager),
            })
            .await
            .context("無法創建 Discord 客戶端")?;
            
        self.client = Some(client);
        
        info!("Discord 客戶端初始化完成");
        Ok(())
    }
    
    /// 啟動客戶端並開始處理事件
    /// 
    /// 這是一個阻塞調用，會持續運行直到客戶端關閉或出現錯誤。
    /// 
    /// # Returns
    /// * `Ok(())` - 客戶端正常關閉
    /// * `Err(...)` - 運行時錯誤
    pub async fn start(&mut self) -> Result<()> {
        let client = self.client
            .take()
            .context("客戶端尚未初始化，請先調用 connect()")?;
            
        info!("正在啟動 Discord 客戶端...");
        
        // 在單獨的任務中運行客戶端
        let client_handle = tokio::spawn(async move {
            let mut client = client; // 明確聲明為可變
            if let Err(why) = client.start().await {
                error!("Discord 客戶端錯誤: {:?}", why);
                return Err(anyhow::anyhow!("Discord 客戶端運行失敗: {}", why));
            }
            Ok(())
        });
        
        // 啟動 Gateway 管理器監控
        let gateway_monitor = self.start_gateway_monitor();
        
        // 等待任一任務完成
        tokio::select! {
            result = client_handle => {
                match result {
                    Ok(Ok(())) => {
                        info!("Discord 客戶端正常關閉");
                        Ok(())
                    },
                    Ok(Err(e)) => {
                        error!("Discord 客戶端運行錯誤: {}", e);
                        Err(e)
                    },
                    Err(e) => {
                        error!("Discord 客戶端任務 panic: {}", e);
                        Err(anyhow::anyhow!("Discord 客戶端任務失敗: {}", e))
                    }
                }
            },
            _ = gateway_monitor => {
                info!("Gateway 監控器已停止");
                Ok(())
            }
        }
    }
    
    /// 啟動 Gateway 連接監控
    /// 
    /// 定期檢查連接狀態並處理自動重連邏輯。
    async fn start_gateway_monitor(&self) {
        let gateway_manager = Arc::clone(&self.gateway_manager);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let manager = gateway_manager.read().await;
                let status = manager.get_status();
                let uptime = manager.get_uptime();
                
                match status {
                    GatewayStatus::Connected => {
                        // 記錄正常運行統計
                        if uptime.as_secs() % 300 == 0 { // 每 5 分鐘記錄一次
                            info!("Discord Gateway 正常運行: {:.2} 分鐘", uptime.as_secs_f64() / 60.0);
                        }
                    },
                    GatewayStatus::Disconnected => {
                        warn!("檢測到 Gateway 連接斷開，等待自動重連...");
                    },
                    GatewayStatus::Connecting => {
                        info!("Gateway 正在連接中...");
                    },
                    GatewayStatus::Error(_) => {
                        error!("Gateway 處於錯誤狀態，需要手動干預");
                    }
                }
            }
        });
    }
    
    /// 獲取當前 Gateway 狀態
    pub async fn get_gateway_status(&self) -> GatewayStatus {
        let manager = self.gateway_manager.read().await;
        manager.get_status()
    }
    
    /// 獲取連接統計信息
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let manager = self.gateway_manager.read().await;
        let rate_limiter_stats = self.rate_limiter.get_stats();
        
        ConnectionStats {
            status: manager.get_status(),
            uptime: manager.get_uptime(),
            reconnect_count: manager.get_reconnect_count(),
            rate_limit_hits: rate_limiter_stats.rate_limit_hits,
            requests_made: rate_limiter_stats.requests_made,
        }
    }
}

/// 連接統計信息
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// 當前連接狀態
    pub status: GatewayStatus,
    /// 運行時間
    pub uptime: std::time::Duration,
    /// 重連次數
    pub reconnect_count: u64,
    /// 速率限制觸發次數
    pub rate_limit_hits: u64,
    /// 總請求數
    pub requests_made: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DiscordConfig, DatabaseConfig, AppConfig};
    
    fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            discord: DiscordConfig::test_config(), // 我們需要為測試添加這個方法
            database: DatabaseConfig {
                url: "sqlite://test.db".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            app: AppConfig {
                log_level: "info".to_string(),
                image_cache_dir: "./test_cache".to_string(),
                max_image_size_mb: 5,
            },
        })
    }
    
    #[tokio::test]
    async fn test_client_creation() {
        // 使用測試配置創建客戶端，不進行實際連接
        let config = create_test_config();
        let client = DiscordClient::new(config).await;
        assert!(client.is_ok(), "應該能夠使用測試配置創建 Discord 客戶端");
    }
    
    #[tokio::test]
    async fn test_gateway_status_initial() {
        let config = create_test_config();
        let client = DiscordClient::new(config).await.unwrap();
        let status = client.get_gateway_status().await;
        
        // 初始狀態應該是斷開連接
        assert!(matches!(status, GatewayStatus::Disconnected));
    }
}