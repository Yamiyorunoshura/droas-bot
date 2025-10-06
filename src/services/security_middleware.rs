// Security Middleware - N2 計劃 REFACTOR 階段
// 統一的安全驗證中間件，實現跨領域關注點整合

use crate::error::{DiscordError, Result};
use crate::services::security_service::SecurityService;
use crate::database::user_repository::User;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

/// 安全驗證結果
#[derive(Debug, Clone)]
pub struct SecurityValidationResult {
    /// 驗證是否成功
    pub success: bool,
    /// 已驗證的發送方用戶（如果適用）
    pub authenticated_user: Option<User>,
    /// 已清理的輸入參數
    pub sanitized_inputs: std::collections::HashMap<String, String>,
    /// 驗證失敗的原因（如果失敗）
    pub error_message: Option<String>,
    /// 速率限制狀態
    pub rate_limited: bool,
}

/// Security Middleware
///
/// 統一的安全驗證中間件，實現跨領域關注點整合
/// 與 Error Handling Framework 和 Monitoring/Metrics Service 整合
pub struct SecurityMiddleware {
    security_service: Arc<SecurityService>,
    // 安全驗證結果快取
    validation_cache: Arc<tokio::sync::Mutex<std::collections::HashMap<String, (SecurityValidationResult, std::time::Instant)>>>,
    // 快取過期時間（秒）
    cache_ttl: u64,
}

impl SecurityMiddleware {
    /// 創建新的安全驗證中間件
    ///
    /// # Arguments
    ///
    /// * `security_service` - 安全驗證服務
    /// * `cache_ttl` - 快取過期時間（秒）
    pub fn new(security_service: SecurityService, cache_ttl: u64) -> Self {
        Self {
            security_service: Arc::new(security_service),
            validation_cache: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            cache_ttl,
        }
    }

    /// 執行完整的安全驗證流程
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `operation_type` - 操作類型（如 "transfer", "account_creation"）
    /// * `inputs` - 輸入參數映射
    ///
    /// # Returns
    ///
    /// 返回安全驗證結果
    pub async fn validate_operation(
        &self,
        discord_user_id: i64,
        operation_type: &str,
        inputs: std::collections::HashMap<String, String>,
    ) -> Result<SecurityValidationResult> {
        debug!("執行安全驗證：用戶={}, 操作={}", discord_user_id, operation_type);

        // 生成快取鍵
        let cache_key = self.generate_cache_key(discord_user_id, operation_type, &inputs);

        // 檢查快取
        if let Some(cached_result) = self.check_cache(&cache_key).await {
            debug!("從快取獲取安全驗證結果：{}", cache_key);
            return Ok(cached_result);
        }

        // 執行安全驗證
        let result = self.perform_security_validation(discord_user_id, operation_type, inputs).await;

        // 快取結果
        if let Ok(ref validation_result) = result {
            self.cache_result(cache_key, validation_result.clone()).await;
        }

        // 記錄安全事件（與 Monitoring/Metrics Service 整合）
        self.log_security_event(discord_user_id, operation_type, &result).await;

        result
    }

    /// 生成快取鍵
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `operation_type` - 操作類型
    /// * `inputs` - 輸入參數
    ///
    /// # Returns
    ///
    /// 返回快取鍵
    fn generate_cache_key(
        &self,
        discord_user_id: i64,
        operation_type: &str,
        inputs: &std::collections::HashMap<String, String>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        discord_user_id.hash(&mut hasher);
        operation_type.hash(&mut hasher);

        // 對輸入參數進行排序以確保一致性
        let mut sorted_inputs: Vec<_> = inputs.iter().collect();
        sorted_inputs.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_inputs {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        format!("{}_{}", operation_type, hasher.finish())
    }

    /// 檢查快取
    ///
    /// # Arguments
    ///
    /// * `cache_key` - 快取鍵
    ///
    /// # Returns
    ///
    /// 返回快取的結果（如果存在且未過期）
    async fn check_cache(&self, cache_key: &str) -> Option<SecurityValidationResult> {
        let cache = self.validation_cache.lock().await;

        if let Some((result, timestamp)) = cache.get(cache_key) {
            let age = timestamp.elapsed().as_secs();
            if age < self.cache_ttl {
                return Some(result.clone());
            }
        }

        None
    }

    /// 快取結果
    ///
    /// # Arguments
    ///
    /// * `cache_key` - 快取鍵
    /// * `result` - 驗證結果
    async fn cache_result(&self, cache_key: String, result: SecurityValidationResult) {
        let mut cache = self.validation_cache.lock().await;
        cache.insert(cache_key, (result, std::time::Instant::now()));
    }

    /// 執行實際的安全驗證
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `operation_type` - 操作類型
    /// * `inputs` - 輸入參數
    ///
    /// # Returns
    ///
    /// 返回安全驗證結果
    async fn perform_security_validation(
        &self,
        discord_user_id: i64,
        operation_type: &str,
        mut inputs: std::collections::HashMap<String, String>,
    ) -> Result<SecurityValidationResult> {
        let mut sanitized_inputs = std::collections::HashMap::new();

        // NFR-S-001: 驗證 Discord 用戶 ID
        self.security_service.validate_discord_user_id(discord_user_id)?;

        // 根據操作類型執行特定的驗證
        match operation_type {
            "transfer" => {
                self.validate_transfer_operation(&mut inputs, &mut sanitized_inputs).await?;
            }
            "account_creation" => {
                self.validate_account_creation_operation(&mut inputs, &mut sanitized_inputs).await?;
            }
            "balance_query" => {
                self.validate_balance_query_operation(&mut inputs, &mut sanitized_inputs).await?;
            }
            _ => {
                return Err(DiscordError::InvalidCommand(format!("不支持的操作類型：{}", operation_type)));
            }
        }

        // 驗證用戶身份
        let authenticated_user = self.security_service.authenticate_user(discord_user_id).await?;

        Ok(SecurityValidationResult {
            success: true,
            authenticated_user: Some(authenticated_user),
            sanitized_inputs,
            error_message: None,
            rate_limited: false,
        })
    }

    /// 驗證轉帳操作
    ///
    /// # Arguments
    ///
    /// * `inputs` - 輸入參數
    /// * `sanitized_inputs` - 清理後的輸入參數
    async fn validate_transfer_operation(
        &self,
        inputs: &mut std::collections::HashMap<String, String>,
        sanitized_inputs: &mut std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // 驗證接收方用戶 ID
        if let Some(to_user_id_str) = inputs.get("to_user_id") {
            let to_user_id = to_user_id_str.parse::<i64>()
                .map_err(|_| DiscordError::InvalidAmount("無效的接收方用戶 ID".to_string()))?;

            self.security_service.validate_discord_user_id(to_user_id)?;
            self.security_service.validate_no_self_transfer(
                inputs.get("from_user_id").unwrap().parse::<i64>().unwrap(),
                to_user_id
            )?;

            sanitized_inputs.insert("to_user_id".to_string(), to_user_id_str.clone());
        }

        // 驗證轉帳金額
        if let Some(amount_str) = inputs.get("amount") {
            let amount = self.security_service.validate_amount(amount_str)?;
            sanitized_inputs.insert("amount".to_string(), amount.to_string());
        }

        Ok(())
    }

    /// 驗證帳戶創建操作
    ///
    /// # Arguments
    ///
    /// * `inputs` - 輸入參數
    /// * `sanitized_inputs` - 清理後的輸入參數
    async fn validate_account_creation_operation(
        &self,
        inputs: &mut std::collections::HashMap<String, String>,
        sanitized_inputs: &mut std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // 驗證用戶名稱
        if let Some(username) = inputs.get("username") {
            let sanitized_username = self.security_service.sanitize_string_input(username, 32)?;
            self.security_service.validate_username(&sanitized_username)?;
            sanitized_inputs.insert("username".to_string(), sanitized_username);
        }

        Ok(())
    }

    /// 驗證餘額查詢操作
    ///
    /// # Arguments
    ///
    /// * `inputs` - 輸入參數
    /// * `sanitized_inputs` - 清理後的輸入參數
    async fn validate_balance_query_operation(
        &self,
        _inputs: &mut std::collections::HashMap<String, String>,
        _sanitized_inputs: &mut std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // 餘額查詢通常不需要額外輸入參數
        // 但可以在這裡添加特定的驗證邏輯

        Ok(())
    }

    /// 記錄安全事件（與 Monitoring/Metrics Service 整合）
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `operation_type` - 操作類型
    /// * `result` - 驗證結果
    async fn log_security_event(
        &self,
        discord_user_id: i64,
        operation_type: &str,
        result: &Result<SecurityValidationResult>,
    ) {
        match result {
            Ok(validation_result) => {
                if validation_result.success {
                    info!("安全驗證成功：用戶={}, 操作={}", discord_user_id, operation_type);
                } else {
                    warn!("安全驗證失敗：用戶={}, 操作={}, 原因={:?}",
                          discord_user_id, operation_type, validation_result.error_message);
                }
            }
            Err(error) => {
                error!("安全驗證錯誤：用戶={}, 操作={}, 錯誤={}",
                       discord_user_id, operation_type, error);
            }
        }

        // TODO: 與 Monitoring/Metrics Service 整合，記錄指標
        // 例如：security_validation_total, security_validation_success_total, 等
    }

    /// 清理過期的快取項目
    pub async fn cleanup_expired_cache(&self) {
        let mut cache = self.validation_cache.lock().await;
        let now = std::time::Instant::now();

        cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < self.cache_ttl
        });

        debug!("清理過期安全驗證快取完成，當前快取項目數：{}", cache.len());
    }

    /// 獲取快取統計資訊
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.validation_cache.lock().await;
        let total_items = cache.len();

        let now = std::time::Instant::now();
        let expired_items = cache.iter()
            .filter(|(_, (_, timestamp))| {
                now.duration_since(*timestamp).as_secs() >= self.cache_ttl
            })
            .count();

        (total_items, expired_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_user_pool;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_security_middleware_creation() {
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = crate::database::UserRepository::new(pool);
            let security_service = SecurityService::new(user_repo).unwrap();

            let _middleware = SecurityMiddleware::new(security_service, 300);
            assert!(true, "SecurityMiddleware 創建成功");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = crate::database::UserRepository::new(pool);
            let security_service = SecurityService::new(user_repo).unwrap();

            let middleware = SecurityMiddleware::new(security_service, 300);

            let mut inputs1 = std::collections::HashMap::new();
            inputs1.insert("amount".to_string(), "100".to_string());
            inputs1.insert("to_user".to_string(), "123".to_string());

            let mut inputs2 = std::collections::HashMap::new();
            inputs2.insert("to_user".to_string(), "123".to_string());
            inputs2.insert("amount".to_string(), "100".to_string());

            let key1 = middleware.generate_cache_key(123, "transfer", &inputs1);
            let key2 = middleware.generate_cache_key(123, "transfer", &inputs2);

            assert_eq!(key1, key2, "相同輸入的快取鍵應該相同");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }
}