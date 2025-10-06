use droas_bot::discord_gateway::{DiscordGateway, ConnectionStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discord_api_connection_success() {
        // Given: Bot 配置有效 Discord API Token
        let mut gateway = DiscordGateway::new();

        // When: Bot 啟動
        let result = gateway.connect().await;

        // Then: 成功連接並顯示在線狀態
        assert!(result.is_ok(), "連接應該成功");
        assert_eq!(gateway.get_status().await, ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_discord_api_connection_failure() {
        // Given: Bot 配置無效 Discord API Token
        let mut gateway = DiscordGateway::new();
        gateway.set_invalid_token();

        // When: Bot 嘗試連接
        let result = gateway.connect().await;

        // Then: 連接失敗並返回錯誤
        assert!(result.is_err(), "無效 token 應該導致連接失敗");
        assert_eq!(gateway.get_status().await, ConnectionStatus::Error);
    }

    #[tokio::test]
    async fn test_command_response_time() {
        // Given: Bot 在線且用戶有適當權限
        let mut gateway = DiscordGateway::new();
        gateway.connect().await.expect("連接應該成功");

        let start_time = std::time::Instant::now();

        // When: 用戶發送指令
        let response = gateway.handle_command("!ping").await;

        let elapsed = start_time.elapsed();

        // Then: 在 2 秒內回應
        assert!(response.is_ok(), "指令處理應該成功");
        assert!(elapsed.as_millis() < 2000, "響應時間應該少於 2 秒，實際: {}ms", elapsed.as_millis());
    }

    #[tokio::test]
    async fn test_event_listening() {
        // Given: Bot 已連接
        let mut gateway = DiscordGateway::new();
        gateway.connect().await.expect("連接應該成功");

        // When: 模擬 Discord 事件
        let event_received = gateway.simulate_message_event().await;

        // Then: 事件應該被正確監聽和處理
        assert!(event_received, "應該能夠監聽到 Discord 事件");
    }
}