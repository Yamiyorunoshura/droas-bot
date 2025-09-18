//! Discord API 客戶端
//! 
//! 提供與 Discord REST API 的真實集成，包括重試機制和錯誤處理。

use crate::error::{DroasError, DroasResult};
use crate::discord::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Discord API 端點基礎 URL
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord 訊息創建請求結構
#[derive(Debug, Clone, Serialize)]
pub struct CreateMessageRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<MessageEmbed>>,
}

/// Discord 嵌入訊息結構（簡化版）
#[derive(Debug, Clone, Serialize)]
pub struct MessageEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<u32>,
}

/// Discord API 錯誤回應結構
#[derive(Debug, Deserialize)]
pub struct DiscordApiError {
    pub code: u32,
    pub message: String,
}

/// Discord API 速率限制資訊
#[derive(Debug)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_after: f64,
    pub bucket: Option<String>,
}

/// Discord API 客戶端
#[derive(Clone)]
pub struct DiscordApiClient {
    client: Client,
    bot_token: String,
    /// 熔斷器保護 API 調用
    circuit_breaker: Arc<CircuitBreaker>,
}

impl DiscordApiClient {
    /// 創建新的 Discord API 客戶端
    /// 
    /// # Arguments
    /// 
    /// * `bot_token` - Discord Bot Token（應該以 "Bot " 開頭）
    pub fn new(bot_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("DROAS-Bot/0.1.0")
            .build()
            .expect("無法創建 HTTP 客戶端");

        // 創建針對 Discord API 的熔斷器配置
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 3,  // 3 次失敗後觸發熔斷
            recovery_timeout: Duration::from_secs(30), // 30 秒後嘗試恢復
            success_threshold: 2,  // 2 次成功後關閉熔斷器
            request_timeout: Duration::from_secs(10), // 10 秒請求超時
        };

        Self {
            client,
            bot_token,
            circuit_breaker: Arc::new(CircuitBreaker::new(circuit_config)),
        }
    }

    /// 使用自定義熔斷器配置創建 Discord API 客戶端
    pub fn with_circuit_breaker(bot_token: String, circuit_config: CircuitBreakerConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("DROAS-Bot/0.1.0")
            .build()
            .expect("無法創建 HTTP 客戶端");

        Self {
            client,
            bot_token,
            circuit_breaker: Arc::new(CircuitBreaker::new(circuit_config)),
        }
    }

    /// 發送訊息到指定頻道（受熔斷器保護）
    /// 
    /// # Arguments
    /// 
    /// * `channel_id` - 頻道 ID
    /// * `message` - 訊息內容
    /// 
    /// # Returns
    /// 
    /// 發送結果，成功時返回訊息 ID
    pub async fn send_message(&self, channel_id: u64, message: &str) -> DroasResult<String> {
        debug!("準備使用熔斷器保護發送 Discord 訊息到頻道 {}: {}", channel_id, 
               message.chars().take(50).collect::<String>());

        // 使用熔斷器保護的 API 調用
        let result = self.circuit_breaker.execute(async {
            self.send_message_with_retry(channel_id, message).await
        }).await;

        match result {
            Ok(message_id) => {
                info!("成功發送 Discord 訊息到頻道 {}: message_id={}", channel_id, message_id);
                Ok(message_id)
            }
            Err(CircuitBreakerError::CircuitOpen) => {
                error!("熔斷器開啟，無法發送訊息到頻道 {}", channel_id);
                Err(DroasError::discord("系統過載，當前無法發送 Discord 訊息。請稍後再試。"))
            }
            Err(CircuitBreakerError::Timeout) => {
                error!("發送訊息到頻道 {} 超時", channel_id);
                Err(DroasError::discord("訊息發送超時"))
            }
            Err(CircuitBreakerError::OperationFailed(e)) => {
                error!("發送訊息到頻道 {} 失敗: {}", channel_id, e);
                Err(e)
            }
        }
    }

    /// 帶重試機制的訊息發送方法（內部使用）
    async fn send_message_with_retry(&self, channel_id: u64, message: &str) -> DroasResult<String> {
        let url = format!("{}/channels/{}/messages", DISCORD_API_BASE, channel_id);
        
        let request_body = CreateMessageRequest {
            content: message.to_string(),
            embeds: None,
        };

        // 實作重試機制
        let mut attempt = 0;
        const MAX_RETRIES: u32 = 3;
        
        while attempt <= MAX_RETRIES {
            match self.send_message_attempt(&url, &request_body).await {
                Ok(message_id) => {
                    debug!("成功發送 Discord 訊息 (嘗試 {}): message_id={}", attempt + 1, message_id);
                    return Ok(message_id);
                }
                Err(e) => {
                    // 檢查是否為不可重試的錯誤
                    if !self.is_retryable_error(&e) {
                        warn!("遇到不可重試錯誤，終止重試: {}", e);
                        return Err(e);
                    }
                    
                    if attempt == MAX_RETRIES {
                        warn!("發送 Discord 訊息失敗，已達最大重試次數: {}", e);
                        return Err(e);
                    }
                    
                    debug!("發送 Discord 訊息失敗 (嘗試 {}): {}，準備重試...", attempt + 1, e);
                    
                    // 指數退避延遲加上隨機抖動
                    let delay = self.calculate_backoff_delay(attempt);
                    debug!("重試延遲 {} 毫秒", delay.as_millis());
                    sleep(delay).await;
                    
                    attempt += 1;
                }
            }
        }

        Err(DroasError::discord("未知錯誤：重試循環意外結束"))
    }

    /// 單次發送訊息嘗試
    async fn send_message_attempt(&self, url: &str, request_body: &CreateMessageRequest) -> DroasResult<String> {
        let response = self.client
            .post(url)
            .header("Authorization", &self.bot_token)
            .header("Content-Type", "application/json")
            .json(request_body)
            .send()
            .await
            .map_err(|e| DroasError::network(format!("HTTP 請求失敗: {}", e)))?;

        self.handle_response(response).await
    }

    /// 處理 Discord API 回應
    async fn handle_response(&self, response: Response) -> DroasResult<String> {
        let status = response.status();
        
        // 檢查速率限制
        if let Some(rate_limit_info) = self.extract_rate_limit_info(&response) {
            debug!("速率限制資訊: 剩餘 {}/{}, 重置時間 {}s", 
                   rate_limit_info.remaining, rate_limit_info.limit, rate_limit_info.reset_after);
        }

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                // 成功回應，解析訊息 ID
                let response_text = response.text().await
                    .map_err(|e| DroasError::network(format!("讀取回應失敗: {}", e)))?;
                
                // 嘗試解析 JSON 並提取訊息 ID
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(id) = json_value.get("id").and_then(|v| v.as_str()) {
                        return Ok(id.to_string());
                    }
                }
                
                warn!("無法從 Discord API 回應中解析訊息 ID: {}", response_text);
                Ok("unknown".to_string())
            }
            StatusCode::TOO_MANY_REQUESTS => {
                // 速率限制，需要等待
                let retry_after = self.extract_retry_after(response).await?;
                warn!("觸發 Discord API 速率限制，需等待 {} 秒", retry_after);
                
                sleep(Duration::from_secs_f64(retry_after)).await;
                Err(DroasError::discord("速率限制"))
            }
            StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await.unwrap_or_default();
                error!("Discord API 認證失敗: {}", error_text);
                Err(DroasError::discord("認證失敗，請檢查 Bot Token"))
            }
            StatusCode::FORBIDDEN => {
                let error_text = response.text().await.unwrap_or_default();
                error!("Discord API 權限不足: {}", error_text);
                Err(DroasError::discord("權限不足，請檢查 Bot 權限設定"))
            }
            StatusCode::NOT_FOUND => {
                error!("Discord 頻道未找到或 Bot 無權限訪問");
                Err(DroasError::discord("頻道未找到或無權限訪問"))
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                error!("Discord API 請求格式錯誤: {}", error_text);
                Err(DroasError::discord(format!("請求格式錯誤: {}", error_text)))
            }
            status if status.is_server_error() => {
                let error_text = response.text().await.unwrap_or_default();
                warn!("Discord API 伺服器錯誤 ({}): {}", status, error_text);
                Err(DroasError::discord(format!("伺服器錯誤: {}", status)))
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                error!("未預期的 Discord API 回應 ({}): {}", status, error_text);
                Err(DroasError::discord(format!("未預期的 API 回應: {}", status)))
            }
        }
    }

    /// 提取速率限制資訊
    fn extract_rate_limit_info(&self, response: &Response) -> Option<RateLimitInfo> {
        let headers = response.headers();
        
        let limit = headers.get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())?;
            
        let remaining = headers.get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())?;
            
        let reset_after = headers.get("x-ratelimit-reset-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())?;
            
        let bucket = headers.get("x-ratelimit-bucket")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        Some(RateLimitInfo {
            limit,
            remaining,
            reset_after,
            bucket,
        })
    }

    /// 提取重試等待時間
    async fn extract_retry_after(&self, mut response: Response) -> DroasResult<f64> {
        // 先嘗試從 headers 獲取
        if let Some(retry_after) = response.headers().get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok()) {
            return Ok(retry_after);
        }

        // 如果 headers 中沒有，嘗試從 JSON body 獲取
        if let Ok(json_text) = response.text().await {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&json_text) {
                if let Some(retry_after) = json_value.get("retry_after").and_then(|v| v.as_f64()) {
                    return Ok(retry_after);
                }
            }
        }

        // 預設等待時間
        warn!("無法解析 retry_after，使用預設值 1 秒");
        Ok(1.0)
    }

    /// 驗證 Bot Token 格式
    pub fn validate_token(&self) -> DroasResult<()> {
        if !self.bot_token.starts_with("Bot ") {
            return Err(DroasError::validation("Bot Token 必須以 'Bot ' 開頭"));
        }
        
        if self.bot_token.len() < 10 {
            return Err(DroasError::validation("Bot Token 過短"));
        }
        
        Ok(())
    }

    /// 獲取熔斷器統計資訊
    pub async fn get_circuit_breaker_stats(&self) -> crate::discord::circuit_breaker::CircuitBreakerStats {
        self.circuit_breaker.get_stats().await
    }

    /// 獲取熔斷器狀態
    pub fn get_circuit_breaker_state(&self) -> crate::discord::circuit_breaker::CircuitState {
        self.circuit_breaker.get_state()
    }

    /// 手動重置熔斷器
    pub async fn reset_circuit_breaker(&self) {
        info!("手動重置 Discord API 熔斷器");
        self.circuit_breaker.reset().await;
    }

    /// 檢查錯誤是否可以重試
    fn is_retryable_error(&self, error: &DroasError) -> bool {
        match error {
            // 網絡錯誤通常可以重試
            DroasError::Network(_) => true,
            // Discord 錯誤需要根據內容判斷
            DroasError::Discord(msg) => {
                // 速率限制可以重試
                if msg.contains("速率限制") {
                    true
                }
                // 伺服器錯誤可以重試
                else if msg.contains("伺服器錯誤") {
                    true
                }
                // 超時可以重試
                else if msg.contains("超時") {
                    true
                }
                // 認證和權限錯誤不可重試
                else if msg.contains("認證") || msg.contains("權限") {
                    false
                }
                // 頻道未找到不可重試
                else if msg.contains("頻道未找到") {
                    false
                }
                // 請求格式錯誤不可重試
                else if msg.contains("請求格式錯誤") {
                    false
                }
                // 其他 Discord 錯誤預設不重試
                else {
                    false
                }
            }
            // 其他錯誤類型不重試
            _ => false,
        }
    }

    /// 計算指數退避延遲（加上隨機抖動）
    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        // 基本指數退避：1秒、2秒、4秒
        let base_delay_ms = 1000 * (2_u64.pow(attempt));
        
        // 最大延遲限制為 30 秒
        let base_delay_ms = base_delay_ms.min(30_000);
        
        // 添加 25% 的隨機抖動以避免驚群效應
        let jitter = self.generate_jitter(base_delay_ms);
        let final_delay_ms = base_delay_ms + jitter;
        
        Duration::from_millis(final_delay_ms)
    }

    /// 產生隨機抖動（使用系統時間作為種子）
    fn generate_jitter(&self, base_delay_ms: u64) -> u64 {
        // 使用系統時間作為偽隨機數種子
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos() as u64;
        
        // 簡單的線性同餘生成器
        let seed = now.wrapping_mul(1103515245).wrapping_add(12345);
        let random = (seed / 65536) % 32768;
        
        // 返回 0-25% 基本延遲的隨機值
        let jitter_range = base_delay_ms / 4; // 25%
        random % jitter_range
    }
}

impl Default for DiscordApiClient {
    fn default() -> Self {
        // 從環境變數獲取 token，用於測試和開發
        let bot_token = std::env::var("DISCORD_BOT_TOKEN")
            .unwrap_or_else(|_| "Bot your_token_here".to_string());
            
        Self::new(bot_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discord_client_creation() {
        let client = DiscordApiClient::new("Bot test_token_123".to_string());
        assert!(client.validate_token().is_ok());
    }

    #[tokio::test]
    async fn test_invalid_token_validation() {
        let client = DiscordApiClient::new("invalid_token".to_string());
        assert!(client.validate_token().is_err());
    }

    #[test]
    fn test_create_message_request_serialization() {
        let request = CreateMessageRequest {
            content: "Test message".to_string(),
            embeds: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Test message"));
        assert!(!json.contains("embeds"));
    }
}