//! 歡迎訊息處理器
//! 
//! 負責處理新成員加入事件並發送歡迎訊息。

use crate::error::{DroasError, DroasResult};
use crate::database::schema::{GuildConfigService, GuildConfig};
use crate::discord::api_client::DiscordApiClient;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use tokio::time::Instant;

/// 歡迎訊息處理器
///
/// 負責處理成員加入事件，包括：
/// - 查詢公會配置
/// - 生成歡迎訊息
/// - 發送歡迎訊息到 Discord
#[derive(Clone)]
pub struct WelcomeHandler {
    /// 公會配置服務（可選，用於更完整的實作）
    guild_service: Option<GuildConfigService>,
    /// Discord API 客戶端（用於發送訊息）
    discord_client: Option<DiscordApiClient>,
}

impl WelcomeHandler {
    /// 創建新的歡迎訊息處理器
    pub fn new() -> Self {
        Self {
            guild_service: None,
            discord_client: None,
        }
    }

    /// 使用公會配置服務創建處理器
    pub fn with_guild_service(guild_service: GuildConfigService) -> Self {
        Self {
            guild_service: Some(guild_service),
            discord_client: None,
        }
    }

    /// 使用 Discord API 客戶端創建處理器
    pub fn with_discord_client(discord_client: DiscordApiClient) -> Self {
        Self {
            guild_service: None,
            discord_client: Some(discord_client),
        }
    }

    /// 使用全部服務創建處理器
    pub fn with_services(guild_service: GuildConfigService, discord_client: DiscordApiClient) -> Self {
        Self {
            guild_service: Some(guild_service),
            discord_client: Some(discord_client),
        }
    }

    /// 處理成員加入事件
    /// 
    /// # Arguments
    /// 
    /// * `guild_id` - 公會 ID
    /// * `user_id` - 用戶 ID
    /// 
    /// # Returns
    /// 
    /// 處理結果，成功時返回 Ok(())，失敗時返回錯誤
    pub async fn handle_member_join(&self, guild_id: u64, user_id: u64) -> DroasResult<()> {
        let start_time = Instant::now();
        
        info!("開始處理新成員 {} 加入公會 {} 的歡迎訊息", user_id, guild_id);
        
        // 輸入驗證
        if guild_id == 0 {
            warn!("無效的公會 ID: {}", guild_id);
            return Err(DroasError::validation("公會 ID 不能為 0"));
        }
        
        if user_id == 0 {
            warn!("無效的用戶 ID: {}", user_id);
            return Err(DroasError::validation("用戶 ID 不能為 0"));
        }
        
        // 模擬處理邏輯
        let result = self.process_welcome_message(guild_id, user_id).await;
        
        let processing_time = start_time.elapsed();
        debug!("歡迎訊息處理用時: {}ms", processing_time.as_millis());
        
        match result {
            Ok(_) => {
                info!("成功為用戶 {} (公會 {}) 處理歡迎訊息，用時 {}ms", 
                     user_id, guild_id, processing_time.as_millis());
                Ok(())
            }
            Err(e) => {
                error!("處理用戶 {} (公會 {}) 歡迎訊息失敗: {}", user_id, guild_id, e);
                Err(e)
            }
        }
    }

    /// 內部處理歡迎訊息邏輯
    async fn process_welcome_message(&self, guild_id: u64, user_id: u64) -> DroasResult<()> {
        // 1. 查詢公會配置以獲取歡迎頻道 ID
        let welcome_channel_id = if let Some(guild_service) = &self.guild_service {
            let guild_config = self.get_guild_config(guild_service, &guild_id.to_string()).await?;
            if let Some(config) = guild_config {
                debug!("公會 {} 配置查詢結果: welcome_channel_id={}", guild_id, config.welcome_channel_id);
                Some(config.welcome_channel_id.parse::<u64>().map_err(|e| {
                    DroasError::validation(format!("無效的歡迎頻道 ID: {}", e))
                })?)
            } else {
                warn!("公會 {} 未設定配置，無法發送歡迎訊息", guild_id);
                None
            }
        } else {
            warn!("無公會配置服務，無法查詢歡迎頻道");
            None
        };
        
        // 2. 如果没有配置歡迎頻道，直接返回
        let channel_id = match welcome_channel_id {
            Some(id) => id,
            None => {
                info!("公會 {} 沒有配置歡迎頻道，跳過歡迎訊息發送", guild_id);
                return Ok(());
            }
        };
        
        // 3. 生成歡迎訊息內容
        let welcome_message = self.generate_welcome_message(guild_id, user_id).await?;
        debug!("生成歡迎訊息: {}", welcome_message);
        
        // 4. 發送歡迎訊息到 Discord
        self.send_welcome_message(channel_id, user_id, &welcome_message).await?;
        
        Ok(())
    }

    /// 查詢公會配置
    async fn get_guild_config(&self, guild_service: &GuildConfigService, guild_id: &str) -> DroasResult<Option<GuildConfig>> {
        match guild_service.get_guild_config(guild_id).await {
            Ok(config) => Ok(config),
            Err(e) => {
                error!("查詢公會 {} 配置失敗: {}", guild_id, e);
                Err(DroasError::database(format!("查詢公會配置失敗: {}", e)))
            }
        }
    }

    /// 生成歡迎訊息內容
    async fn generate_welcome_message(&self, guild_id: u64, user_id: u64) -> DroasResult<String> {
        // 模擬訊息生成邏輯
        tokio::time::sleep(Duration::from_millis(1)).await; // 模擬處理時間
        
        let message = format!(
            "🎉 歡迎 <@{}> 加入我們的社群！\n\
            感謝您成為公會 {} 的一員，希望您在這裡度過愉快的時光！",
            user_id, guild_id
        );
        
        Ok(message)
    }

    /// 發送歡迎訊息到 Discord 頻道
    async fn send_welcome_message(&self, channel_id: u64, user_id: u64, message: &str) -> DroasResult<()> {
        // 檢查是否有 Discord API 客戶端
        if let Some(discord_client) = &self.discord_client {
            debug!("準備使用 Discord API 發送歡迎訊息到頻道 {}", channel_id);
            
            // 使用真實的 Discord API 客戶端發送訊息
            let message_id = discord_client.send_message(channel_id, message).await?;
            
            info!("歡迎訊息成功發送到頻道 {} 給用戶 {}: message_id={}", 
                 channel_id, user_id, message_id);
                 
            Ok(())
        } else {
            // Fallback: 如果沒有 Discord API 客戶端，使用模擬方式（用於測試）
            warn!("沒有 Discord API 客戶端，使用模擬發送模式");
            
            tokio::time::sleep(Duration::from_millis(5)).await; // 模擬網絡延遲
            debug!("模擬發送歡迎訊息到頻道 {}: {}", channel_id, 
                   message.chars().take(50).collect::<String>());
            
            // 模擬網絡錯誤（很低概率，但用於測試錯誤處理）
            if user_id % 1000 == 0 { // 0.1% 錯誤率
                return Err(DroasError::network("模擬網絡連接失敗"));
            }
            
            info!("模擬歡迎訊息已發送到頻道 {} 給用戶 {}", channel_id, user_id);
            Ok(())
        }
    }
}

impl Default for WelcomeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_welcome_handler_basic() {
        let handler = WelcomeHandler::new();
        let result = handler.handle_member_join(123456789, 987654321).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_welcome_handler_invalid_input() {
        let handler = WelcomeHandler::new();
        
        // 測試無效的公會 ID
        let result = handler.handle_member_join(0, 987654321).await;
        assert!(result.is_err());
        
        // 測試無效的用戶 ID  
        let result = handler.handle_member_join(123456789, 0).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_welcome_message_generation() {
        let handler = WelcomeHandler::new();
        let message = handler.generate_welcome_message(123456789, 987654321).await;
        
        assert!(message.is_ok());
        let message = message.unwrap();
        assert!(message.contains("歡迎"));
        assert!(message.contains("987654321"));
        assert!(message.contains("123456789"));
    }
}
