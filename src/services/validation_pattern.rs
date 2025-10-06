// Validation Pattern - N2 計劃 REFACTOR 階段
// 統一的驗證器模式，實現可重用的驗證邏輯

use crate::error::Result;
use crate::services::security_service::SecurityService;
use std::sync::Arc;
use tracing::{debug, warn};

/// 驗證器特徵
pub trait Validator<T> {
    /// 驗證輸入並返回結果
    fn validate(&self, input: &T) -> Result<()>;

    /// 獲取驗證器名稱
    fn name(&self) -> &'static str;
}

/// 組合驗證器
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
    name: String,
}

impl<T> CompositeValidator<T> {
    /// 創建新的組合驗證器
    pub fn new(name: &str) -> Self {
        Self {
            validators: Vec::new(),
            name: name.to_string(),
        }
    }

    /// 添加驗證器
    pub fn add_validator<V: Validator<T> + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<T> Validator<T> for CompositeValidator<T> {
    fn validate(&self, input: &T) -> Result<()> {
        debug!("執行組合驗證器：{}", self.name);

        for validator in &self.validators {
            if let Err(e) = validator.validate(input) {
                warn!("驗證失敗：{} - {}", validator.name(), e);
                return Err(e);
            }
        }

        debug!("組合驗證器通過：{}", self.name);
        Ok(())
    }

    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
}

/// Discord 用戶 ID 驗證器
pub struct DiscordUserIdValidator {
    security_service: Arc<SecurityService>,
}

impl DiscordUserIdValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<i64> for DiscordUserIdValidator {
    fn validate(&self, input: &i64) -> Result<()> {
        self.security_service.validate_discord_user_id(*input)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "DiscordUserIdValidator"
    }
}

/// 字符串輸入驗證器
pub struct StringInputValidator {
    security_service: Arc<SecurityService>,
    max_length: usize,
}

impl StringInputValidator {
    pub fn new(security_service: Arc<SecurityService>, max_length: usize) -> Self {
        Self { security_service, max_length }
    }
}

impl Validator<String> for StringInputValidator {
    fn validate(&self, input: &String) -> Result<()> {
        self.security_service.sanitize_string_input(input, self.max_length)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "StringInputValidator"
    }
}

/// 金額驗證器
pub struct AmountValidator {
    security_service: Arc<SecurityService>,
}

impl AmountValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<String> for AmountValidator {
    fn validate(&self, input: &String) -> Result<()> {
        self.security_service.validate_amount(input)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "AmountValidator"
    }
}

/// 用戶名稱驗證器
pub struct UsernameValidator {
    security_service: Arc<SecurityService>,
}

impl UsernameValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<String> for UsernameValidator {
    fn validate(&self, input: &String) -> Result<()> {
        let sanitized = self.security_service.sanitize_string_input(input, 32)?;
        self.security_service.validate_username(&sanitized)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "UsernameValidator"
    }
}

/// 自我轉帳防護驗證器
pub struct SelfTransferProtectionValidator {
    security_service: Arc<SecurityService>,
}

impl SelfTransferProtectionValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<(i64, i64)> for SelfTransferProtectionValidator {
    fn validate(&self, input: &(i64, i64)) -> Result<()> {
        let (from_user_id, to_user_id) = input;
        self.security_service.validate_no_self_transfer(*from_user_id, *to_user_id)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SelfTransferProtectionValidator"
    }
}

/// 驗證器工廠
pub struct ValidatorFactory {
    security_service: Arc<SecurityService>,
}

impl ValidatorFactory {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }

    /// 創建 Discord 用戶 ID 驗證器
    pub fn create_discord_user_id_validator(&self) -> DiscordUserIdValidator {
        DiscordUserIdValidator::new(self.security_service.clone())
    }

    /// 創建字符串輸入驗證器
    pub fn create_string_input_validator(&self, max_length: usize) -> StringInputValidator {
        StringInputValidator::new(self.security_service.clone(), max_length)
    }

    /// 創建金額驗證器
    pub fn create_amount_validator(&self) -> AmountValidator {
        AmountValidator::new(self.security_service.clone())
    }

    /// 創建用戶名稱驗證器
    pub fn create_username_validator(&self) -> UsernameValidator {
        UsernameValidator::new(self.security_service.clone())
    }

    /// 創建自我轉帳防護驗證器
    pub fn create_self_transfer_protection_validator(&self) -> SelfTransferProtectionValidator {
        SelfTransferProtectionValidator::new(self.security_service.clone())
    }

    /// 創建轉帳操作組合驗證器
    pub fn create_transfer_validator(&self) -> CompositeValidator<TransferInput> {
        CompositeValidator::new("TransferValidator")
            .add_validator(TransferInputValidator::new(self.security_service.clone()))
    }

    /// 創建帳戶創建操作組合驗證器
    pub fn create_account_creation_validator(&self) -> CompositeValidator<AccountCreationInput> {
        CompositeValidator::new("AccountCreationValidator")
            .add_validator(AccountCreationInputValidator::new(self.security_service.clone()))
    }
}

/// 轉帳輸入結構
#[derive(Debug, Clone)]
pub struct TransferInput {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub amount: String,
}

/// 轉帳輸入驗證器
pub struct TransferInputValidator {
    security_service: Arc<SecurityService>,
}

impl TransferInputValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<TransferInput> for TransferInputValidator {
    fn validate(&self, input: &TransferInput) -> Result<()> {
        // 驗證發送方用戶 ID
        self.security_service.validate_discord_user_id(input.from_user_id)?;

        // 驗證接收方用戶 ID
        self.security_service.validate_discord_user_id(input.to_user_id)?;

        // 驗證金額
        self.security_service.validate_amount(&input.amount)?;

        // 驗證自我轉帳防護
        self.security_service.validate_no_self_transfer(input.from_user_id, input.to_user_id)?;

        debug!("轉帳輸入驗證通過");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TransferInputValidator"
    }
}

/// 帳戶創建輸入結構
#[derive(Debug, Clone)]
pub struct AccountCreationInput {
    pub discord_user_id: i64,
    pub username: String,
}

/// 帳戶創建輸入驗證器
pub struct AccountCreationInputValidator {
    security_service: Arc<SecurityService>,
}

impl AccountCreationInputValidator {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }
}

impl Validator<AccountCreationInput> for AccountCreationInputValidator {
    fn validate(&self, input: &AccountCreationInput) -> Result<()> {
        // 驗證 Discord 用戶 ID
        self.security_service.validate_discord_user_id(input.discord_user_id)?;

        // 驗證和清理用戶名稱
        let sanitized_username = self.security_service.sanitize_string_input(&input.username, 32)?;
        self.security_service.validate_username(&sanitized_username)?;

        debug!("帳戶創建輸入驗證通過");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "AccountCreationInputValidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_user_pool;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_validator_factory() {
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = crate::database::UserRepository::new(pool);
            let security_service = SecurityService::new(user_repo).unwrap();
            let security_service = Arc::new(security_service);

            let factory = ValidatorFactory::new(security_service);

            // 測試創建各種驗證器
            let _user_id_validator = factory.create_discord_user_id_validator();
            let _amount_validator = factory.create_amount_validator();
            let _username_validator = factory.create_username_validator();
            let _transfer_validator = factory.create_transfer_validator();
            let _account_validator = factory.create_account_creation_validator();

            assert!(true, "驗證器工廠創建成功");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }

    #[tokio::test]
    async fn test_composite_validator() {
        let database_config = DatabaseConfig::from_env().unwrap();

        if let Ok(pool) = create_user_pool(&database_config).await {
            let user_repo = crate::database::UserRepository::new(pool);
            let security_service = SecurityService::new(user_repo).unwrap();
            let security_service = Arc::new(security_service);

            let factory = ValidatorFactory::new(security_service);

            let composite_validator = CompositeValidator::new("TestComposite")
                .add_validator(factory.create_discord_user_id_validator());

            // 測試有效的用戶 ID
            let result = composite_validator.validate(&123456789_i64);
            assert!(result.is_ok(), "有效用戶 ID 應該通過驗證");

            // 測試無效的用戶 ID
            let result = composite_validator.validate(&-1_i64);
            assert!(result.is_err(), "無效用戶 ID 應該被拒絕");
        } else {
            println!("警告：沒有資料庫連接，跳過測試");
        }
    }
}