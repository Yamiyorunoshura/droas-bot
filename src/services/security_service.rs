// 安全驗證服務 - N2 計劃增強版本 (GREEN 階段)
// 確保所有操作通過 Discord 用戶 ID 驗證，符合 NFR-S-001 安全需求
// 實現輸入驗證和清理功能，符合 NFR-S-002 安全需求

use crate::error::DiscordError;
use crate::database::user_repository::{User, UserRepositoryTrait};
use tracing::{info, warn, error, debug};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 安全驗證服務
///
/// 負責輸入驗證、身份驗證和安全檢查
/// 確保所有操作符合安全要求 (NFR-S-001, NFR-S-002)
pub struct SecurityService {
    // 黑名單用戶 ID
    blacklisted_users: HashSet<i64>,
    // 輸入驗證正則表達式
    username_regex: Regex,
    amount_regex: Regex,
    // 安全驗證正則表達式
    xss_pattern: Regex,
    sql_injection_pattern: Regex,
    malicious_script_pattern: Regex,
    // 用戶資料庫存取
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
    // 速率限制記錄 (用戶 ID -> (請求次數, 最後請求時間))
    rate_limit_records: std::collections::HashMap<i64, (u32, u64)>,
}

impl SecurityService {
    /// 創建新的安全驗證服務
    pub fn new<T>(user_repository: T) -> Result<Self, DiscordError>
    where
        T: UserRepositoryTrait + Send + Sync + 'static,
    {
        // 編譯驗證正則表達式
        let username_regex = Regex::new(r"^[a-zA-Z0-9_\-\s]{2,32}$")
            .map_err(|e| DiscordError::ConfigError(format!("無效的用戶名正則表達式：{}", e)))?;

        let amount_regex = Regex::new(r"^\d+(\.\d{1,2})?$")
            .map_err(|e| DiscordError::ConfigError(format!("無效的金額正則表達式：{}", e)))?;

        // 編譯安全驗證正則表達式
        let xss_pattern = Regex::new(r"(?i)<script[^>]*>.*?</script>|<iframe[^>]*>.*?</iframe>|<object[^>]*>.*?</object>|<embed[^>]*>.*?</embed>|javascript:|on\w+\s*=")
            .map_err(|e| DiscordError::ConfigError(format!("無效的 XSS 檢測正則表達式：{}", e)))?;

        let sql_injection_pattern = Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)\b|\bor\b|\band\b|--|'")
            .map_err(|e| DiscordError::ConfigError(format!("無效的 SQL 注入檢測正則表達式：{}", e)))?;

        let malicious_script_pattern = Regex::new(r"(?i)(eval|alert|confirm|prompt|document\.|window\.|location\.|cookie\s*=|settimeout|setinterval)\s*\(")
            .map_err(|e| DiscordError::ConfigError(format!("無效的惡意腳本檢測正則表達式：{}", e)))?;

        Ok(Self {
            blacklisted_users: HashSet::new(),
            username_regex,
            amount_regex,
            xss_pattern,
            sql_injection_pattern,
            malicious_script_pattern,
            user_repository: Arc::new(user_repository),
            rate_limit_records: std::collections::HashMap::new(),
        })
    }

    /// 驗證 Discord 用戶 ID
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶有效
    pub fn validate_discord_user_id(&self, discord_user_id: i64) -> Result<bool, DiscordError> {
        debug!("驗證 Discord 用戶 ID：{}", discord_user_id);

        // 檢查用戶 ID 是否為正數
        if discord_user_id <= 0 {
            warn!("無效的 Discord 用戶 ID：{}", discord_user_id);
            return Err(DiscordError::InvalidAmount(format!("無效的 Discord 用戶 ID：{}", discord_user_id)));
        }

        // 檢查是否在黑名單中
        if self.blacklisted_users.contains(&discord_user_id) {
            warn!("用戶 {} 在黑名單中", discord_user_id);
            return Err(DiscordError::AccountCreationFailed("用戶已被封鎖".to_string()));
        }

        info!("用戶 {} 驗證通過", discord_user_id);
        Ok(true)
    }

    /// 驗證用戶名稱
    ///
    /// # Arguments
    ///
    /// * `username` - 用戶名稱
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶名稱有效
    pub fn validate_username(&self, username: &str) -> Result<bool, DiscordError> {
        debug!("驗證用戶名稱：{}", username);

        // 檢查長度
        if username.len() < 2 || username.len() > 32 {
            warn!("用戶名稱長度無效：{}", username.len());
            return Err(DiscordError::InvalidAmount("用戶名稱長度必須在 2-32 字符之間".to_string()));
        }

        // 檢查字符有效性
        if !self.username_regex.is_match(username) {
            warn!("用戶名稱包含無效字符：{}", username);
            return Err(DiscordError::InvalidAmount("用戶名稱只能包含字母、數字、下劃線、連字符和空格".to_string()));
        }

        // 檢查是否為空或只包含空格
        if username.trim().is_empty() {
            warn!("用戶名稱為空或只包含空格");
            return Err(DiscordError::InvalidAmount("用戶名稱不能為空".to_string()));
        }

        info!("用戶名稱 {} 驗證通過", username);
        Ok(true)
    }

    /// 驗證金額輸入
    ///
    /// # Arguments
    ///
    /// * `amount_str` - 金額字符串
    ///
    /// # Returns
    ///
    /// 返回 Result<f64>，成功時包含解析後的金額
    pub fn validate_amount(&self, amount_str: &str) -> Result<f64, DiscordError> {
        debug!("驗證金額：{}", amount_str);

        // 檢查格式
        if !self.amount_regex.is_match(amount_str) {
            warn!("金額格式無效：{}", amount_str);
            return Err(DiscordError::InvalidAmount("金額必須是正數，最多支持兩位小數".to_string()));
        }

        // 解析金額
        let amount = amount_str.parse::<f64>()
            .map_err(|_| DiscordError::InvalidAmount("無法解析金額".to_string()))?;

        // 檢查金額範圍
        if amount <= 0.0 {
            warn!("金額必須為正數：{}", amount);
            return Err(DiscordError::InvalidAmount("金額必須大於 0".to_string()));
        }

        if amount > 1_000_000.0 {
            warn!("金額超過上限：{}", amount);
            return Err(DiscordError::InvalidAmount("單筆交易金額不能超過 1,000,000".to_string()));
        }

        info!("金額 {} 驗證通過", amount);
        Ok(amount)
    }

    /// 清理和驗證輸入字符串 (增強版本)
    ///
    /// # Arguments
    ///
    /// * `input` - 輸入字符串
    /// * `max_length` - 最大長度
    ///
    /// # Returns
    ///
    /// 返回清理後的字符串
    pub fn sanitize_string_input(&self, input: &str, max_length: usize) -> Result<String, DiscordError> {
        debug!("清理輸入字符串，長度：{}", input.len());

        // 移除前後空白字符
        let cleaned = input.trim();

        // 檢查長度
        if cleaned.len() > max_length {
            warn!("輸入字符串超過最大長度：{} > {}", cleaned.len(), max_length);
            return Err(DiscordError::InvalidAmount(format!("輸入長度不能超過 {} 字符", max_length)));
        }

        // 檢查是否為空
        if cleaned.is_empty() {
            warn!("輸入字符串為空");
            return Err(DiscordError::InvalidAmount("輸入不能為空".to_string()));
        }

        // 執行安全檢查
        self.perform_security_checks(cleaned)?;

        // 移除潛在的危險字符（用於一般文本輸入）
        let safe_input = cleaned
            .chars()
            .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
            .collect::<String>();

        if safe_input != cleaned {
            warn!("輸入包含非安全字符，已清理");
        }

        info!("輸入字符串清理完成，長度：{}", safe_input.len());
        Ok(safe_input)
    }

    /// 執行全面的安全檢查 (NFR-S-002)
    ///
    /// # Arguments
    ///
    /// * `input` - 要檢查的輸入字符串
    ///
    /// # Returns
    ///
    /// 返回 Result<()>，成功表示輸入通過安全檢查
    fn perform_security_checks(&self, input: &str) -> Result<(), DiscordError> {
        debug!("執行安全檢查：{}", input);

        // XSS 攻擊檢測
        if self.xss_pattern.is_match(input) {
            warn!("檢測到潛在 XSS 攻擊：{}", input);
            return Err(DiscordError::InvalidAmount("輸入包含不安全的 HTML 內容".to_string()));
        }

        // SQL 注入攻擊檢測
        if self.sql_injection_pattern.is_match(input) {
            warn!("檢測到潛在 SQL 注入攻擊：{}", input);
            return Err(DiscordError::InvalidAmount("輸入包含潛在的 SQL 注入".to_string()));
        }

        // 惡意腳本檢測
        if self.malicious_script_pattern.is_match(input) {
            warn!("檢測到潛在惡意腳本：{}", input);
            return Err(DiscordError::InvalidAmount("輸入包含潛在的惡意腳本".to_string()));
        }

        // 檢查是否包含控制字符（除了常見的空白字符）
        if input.chars().any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r')) {
            warn!("輸入包含控制字符：{}", input);
            return Err(DiscordError::InvalidAmount("輸入包含不允許的控制字符".to_string()));
        }

        debug!("安全檢查通過：{}", input);
        Ok(())
    }

    /// 深度清理輸入字符串 (移除所有潛在危險內容)
    ///
    /// # Arguments
    ///
    /// * `input` - 輸入字符串
    /// * `max_length` - 最大長度
    ///
    /// # Returns
    ///
    /// 返回深度清理後的字符串
    pub fn deep_sanitize_input(&self, input: &str, max_length: usize) -> Result<String, DiscordError> {
        debug!("深度清理輸入字符串：{}", input);

        // 基本清理
        let mut cleaned = self.sanitize_string_input(input, max_length)?;

        // 移除 HTML 標籤
        cleaned = self.remove_html_tags(&cleaned);

        // 轉義特殊字符
        cleaned = self.escape_special_characters(&cleaned);

        // 移除 Unicode 控制字符
        cleaned = self.remove_control_characters(&cleaned);

        // 最終長度檢查
        if cleaned.len() > max_length {
            warn!("深度清理後輸入仍然過長，截斷：{} > {}", cleaned.len(), max_length);
            cleaned = cleaned.chars().take(max_length).collect();
        }

        info!("深度清理完成：{}", cleaned);
        Ok(cleaned)
    }

    /// 移除 HTML 標籤
    ///
    /// # Arguments
    ///
    /// * `input` - 輸入字符串
    ///
    /// # Returns
    ///
    /// 返回移除 HTML 標籤後的字符串
    fn remove_html_tags(&self, input: &str) -> String {
        let html_tag_regex = Regex::new(r"<[^>]*>").unwrap();
        html_tag_regex.replace_all(input, "").to_string()
    }

    /// 轉義特殊字符
    ///
    /// # Arguments
    ///
    /// * `input` - 輸入字符串
    ///
    /// # Returns
    ///
    /// 返回轉義特殊字符後的字符串
    fn escape_special_characters(&self, input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// 移除控制字符
    ///
    /// # Arguments
    ///
    /// * `input` - 輸入字符串
    ///
    /// # Returns
    ///
    /// 返回移除控制字符後的字符串
    fn remove_control_characters(&self, input: &str) -> String {
        input.chars().filter(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r')).collect()
    }

    /// 驗證速率限制
    ///
    /// # Arguments
    ///
    /// * `user_id` - 用戶 ID
    /// * `max_requests` - 最大請求次數
    /// * `time_window_seconds` - 時間窗口（秒）
    ///
    /// # Returns
    ///
    /// 返回 Result<()>，成功表示通過速率限制檢查
    pub fn check_rate_limit(&mut self, user_id: i64, max_requests: u32, time_window_seconds: u64) -> Result<(), DiscordError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        debug!("檢查用戶 {} 的速率限制，當前時間：{}", user_id, current_time);

        match self.rate_limit_records.get_mut(&user_id) {
            Some((count, last_request_time)) => {
                // 檢查是否在時間窗口內
                if current_time - *last_request_time < time_window_seconds {
                    *count += 1;
                    if *count > max_requests {
                        warn!("用戶 {} 超過速率限制：{}/{} 秒", user_id, *count, time_window_seconds);
                        return Err(DiscordError::InvalidAmount("請求過於頻繁，請稍後再試".to_string()));
                    }
                } else {
                    // 重置計數器
                    *count = 1;
                    *last_request_time = current_time;
                }
            }
            None => {
                // 首次請求
                self.rate_limit_records.insert(user_id, (1, current_time));
            }
        }

        debug!("用戶 {} 速率限制檢查通過", user_id);
        Ok(())
    }

    /// 清理過期的速率限制記錄
    ///
    /// # Arguments
    ///
    /// * `time_window_seconds` - 時間窗口（秒）
    pub fn cleanup_expired_rate_limits(&mut self, time_window_seconds: u64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.rate_limit_records.retain(|_, (_, last_request_time)| {
            current_time - *last_request_time < time_window_seconds
        });

        debug!("清理過期速率限制記錄完成，當前記錄數：{}", self.rate_limit_records.len());
    }

    /// 驗證自我轉帳防護
    ///
    /// # Arguments
    ///
    /// * `from_user_id` - 發送方用戶 ID
    /// * `to_user_id` - 接收方用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示不是自我轉帳
    pub fn validate_no_self_transfer(&self, from_user_id: i64, to_user_id: i64) -> Result<bool, DiscordError> {
        debug!("檢查自我轉帳：{} -> {}", from_user_id, to_user_id);

        if from_user_id == to_user_id {
            warn!("檢測到自我轉帳嘗試：{}", from_user_id);
            return Err(DiscordError::InvalidAmount("不能轉帳給自己".to_string()));
        }

        info!("自我轉帳檢查通過：{} -> {}", from_user_id, to_user_id);
        Ok(true)
    }

    /// 將用戶添加到黑名單
    ///
    /// # Arguments
    ///
    /// * `user_id` - 要封鎖的用戶 ID
    pub fn add_user_to_blacklist(&mut self, user_id: i64) {
        info!("將用戶 {} 添加到黑名單", user_id);
        self.blacklisted_users.insert(user_id);
    }

    /// 從黑名單中移除用戶
    ///
    /// # Arguments
    ///
    /// * `user_id` - 要解封的用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 bool，true 表示用戶存在於黑名單中並已被移除
    pub fn remove_user_from_blacklist(&mut self, user_id: i64) -> bool {
        let removed = self.blacklisted_users.remove(&user_id);
        if removed {
            info!("將用戶 {} 從黑名單中移除", user_id);
        } else {
            debug!("用戶 {} 不在黑名單中", user_id);
        }
        removed
    }

    /// 檢查用戶是否在黑名單中
    ///
    /// # Arguments
    ///
    /// * `user_id` - 要檢查的用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 bool，true 表示用戶在黑名單中
    pub fn is_user_blacklisted(&self, user_id: i64) -> bool {
        self.blacklisted_users.contains(&user_id)
    }

    /// 驗證命令參數的完整性
    ///
    /// # Arguments
    ///
    /// * `required_params` - 必需的參數列表
    /// * `provided_params` - 提供的參數列表
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示所有必需參數都已提供
    pub fn validate_required_params(&self, required_params: &[&str], provided_params: &[String]) -> Result<bool, DiscordError> {
        debug!("驗證命令參數：必需 {:?}，提供 {:?}", required_params, provided_params);

        if provided_params.len() < required_params.len() {
            warn!("缺少必需參數：需要 {} 個，提供 {} 個", required_params.len(), provided_params.len());
            return Err(DiscordError::InvalidCommand(format!(
                "缺少必需參數。需要：{:?}",
                required_params
            )));
        }

        // 檢查每個必需參數是否為空
        for (i, param) in provided_params.iter().enumerate() {
            if i < required_params.len() && param.trim().is_empty() {
                warn!("參數 {} 為空", i + 1);
                return Err(DiscordError::InvalidCommand(format!(
                    "參數 {} 不能為空",
                    i + 1
                )));
            }
        }

        info!("命令參數驗證通過");
        Ok(true)
    }

    /// 驗證並創建用戶
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `username` - 用戶名稱
    ///
    /// # Returns
    ///
    /// 返回 Result<User>，成功時包含創建的用戶資訊
    pub async fn validate_and_create_user(&self, discord_user_id: i64, username: String) -> Result<User, DiscordError> {
        debug!("驗證並創建用戶：ID {}, 名稱 {}", discord_user_id, username);

        // 首先驗證用戶 ID
        self.validate_discord_user_id(discord_user_id)?;

        // 驗證用戶名稱
        self.validate_username(&username)?;

        // 檢查用戶是否已存在
        match self.user_repository.find_by_user_id(discord_user_id).await {
            Ok(Some(_)) => {
                warn!("用戶 {} 帳戶已存在", discord_user_id);
                return Err(DiscordError::AccountCreationFailed("帳戶已存在".to_string()));
            }
            Ok(None) => {
                // 用戶不存在，可以創建
                info!("用戶 {} 不存在，將創建新帳戶", discord_user_id);
            }
            Err(e) => {
                error!("檢查用戶 {} 時發生錯誤：{}", discord_user_id, e);
                return Err(DiscordError::DatabaseQueryError(format!("檢查用戶時發生錯誤：{}", e)));
            }
        }

        // 創建用戶
        self.user_repository.create_user(discord_user_id, &username).await
            .map_err(|e| DiscordError::DatabaseQueryError(format!("創建用戶失敗：{}", e)))
    }

    /// 驗證用戶身份
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<User>，成功時包含用戶資訊
    pub async fn authenticate_user(&self, discord_user_id: i64) -> Result<User, DiscordError> {
        debug!("驗證用戶身份：{}", discord_user_id);

        // 首先驗證用戶 ID
        self.validate_discord_user_id(discord_user_id)?;

        // 查詢用戶是否存在
        match self.user_repository.find_by_user_id(discord_user_id).await {
            Ok(Some(user)) => {
                info!("用戶 {} 身份驗證成功", discord_user_id);
                Ok(user)
            }
            Ok(None) => {
                warn!("用戶 {} 不存在", discord_user_id);
                Err(DiscordError::UserNotFound("用戶不存在".to_string()))
            }
            Err(e) => {
                error!("驗證用戶 {} 時發生錯誤：{}", discord_user_id, e);
                Err(DiscordError::DatabaseQueryError(format!("驗證用戶時發生錯誤：{}", e)))
            }
        }
    }
}

// 移除 Default 實作，因為現在需要 UserRepository

// 移除內部測試，因為 SecurityService 現在需要 UserRepository
// 相關測試已移至 tests/security_service_test.rs