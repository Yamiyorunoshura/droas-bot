// Transfer Validation Service
// REFACTOR 階段：重構與優化

use crate::database::user_repository::User;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use tracing::{debug, trace, warn};

/// 驗證規則 Trait
/// 提供可插拔的驗證規則系統，提高可維護性和擴展性
pub trait ValidationRule: Send + Sync {
    /// 驗證規則名稱
    fn name(&self) -> &'static str;

    /// 執行驗證
    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError>;

    /// 獲取驗證規則優先級（數字越小優先級越高）
    fn priority(&self) -> u32 { 100 }
}

/// 驗證上下文
/// 包含驗證所需的所有數據
#[derive(Debug)]
pub struct ValidationContext {
    /// 發送方用戶
    pub from_user: User,
    /// 接收方用戶
    pub to_user: User,
    /// 轉帳金額
    pub amount: BigDecimal,
    /// 系統配置
    pub config: ValidationConfig,
}

/// 驗證配置
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// 最大單筆轉帳限制
    pub max_single_transfer: BigDecimal,
    /// 最小轉帳金額
    pub min_transfer_amount: BigDecimal,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_single_transfer: BigDecimal::from_str("10000.00").unwrap(),
            min_transfer_amount: BigDecimal::from_str("0.01").unwrap(),
        }
    }
}

/// 驗證結果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 驗證是否通過
    pub is_valid: bool,
    /// 錯誤消息（如果驗證失敗）
    pub message: String,
    /// 失敗的驗證規則名稱
    pub failed_rule: Option<String>,
    /// 驗證時間戳
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl ValidationResult {
    /// 創建成功的驗證結果
    pub fn success() -> Self {
        Self {
            is_valid: true,
            message: String::new(),
            failed_rule: None,
            validated_at: chrono::Utc::now(),
        }
    }

    /// 創建失敗的驗證結果
    pub fn failure(rule_name: &str, message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            message: message.into(),
            failed_rule: Some(rule_name.to_string()),
            validated_at: chrono::Utc::now(),
        }
    }
}

/// 驗證錯誤類型
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// 餘額不足
    InsufficientBalance { message: String },
    /// 自我轉帳
    SelfTransfer { message: String },
    /// 無效金額
    InvalidAmount { message: String },
    /// 金額超過限制
    AmountExceedsLimit { message: String, limit: BigDecimal },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InsufficientBalance { message } => write!(f, "{}", message),
            ValidationError::SelfTransfer { message } => write!(f, "{}", message),
            ValidationError::InvalidAmount { message } => write!(f, "{}", message),
            ValidationError::AmountExceedsLimit { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ValidationError {}

// 具體驗證規則實現

/// 自我轉帳阻止規則
#[derive(Debug)]
pub struct SelfTransferRule;

impl ValidationRule for SelfTransferRule {
    fn name(&self) -> &'static str {
        "self_transfer_protection"
    }

    fn priority(&self) -> u32 {
        10 // 高優先級，首先檢查
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        trace!("檢查自我轉帳：{} -> {}", context.from_user.discord_user_id, context.to_user.discord_user_id);

        if context.from_user.discord_user_id == context.to_user.discord_user_id {
            debug!("檢測到自我轉帳嘗試：用戶 {}", context.from_user.discord_user_id);
            return Err(ValidationError::SelfTransfer {
                message: "不能轉帳給自己".to_string(),
            });
        }

        Ok(())
    }
}

/// 金額有效性驗證規則
#[derive(Debug)]
pub struct AmountValidityRule;

impl ValidationRule for AmountValidityRule {
    fn name(&self) -> &'static str {
        "amount_validity"
    }

    fn priority(&self) -> u32 {
        20 // 第二優先級
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        trace!("檢查金額有效性：{}", context.amount);

        // 檢查金額是否為正數
        if context.amount <= BigDecimal::from_str("0").unwrap() {
            debug!("檢測到無效金額：{}", context.amount);
            return Err(ValidationError::InvalidAmount {
                message: "金額必須為正數".to_string(),
            });
        }

        // 檢查最小金額限制
        if context.amount < context.config.min_transfer_amount {
            debug!("金額過小：{} < {}", context.amount, context.config.min_transfer_amount);
            return Err(ValidationError::InvalidAmount {
                message: format!("金額過小，最小轉帳金額為 {} 幣", context.config.min_transfer_amount),
            });
        }

        Ok(())
    }
}

/// 餘額充足性驗證規則
#[derive(Debug)]
pub struct BalanceSufficiencyRule;

impl ValidationRule for BalanceSufficiencyRule {
    fn name(&self) -> &'static str {
        "balance_sufficiency"
    }

    fn priority(&self) -> u32 {
        30 // 第三優先級
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        trace!("檢查餘額充足性：用戶 {}, 餘額 {}, 需要 {}",
               context.from_user.discord_user_id, context.from_user.balance, context.amount);

        if context.from_user.balance < context.amount {
            debug!("餘額不足：用戶 {}, 餘額 {} < {}",
                   context.from_user.discord_user_id, context.from_user.balance, context.amount);
            return Err(ValidationError::InsufficientBalance {
                message: format!("餘額不足。當前餘額：{} 幣", context.from_user.balance),
            });
        }

        Ok(())
    }
}

/// 大額轉帳限制規則
#[derive(Debug)]
pub struct LargeTransferLimitRule;

impl ValidationRule for LargeTransferLimitRule {
    fn name(&self) -> &'static str {
        "large_transfer_limit"
    }

    fn priority(&self) -> u32 {
        40 // 較低優先級，在最後檢查
    }

    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        trace!("檢查大額轉帳限制：{}", context.amount);

        if context.amount > context.config.max_single_transfer {
            debug!("超過單筆轉帳限制：{} > {}", context.amount, context.config.max_single_transfer);
            return Err(ValidationError::AmountExceedsLimit {
                message: format!("超過單筆轉帳限制 {} 幣", context.config.max_single_transfer),
                limit: context.config.max_single_transfer.clone(),
            });
        }

        Ok(())
    }
}

/// Transfer Validation Service
///
/// 負責驗證轉帳請求的各種條件
/// 實現 Security/Validation Service 架構元件的轉帳驗證功能
/// REFACTOR 階段：使用可插拔的驗證規則系統
pub struct TransferValidationService {
    /// 驗證規則列表（按優先級排序）
    validation_rules: Vec<Box<dyn ValidationRule>>,
    /// 驗證配置
    config: ValidationConfig,
}

impl TransferValidationService {
    /// 創建新的 Transfer Validation Service 實例
    pub fn new() -> Self {
        let config = ValidationConfig::default();
        Self::with_config(config)
    }

    /// 使用自定義配置創建 Transfer Validation Service
    pub fn with_config(config: ValidationConfig) -> Self {
        let mut rules: Vec<Box<dyn ValidationRule>> = Vec::new();

        // 註冊驗證規則（按優先級自動排序）
        rules.push(Box::new(SelfTransferRule));
        rules.push(Box::new(AmountValidityRule));
        rules.push(Box::new(BalanceSufficiencyRule));
        rules.push(Box::new(LargeTransferLimitRule));

        // 按優先級排序
        rules.sort_by_key(|rule| rule.priority());

        Self {
            validation_rules: rules,
            config,
        }
    }

    /// 使用自定義限制創建 Transfer Validation Service（向後兼容）
    ///
    /// # Arguments
    ///
    /// * `max_single_transfer` - 單筆最大轉帳金額
    /// * `min_transfer_amount` - 最小轉帳金額
    pub fn with_limits(max_single_transfer: BigDecimal, min_transfer_amount: BigDecimal) -> Self {
        let config = ValidationConfig {
            max_single_transfer,
            min_transfer_amount,
        };
        Self::with_config(config)
    }

    /// 添加自定義驗證規則
    ///
    /// # Arguments
    ///
    /// * `rule` - 自定義驗證規則
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.validation_rules.push(rule);
        // 重新排序
        self.validation_rules.sort_by_key(|rule| rule.priority());
        self
    }

    /// 驗證轉帳請求
    ///
    /// # Arguments
    ///
    /// * `from_user` - 發送方用戶
    /// * `to_user` - 接收方用戶
    /// * `amount` - 轉帳金額
    ///
    /// # Returns
    ///
    /// 返回 `Result<ValidationResult, ValidationError>`，包含驗證結果或錯誤
    pub fn validate_transfer(
        &self,
        from_user: &User,
        to_user: &User,
        amount: &BigDecimal,
    ) -> Result<ValidationResult, ValidationError> {
        debug!("開始轉帳驗證：用戶 {} -> 用戶 {}, 金額：{}",
               from_user.discord_user_id, to_user.discord_user_id, amount);

        // 創建驗證上下文
        let context = ValidationContext {
            from_user: from_user.clone(),
            to_user: to_user.clone(),
            amount: amount.clone(),
            config: self.config.clone(),
        };

        // 依序執行所有驗證規則
        for rule in &self.validation_rules {
            trace!("執行驗證規則：{}", rule.name());

            match rule.validate(&context) {
                Ok(()) => {
                    debug!("驗證規則通過：{}", rule.name());
                }
                Err(error) => {
                    warn!("驗證規則失敗：{} - {}", rule.name(), error);
                    return Err(error);
                }
            }
        }

        debug!("所有驗證規則通過");
        Ok(ValidationResult::success())
    }

    /// 獲取驗證配置
    pub fn get_config(&self) -> &ValidationConfig {
        &self.config
    }

    /// 獲取已註冊的驗證規則數量
    pub fn get_rule_count(&self) -> usize {
        self.validation_rules.len()
    }

    /// 獲取所有驗證規則名稱
    pub fn get_rule_names(&self) -> Vec<&'static str> {
        self.validation_rules.iter().map(|rule| rule.name()).collect()
    }
}

impl Default for TransferValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_test_user(id: i64, balance: &str) -> User {
        User {
            discord_user_id: id,
            username: format!("user_{}", id),
            balance: BigDecimal::from_str(balance).unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_validate_transfer_success() {
        let service = TransferValidationService::new();
        let from_user = create_test_user(123, "1000.00");
        let to_user = create_test_user(456, "500.00");
        let amount = BigDecimal::from_str("200.00").unwrap();

        let result = service.validate_transfer(&from_user, &to_user, &amount);
        assert!(result.is_ok());

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert!(validation_result.message.is_empty());
    }

    #[test]
    fn test_self_transfer_validation() {
        let service = TransferValidationService::new();
        let user = create_test_user(123, "1000.00");
        let amount = BigDecimal::from_str("200.00").unwrap();

        let result = service.validate_transfer(&user, &user, &amount);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::SelfTransfer { message } => {
                assert!(message.contains("不能轉帳給自己"));
            }
            _ => panic!("應返回 SelfTransfer 錯誤"),
        }
    }

    #[test]
    fn test_insufficient_balance_validation() {
        let service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(456, "500.00");
        let amount = BigDecimal::from_str("200.00").unwrap();

        let result = service.validate_transfer(&from_user, &to_user, &amount);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::InsufficientBalance { message } => {
                assert!(message.contains("餘額不足"));
            }
            _ => panic!("應返回 InsufficientBalance 錯誤"),
        }
    }

    #[test]
    fn test_invalid_amount_validation() {
        let service = TransferValidationService::new();
        let from_user = create_test_user(123, "1000.00");
        let to_user = create_test_user(456, "500.00");

        // 測試負數
        let negative_amount = BigDecimal::from_str("-10.00").unwrap();
        let result = service.validate_transfer(&from_user, &to_user, &negative_amount);
        assert!(result.is_err());

        // 測試零
        let zero_amount = BigDecimal::from_str("0.00").unwrap();
        let result = service.validate_transfer(&from_user, &to_user, &zero_amount);
        assert!(result.is_err());

        // 測試過小金額
        let tiny_amount = BigDecimal::from_str("0.001").unwrap();
        let result = service.validate_transfer(&from_user, &to_user, &tiny_amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_transfer_limitation() {
        let service = TransferValidationService::new();
        let from_user = create_test_user(123, "50000.00");
        let to_user = create_test_user(456, "0.00");
        let large_amount = BigDecimal::from_str("15000.00").unwrap(); // 超過預設限制 10000

        let result = service.validate_transfer(&from_user, &to_user, &large_amount);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::AmountExceedsLimit { message, limit } => {
                assert!(message.contains("超過單筆轉帳限制"));
                assert_eq!(limit, BigDecimal::from_str("10000.00").unwrap());
            }
            _ => panic!("應返回 AmountExceedsLimit 錯誤"),
        }
    }

    #[test]
    fn test_boundary_condition_transfers() {
        let service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(456, "0.00");
        let amount = BigDecimal::from_str("100.00").unwrap(); // 等於餘額

        let result = service.validate_transfer(&from_user, &to_user, &amount);
        assert!(result.is_ok());

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
    }
}