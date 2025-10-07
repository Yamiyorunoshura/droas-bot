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

    /// 驗證管理員權限（實現雙重驗證機制基礎）
    ///
    /// # Arguments
    ///
    /// * `discord_user_id` - Discord 用戶 ID
    /// * `admin_users` - 授權的管理員用戶 ID 列表
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶具有管理員權限
    pub async fn verify_admin_permission(&self, discord_user_id: i64, admin_users: &[i64]) -> Result<bool, DiscordError> {
        debug!("驗證管理員權限：{}", discord_user_id);

        // 第一重驗證：檢查用戶 ID 是否在授權列表中
        if !admin_users.contains(&discord_user_id) {
            warn!("用戶 {} 不在管理員授權列表中", discord_user_id);
            return Err(DiscordError::PermissionDenied("用戶沒有管理員權限".to_string()));
        }

        // 第二重驗證：驗證用戶身份確實存在且有效
        let user = self.authenticate_user(discord_user_id).await
            .map_err(|e| {
                error!("管理員身份驗證失敗：{}", e);
                DiscordError::PermissionDenied(format!("管理員身份驗證失敗：{}", e))
            })?;

        // 第三重驗證：檢查用戶是否在黑名單中（額外安全檢查）
        if self.is_user_blacklisted(discord_user_id) {
            warn!("管理員 {} 在黑名單中，拒絕管理員操作", discord_user_id);
            return Err(DiscordError::PermissionDenied("管理員帳戶已被限制".to_string()));
        }

        info!("管理員權限驗證通過：{} ({})", discord_user_id, user.username);
        Ok(true)
    }

    /// 驗證大額操作的二次確認需求
    ///
    /// # Arguments
    ///
    /// * `amount` - 操作金額
    /// * `operation_type` - 操作類型
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示需要二次確認
    pub fn requires_double_confirmation(&self, amount: f64, operation_type: &str) -> Result<bool, DiscordError> {
        debug!("檢查是否需要二次確認：金額 {}, 類型 {}", amount, operation_type);

        // 根據操作類型和金額設置不同的閾值
        let threshold = match operation_type {
            "ADJUST_BALANCE" | "SET_BALANCE" => 10000.0,  // 金額調整超過 10000 需要確認
            "FREEZE_ACCOUNT" | "UNFREEZE_ACCOUNT" => 0.0, // 帳戶操作總是需要確認
            "TRANSFER_OWNERSHIP" => 0.0,                   // 所有權轉移總是需要確認
            _ => 50000.0,                                   // 其他操作超過 50000 需要確認
        };

        let requires_confirmation = amount >= threshold;

        if requires_confirmation {
            info!("操作需要二次確認：金額 {} >= 閾值 {}, 類型 {}", amount, threshold, operation_type);
        } else {
            debug!("操作不需要二次確認：金額 {} < 閾值 {}, 類型 {}", amount, threshold, operation_type);
        }

        Ok(requires_confirmation)
    }

    /// 驗證操作是否屬於敏感操作
    ///
    /// # Arguments
    ///
    /// * `operation_type` - 操作類型
    /// * `target_user_id` - 目標用戶 ID（可選）
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示是敏感操作
    pub fn is_sensitive_operation(&self, operation_type: &str, target_user_id: Option<i64>) -> Result<bool, DiscordError> {
        debug!("檢查是否為敏感操作：類型 {}, 目標用戶 {:?}", operation_type, target_user_id);

        let sensitive_operations = vec![
            "FREEZE_ACCOUNT",
            "UNFREEZE_ACCOUNT",
            "TRANSFER_OWNERSHIP",
            "DELETE_ACCOUNT",
            "RESET_PASSWORD",
            "MODIFY_PERMISSIONS",
        ];

        let is_sensitive = sensitive_operations.contains(&operation_type) ||
                           operation_type.starts_with("SYSTEM_") ||
                           operation_type.starts_with("ADMIN_");

        // 額外檢查：是否對其他管理員進行操作
        let is_admin_operation = if let Some(target) = target_user_id {
            // 這裡可以與管理員列表比較，但為了簡化，我們假設對非當前用戶的操作都是敏感的
            target != 0
        } else {
            false
        };

        let result = is_sensitive || is_admin_operation;

        if result {
            info!("檢測到敏感操作：類型 {}, 目標用戶 {:?}", operation_type, target_user_id);
        }

        Ok(result)
    }

    /// 檢查異常操作模式
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 管理員用戶 ID
    /// * `operation_count` - 短時間內的操作次數
    /// * `time_window_minutes` - 時間窗口（分鐘）
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示檢測到異常模式
    pub fn check_anomalous_pattern(&self, admin_user_id: i64, operation_count: u32, time_window_minutes: u32) -> Result<bool, DiscordError> {
        debug!("檢查異常操作模式：管理員 {}, 操作次數 {}, 時間窗口 {} 分鐘",
               admin_user_id, operation_count, time_window_minutes);

        // 計算操作頻率（每分鐘操作次數）
        let frequency = operation_count as f64 / time_window_minutes as f64;

        // 異常閾值：每分鐘超過 5 次操作，或 10 分鐘內超過 20 次操作
        let is_anomalous = frequency > 5.0 ||
                          (time_window_minutes <= 10 && operation_count > 20);

        if is_anomalous {
            warn!("檢測到異常操作模式：管理員 {} 在 {} 分鐘內執行了 {} 次操作（頻率：{:.2}/分鐘）",
                  admin_user_id, time_window_minutes, operation_count, frequency);
        } else {
            debug!("操作模式正常：管理員 {} 在 {} 分鐘內執行了 {} 次操作（頻率：{:.2}/分鐘）",
                   admin_user_id, time_window_minutes, operation_count, frequency);
        }

        Ok(is_anomalous)
    }

    /// 驗證管理員操作的安全限制
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 管理員用戶 ID
    /// * `operation_type` - 操作類型
    /// * `amount` - 操作金額
    /// * `target_user_id` - 目標用戶 ID（可選）
    ///
    /// # Returns
    ///
    /// 返回 Result<AdminSecurityCheck>，包含安全檢查結果
    pub async fn validate_admin_operation_security(
        &self,
        admin_user_id: i64,
        operation_type: &str,
        amount: f64,
        target_user_id: Option<i64>
    ) -> Result<AdminSecurityCheck, DiscordError> {
        debug!("驗證管理員操作安全：{} 執行 {}，金額 {}", admin_user_id, operation_type, amount);

        let mut security_check = AdminSecurityCheck::default();

        // 檢查是否為敏感操作
        security_check.is_sensitive = self.is_sensitive_operation(operation_type, target_user_id)?;

        // 檢查是否需要二次確認
        security_check.requires_confirmation = self.requires_double_confirmation(amount, operation_type)?;

        // 檢查金額是否在允許範圍內
        security_check.amount_valid = amount >= 0.0 && amount <= 1_000_000.0;

        // 檢查操作類型是否有效
        security_check.operation_valid = !operation_type.is_empty() && operation_type.len() <= 50;

        // 綜合安全評估
        security_check.is_safe = security_check.amount_valid &&
                                security_check.operation_valid &&
                                (!security_check.requires_confirmation || security_check.is_sensitive);

        if security_check.is_safe {
            info!("管理員操作安全檢查通過：{} 執行 {}", admin_user_id, operation_type);
        } else {
            warn!("管理員操作安全檢查未完全通過：{} 執行 {} (敏感: {}, 需確認: {}, 金額有效: {}, 操作有效: {})",
                  admin_user_id, operation_type,
                  security_check.is_sensitive, security_check.requires_confirmation,
                  security_check.amount_valid, security_check.operation_valid);
        }

        Ok(security_check)
    }
}

/// 管理員安全檢查結果
#[derive(Debug, Clone)]
pub struct AdminSecurityCheck {
    /// 是否為敏感操作
    pub is_sensitive: bool,
    /// 是否需要二次確認
    pub requires_confirmation: bool,
    /// 金額是否有效
    pub amount_valid: bool,
    /// 操作類型是否有效
    pub operation_valid: bool,
    /// 綜合安全評估
    pub is_safe: bool,
}

impl Default for AdminSecurityCheck {
    fn default() -> Self {
        Self {
            is_sensitive: false,
            requires_confirmation: false,
            amount_valid: true,
            operation_valid: true,
            is_safe: true,
        }
    }
}

// 移除 Default 實作，因為現在需要 UserRepository

// 移除內部測試，因為 SecurityService 現在需要 UserRepository
// 相關測試已移至 tests/security_service_test.rs