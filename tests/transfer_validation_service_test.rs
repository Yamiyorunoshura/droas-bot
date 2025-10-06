// Transfer Validation Service 測試
// RED 階段：測試先行的開發，確保所有測試案例失敗

use droas_bot::services::transfer_validation_service::{TransferValidationService, ValidationError};
use droas_bot::database::user_repository::User;
use bigdecimal::BigDecimal;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    /// 創建測試用戶
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
    fn test_insufficient_balance_validation() {
        // 測試餘額不足驗證
        // 預期：當用戶嘗試轉帳超過當前餘額時，應返回餘額不足錯誤

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(456, "50.00");
        let transfer_amount = BigDecimal::from_str("150.00").unwrap();

        let result = validation_service.validate_transfer(&from_user, &to_user, &transfer_amount);

        // 預期結果：驗證失敗，返回餘額不足錯誤
        assert!(result.is_err(), "餘額不足的轉帳應該驗證失敗");

        match result.unwrap_err() {
            ValidationError::InsufficientBalance { message } => {
                assert!(message.contains("餘額不足"));
            }
            _ => panic!("應返回餘額不足錯誤"),
        }
    }

    #[test]
    fn test_self_transfer_prevention() {
        // 測試自我轉帳阻止
        // 預期：當用戶嘗試轉帳給自己時，應返回自我轉帳錯誤

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(123, "100.00"); // 相同用戶 ID
        let transfer_amount = BigDecimal::from_str("50.00").unwrap();

        let result = validation_service.validate_transfer(&from_user, &to_user, &transfer_amount);

        // 預期結果：驗證失敗，返回自我轉帳錯誤
        assert!(result.is_err(), "自我轉帳應該驗證失敗");

        match result.unwrap_err() {
            ValidationError::SelfTransfer { message } => {
                assert!(message.contains("不能轉帳給自己"));
            }
            _ => panic!("應返回自我轉帳錯誤"),
        }
    }

    #[test]
    fn test_invalid_amount_validation() {
        // 測試無效金額驗證
        // 預期：當輸入負數、零或過小金額時，應返回無效金額錯誤

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(456, "50.00");

        // 測試負數金額
        let negative_amount = BigDecimal::from_str("-10.00").unwrap();
        let result = validation_service.validate_transfer(&from_user, &to_user, &negative_amount);
        assert!(result.is_err(), "負數金額應該驗證失敗");
        match result.unwrap_err() {
            ValidationError::InvalidAmount { message } => {
                assert!(message.contains("金額必須為正數"));
            }
            _ => panic!("應返回無效金額錯誤"),
        }

        // 測試零金額
        let zero_amount = BigDecimal::from_str("0.00").unwrap();
        let result = validation_service.validate_transfer(&from_user, &to_user, &zero_amount);
        assert!(result.is_err(), "零金額應該驗證失敗");
        match result.unwrap_err() {
            ValidationError::InvalidAmount { message } => {
                assert!(message.contains("金額必須為正數"));
            }
            _ => panic!("應返回無效金額錯誤"),
        }

        // 測試過小金額（小於系統最小值）
        let tiny_amount = BigDecimal::from_str("0.001").unwrap();
        let result = validation_service.validate_transfer(&from_user, &to_user, &tiny_amount);
        assert!(result.is_err(), "過小金額應該驗證失敗");
        match result.unwrap_err() {
            ValidationError::InvalidAmount { message } => {
                assert!(message.contains("金額過小"));
            }
            _ => panic!("應返回無效金額錯誤"),
        }
    }

    #[test]
    fn test_boundary_condition_transfers() {
        // 測試邊界條件
        // 預期：當轉帳金額等於當前餘額時，應驗證通過

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.00");
        let to_user = create_test_user(456, "0.00");
        let transfer_amount = BigDecimal::from_str("100.00").unwrap(); // 等於餘額

        let result = validation_service.validate_transfer(&from_user, &to_user, &transfer_amount);

        // 預期結果：驗證通過
        assert!(result.is_ok(), "轉帳金額等於餘額應該驗證通過");

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid, "驗證結果應為有效");
        assert!(validation_result.message.is_empty(), "成功的驗證不應有錯誤消息");
    }

    #[test]
    fn test_large_transfer_limitation() {
        // 測試大額轉帳限制
        // 預期：當轉帳金額超過系統設定的單筆限制時，應返回大額轉帳錯誤

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100000.00");
        let to_user = create_test_user(456, "0.00");
        let large_amount = BigDecimal::from_str("50000.00").unwrap(); // 假設系統限制為 10000

        let result = validation_service.validate_transfer(&from_user, &to_user, &large_amount);

        // 預期結果：驗證失敗，返回大額轉帳錯誤
        assert!(result.is_err(), "超過限制的大額轉帳應該驗證失敗");

        match result.unwrap_err() {
            ValidationError::AmountExceedsLimit { message, limit } => {
                assert!(message.contains("超過單筆轉帳限制"));
                assert!(limit > BigDecimal::from_str("0").unwrap());
            }
            _ => panic!("應返回大額轉帳錯誤"),
        }
    }

    #[test]
    fn test_valid_transfer_success() {
        // 測試有效轉帳成功案例
        // 預期：所有條件都符合的轉帳應驗證通過

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "1000.00");
        let to_user = create_test_user(456, "500.00");
        let transfer_amount = BigDecimal::from_str("200.00").unwrap();

        let result = validation_service.validate_transfer(&from_user, &to_user, &transfer_amount);

        // 預期結果：驗證通過
        assert!(result.is_ok(), "有效的轉帳應該驗證通過");

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid, "驗證結果應為有效");
        assert!(validation_result.message.is_empty(), "成功的驗證不應有錯誤消息");
    }

    #[test]
    fn test_decimal_precision_handling() {
        // 測試小數精度處理
        // 預期：系統應正確處理多位小數的金額

        let validation_service = TransferValidationService::new();
        let from_user = create_test_user(123, "100.123456");
        let to_user = create_test_user(456, "0.00");
        let precise_amount = BigDecimal::from_str("50.123456").unwrap();

        let result = validation_service.validate_transfer(&from_user, &to_user, &precise_amount);

        // 預期結果：驗證通過，系統能處理高精度小數
        assert!(result.is_ok(), "高精度小數金額應該驗證通過");

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid, "驗證結果應為有效");
    }
}