//! 共享HTTP服務
//! 
//! 提供統一的HTTP客戶端和下載功能，供所有命令處理器使用。

use crate::discord::commands::framework::{CommandResult, CommandError};
use reqwest::{Client, Response};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn, error};

/// HTTP服務配置
#[derive(Debug, Clone)]
pub struct HttpServiceConfig {
    /// 請求超時時間
    pub timeout: Duration,
    /// 用戶代理字符串
    pub user_agent: String,
    /// 最大重試次數
    pub max_retries: u32,
    /// 重試延迟基數（毫秒）
    pub retry_base_delay: u64,
    /// 最大文件下載大小
    pub max_download_size: usize,
}

impl Default for HttpServiceConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: "DROAS-Bot/0.1.0".to_string(),
            max_retries: 3,
            retry_base_delay: 1000, // 1秒
            max_download_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// 共享HTTP服務
#[derive(Clone)]
pub struct HttpService {
    client: Client,
    config: HttpServiceConfig,
}

impl HttpService {
    /// 創建新的HTTP服務
    pub fn new(config: HttpServiceConfig) -> CommandResult<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| CommandError::ExecutionFailed(format!("無法創建HTTP客戶端: {}", e)))?;

        Ok(Self { client, config })
    }

    /// 使用默認配置創建HTTP服務
    pub fn with_default_config() -> CommandResult<Self> {
        Self::new(HttpServiceConfig::default())
    }

    /// 下載數據（帶重試機制）
    pub async fn download_data(&self, url: &str) -> CommandResult<Vec<u8>> {
        debug!("開始下載數據: {}", url);
        
        for attempt in 0..=self.config.max_retries {
            match self.download_attempt(url).await {
                Ok(data) => {
                    debug!("成功下載數據: {} bytes (嘗試 {})", data.len(), attempt + 1);
                    return Ok(data);
                }
                Err(e) => {
                    if attempt == self.config.max_retries {
                        error!("下載失敗，已達最大重試次數: {}", e);
                        return Err(e);
                    }
                    
                    if !self.is_retryable_error(&e) {
                        warn!("遇到不可重試錯誤: {}", e);
                        return Err(e);
                    }
                    
                    let delay = self.calculate_retry_delay(attempt);
                    debug!("下載失敗 (嘗試 {}): {}，{} 毫秒後重試", attempt + 1, e, delay);
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
        
        Err(CommandError::ExecutionFailed("未知錯誤：重試循環意外結束".to_string()))
    }

    /// 驗證URL安全性
    pub fn validate_url(&self, url: &str) -> CommandResult<()> {
        let parsed_url = url::Url::parse(url)
            .map_err(|_| CommandError::InvalidArguments("無效的URL格式".to_string()))?;
        
        // 只允許HTTPS
        if parsed_url.scheme() != "https" {
            return Err(CommandError::InvalidArguments(
                "出於安全考慮，只支援HTTPS網址".to_string(),
            ));
        }
        
        // 檢查是否為本地地址（安全考慮）
        if let Some(host_str) = parsed_url.host_str() {
            if self.is_local_address(host_str) {
                return Err(CommandError::InvalidArguments(
                    "不允許存取本地地址".to_string(),
                ));
            }
        }
        
        Ok(())
    }

    /// 下載圖片數據（帶內容類型檢查）
    pub async fn download_image(&self, url: &str) -> CommandResult<Vec<u8>> {
        debug!("下載圖片數據: {}", url);
        
        // 先驗證URL
        self.validate_url(url)?;
        
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("HTTP請求失敗: {}", e)))?;

        // 檢查響應狀態
        if !response.status().is_success() {
            return Err(CommandError::ExecutionFailed(format!(
                "下載失敗，HTTP狀態: {}",
                response.status()
            )));
        }

        // 檢查內容類型
        self.validate_image_content_type(&response)?;
        
        // 檢查內容長度
        self.validate_content_length(&response)?;

        let data = response
            .bytes()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("讀取響應數據失敗: {}", e)))?
            .to_vec();

        // 最終大小檢查
        if data.len() > self.config.max_download_size {
            return Err(CommandError::InvalidArguments(format!(
                "下載的檔案過大: {} bytes (最大允許: {} bytes)",
                data.len(),
                self.config.max_download_size
            )));
        }

        debug!("成功下載圖片數據: {} bytes", data.len());
        Ok(data)
    }

    /// 單次下載嘗試
    async fn download_attempt(&self, url: &str) -> CommandResult<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("HTTP請求失敗: {}", e)))?;

        if !response.status().is_success() {
            return Err(CommandError::ExecutionFailed(format!(
                "HTTP錯誤: {}",
                response.status()
            )));
        }

        // 檢查內容長度
        self.validate_content_length(&response)?;

        let data = response
            .bytes()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("讀取響應失敗: {}", e)))?
            .to_vec();

        if data.len() > self.config.max_download_size {
            return Err(CommandError::InvalidArguments(format!(
                "檔案過大: {} bytes",
                data.len()
            )));
        }

        Ok(data)
    }

    /// 驗證圖片內容類型
    fn validate_image_content_type(&self, response: &Response) -> CommandResult<()> {
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type
                .to_str()
                .unwrap_or("");
                
            if !content_type_str.starts_with("image/") {
                return Err(CommandError::InvalidArguments(format!(
                    "URL指向非圖片內容，Content-Type: {}",
                    content_type_str
                )));
            }
            
            // 檢查是否為支援的圖片類型
            match content_type_str {
                "image/png" | "image/jpeg" | "image/jpg" => Ok(()),
                _ => Err(CommandError::InvalidArguments(format!(
                    "不支援的圖片類型: {}，只支援PNG和JPEG",
                    content_type_str
                ))),
            }
        } else {
            warn!("響應缺少Content-Type標頭，無法驗證圖片類型");
            Ok(()) // 允許缺少Content-Type，稍後通過檔案內容檢測
        }
    }

    /// 驗證內容長度
    fn validate_content_length(&self, response: &Response) -> CommandResult<()> {
        if let Some(content_length) = response.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > self.config.max_download_size {
                        return Err(CommandError::InvalidArguments(format!(
                            "檔案過大: {} bytes (最大允許: {} bytes)",
                            length,
                            self.config.max_download_size
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// 檢查是否為本地地址
    fn is_local_address(&self, host: &str) -> bool {
        // IPv4本地地址
        if host == "localhost" 
            || host == "127.0.0.1" 
            || host.starts_with("192.168.") 
            || host.starts_with("10.") 
            || host.starts_with("172.") {
            return true;
        }
        
        // IPv6本地地址
        if host == "::1" || host.starts_with("fe80:") {
            return true;
        }
        
        false
    }

    /// 檢查錯誤是否可重試
    fn is_retryable_error(&self, error: &CommandError) -> bool {
        match error {
            CommandError::ExecutionFailed(msg) => {
                // 網絡相關錯誤通常可以重試
                msg.contains("網絡") 
                    || msg.contains("連接") 
                    || msg.contains("超時")
                    || msg.contains("HTTP請求失敗")
                    || msg.contains("讀取響應失敗")
            }
            CommandError::InvalidArguments(_) => false, // 參數錯誤不可重試
            CommandError::InsufficientPermissions(_) => false, // 權限錯誤不可重試
            _ => false,
        }
    }

    /// 計算重試延遲時間
    fn calculate_retry_delay(&self, attempt: u32) -> u64 {
        // 指數退避：1s, 2s, 4s
        let base_delay = self.config.retry_base_delay * (2_u64.pow(attempt));
        // 最大延遲10秒
        base_delay.min(10000)
    }

    /// 獲取HTTP客戶端（用於高級操作）
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 獲取配置
    pub fn config(&self) -> &HttpServiceConfig {
        &self.config
    }
}

/// 創建共享HTTP服務實例
pub fn create_shared_http_service() -> CommandResult<Arc<HttpService>> {
    let service = HttpService::with_default_config()?;
    Ok(Arc::new(service))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_service_creation() {
        let config = HttpServiceConfig::default();
        let service = HttpService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_url_validation() {
        let service = HttpService::with_default_config().unwrap();
        
        // 有效的HTTPS URL
        assert!(service.validate_url("https://example.com/image.png").is_ok());
        
        // 無效的HTTP URL
        assert!(service.validate_url("http://example.com/image.png").is_err());
        
        // 本地地址
        assert!(service.validate_url("https://localhost/image.png").is_err());
        assert!(service.validate_url("https://127.0.0.1/image.png").is_err());
        assert!(service.validate_url("https://192.168.1.1/image.png").is_err());
        
        // 無效的URL格式
        assert!(service.validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_is_local_address() {
        let service = HttpService::with_default_config().unwrap();
        
        assert!(service.is_local_address("localhost"));
        assert!(service.is_local_address("127.0.0.1"));
        assert!(service.is_local_address("192.168.1.1"));
        assert!(service.is_local_address("10.0.0.1"));
        assert!(service.is_local_address("::1"));
        assert!(service.is_local_address("fe80::1"));
        
        assert!(!service.is_local_address("example.com"));
        assert!(!service.is_local_address("8.8.8.8"));
        assert!(!service.is_local_address("2001:4860:4860::8888"));
    }

    #[test]
    fn test_retry_delay_calculation() {
        let service = HttpService::with_default_config().unwrap();
        
        assert_eq!(service.calculate_retry_delay(0), 1000); // 1s
        assert_eq!(service.calculate_retry_delay(1), 2000); // 2s
        assert_eq!(service.calculate_retry_delay(2), 4000); // 4s
        assert_eq!(service.calculate_retry_delay(10), 10000); // 最大10s
    }

    #[test]
    fn test_is_retryable_error() {
        let service = HttpService::with_default_config().unwrap();
        
        // 可重試的錯誤
        let network_error = CommandError::ExecutionFailed("網絡連接失敗".to_string());
        assert!(service.is_retryable_error(&network_error));
        
        let timeout_error = CommandError::ExecutionFailed("連接超時".to_string());
        assert!(service.is_retryable_error(&timeout_error));
        
        // 不可重試的錯誤
        let arg_error = CommandError::InvalidArguments("無效參數".to_string());
        assert!(!service.is_retryable_error(&arg_error));
        
        let perm_error = CommandError::InsufficientPermissions("權限不足".to_string());
        assert!(!service.is_retryable_error(&perm_error));
    }
}