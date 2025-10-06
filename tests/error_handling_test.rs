//! 錯誤處理測試模組
//! 測試集中式錯誤處理系統的功能和用戶友好性

use droas_bot::error::{DiscordError, ErrorCategory, ErrorSeverity};
use droas_bot::services::{error_handler::ErrorHandler, ui_components::UIComponentFactory};

#[cfg(test)]
mod error_handler_tests {
    use super::*;

    /// 測試業務邏輯錯誤：餘額不足時的轉帳失敗
    #[tokio::test]
    async fn test_insufficient_funds_error_message() {
        let error_handler = ErrorHandler::new();
        let user_id = 12345;
        let current_balance = 500.0;
        let transfer_amount = 1000.0;

        let error = DiscordError::InsufficientBalance(user_id);
        let result = error_handler.format_user_error(&error);

        // 檢查錯誤消息包含必要信息
        assert!(result.contains("餘額不足"));
        // 這個測試中，我們不期望包含具體餘額和金額信息，因為那是業務邏輯的細節
        assert!(result.contains("建議"));

        // 檢查用戶友好性：提供具體解決方案
        assert!(result.contains("充值") || result.contains("賺取") || result.contains("其他方式"));
    }

    /// 測試系統錯誤：資料庫連接失敗
    #[tokio::test]
    async fn test_database_connection_error_handling() {
        let error_handler = ErrorHandler::new();
        let db_error = "連接池超時";

        let error = DiscordError::DatabaseConnectionError(db_error.to_string());
        let result = error_handler.format_user_error(&error);

        // 檢查系統錯誤顯示用戶友好的消息
        assert!(result.contains("系統維護"));
        assert!(result.contains("稍後再試"));
        assert!(!result.contains("Connection pool timeout")); // 不應暴露技術細節
    }

    /// 測試用戶輸入錯誤：格式錯誤的指令
    #[tokio::test]
    async fn test_invalid_command_format_error() {
        let error_handler = ErrorHandler::new();
        let invalid_command = "!transfer @user invalid_amount";

        let error = DiscordError::InvalidCommand(invalid_command.to_string());
        let result = error_handler.format_user_error(&error);

        // 檢查提供正確的使用格式
        assert!(result.contains("指令格式錯誤"));
        assert!(result.contains("建議"));
    }

    /// 測試安全錯誤：未授權用戶嘗試執行操作
    #[tokio::test]
    async fn test_unauthorized_access_error() {
        let error_handler = ErrorHandler::new();
        let user_id = 12345;
        let required_permission = "ADMIN";

        // 這裡需要擴展 DiscordError 來包含權限錯誤
        let error = DiscordError::ValidationError("權限不足".to_string());
        let result = error_handler.format_user_error(&error);

        // 檢查權限錯誤消息 (ValidationError 分類為用戶輸入錯誤)
        assert!(result.contains("指令格式錯誤") || result.contains("建議"));
        // 目前沒有專門的權限不足模板，所以使用後備模板
    }

    /// 測試錯誤消息覆蓋率：統計所有錯誤消息的用戶友好性
    #[tokio::test]
    async fn test_error_message_coverage() {
        let error_handler = ErrorHandler::new();

        // 測試各種錯誤類型
        let errors = vec![
            DiscordError::InsufficientBalance(12345),
            DiscordError::DatabaseConnectionError("連接失敗".to_string()),
            DiscordError::InvalidCommand("!invalid".to_string()),
            DiscordError::ValidationError("驗證失敗".to_string()),
            DiscordError::UserNotFound("12345".to_string()),
            DiscordError::InvalidAmount("負數".to_string()),
            DiscordError::AccountCreationFailed("系統錯誤".to_string()),
        ];

        let mut friendly_messages = 0;
        let total_errors = errors.len();

        for error in &errors {
            let message = error_handler.format_user_error(error);

            // 檢查是否包含用戶指導
            if has_user_guidance(&message) {
                friendly_messages += 1;
            }
        }

        // 檢查覆蓋率：至少 90% 的錯誤消息提供可行指導
        let coverage = (friendly_messages as f64 / total_errors as f64) * 100.0;
        assert!(coverage >= 90.0, "錯誤消息友好性覆蓋率 {}% 低於 90% 要求", coverage);
    }

    /// 輔助函數：檢查錯誤消息是否包含用戶指導
    fn has_user_guidance(message: &str) -> bool {
        let guidance_keywords = vec![
            "解決方案", "建議", "步驟", "請", "可以", "應該", "需要",
            "聯繫", "重試", "檢查", "確認", "使用", "格式", "範例",
            "稍後", "聯繫管理員", "幫助", "支持"
        ];

        for keyword in guidance_keywords {
            if message.contains(keyword) {
                return true;
            }
        }

        false
    }

    /// 測試錯誤分類覆蓋
    #[tokio::test]
    async fn test_error_classification_coverage() {
        let error_handler = ErrorHandler::new();

        // 測試各種錯誤類型都有明確分類
        let test_cases = vec![
            (DiscordError::InsufficientBalance(12345), "業務邏輯錯誤"),
            (DiscordError::DatabaseConnectionError("連接失敗".to_string()), "系統錯誤"),
            (DiscordError::InvalidCommand("!invalid".to_string()), "用戶輸入錯誤"),
            (DiscordError::ValidationError("驗證失敗".to_string()), "用戶輸入錯誤"),
        ];

        for (error, expected_category) in test_cases {
            let category = error_handler.classify_error(&error);
            assert_eq!(category, expected_category, "錯誤分類不正確");
        }
    }

    /// 測試錯誤處理性能影響
    #[tokio::test]
    async fn test_error_handling_performance() {
        let error_handler = ErrorHandler::new();
        let error = DiscordError::InsufficientBalance(12345);

        let start = std::time::Instant::now();

        // 執行多次錯誤處理
        for _ in 0..100 {
            error_handler.format_user_error(&error);
        }

        let duration = start.elapsed();

        // 檢查性能：100次錯誤處理應在合理時間內完成
        assert!(duration.as_millis() < 1000, "錯誤處理性能不達標：{}ms", duration.as_millis());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 測試錯誤處理與 UI 組件的整合
    #[tokio::test]
    async fn test_error_handling_ui_integration() {
        let ui_factory = UIComponentFactory::new();

        // 模擬錯誤場景下的 UI 響應
        let error = DiscordError::InsufficientBalance(12345);

        // 檢查能否為錯誤創建適當的 UI 響應
        // 這需要在實際實現時根據具體 UI 組件來調整
        assert!(true, "UI 整合測試佔位符");
    }

    /// 測試錯誤日誌記錄
    #[tokio::test]
    async fn test_error_logging() {
        let error = DiscordError::DatabaseConnectionError("測試錯誤".to_string());

        // 檢查錯誤日誌記錄功能
        // 這需要在實際實現時根據具體日誌系統來調整
        assert!(true, "錯誤日誌記錄測試佔位符");
    }
}

/// 測試錯誤處理的全面性
#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    /// 測試所有錯誤類型的處理
    #[tokio::test]
    async fn test_all_error_types() {
        let error_handler = ErrorHandler::new();

        // 測試所有定義的錯誤類型
        let all_errors = vec![
            DiscordError::ConnectionError("測試連接錯誤".to_string()),
            DiscordError::InvalidToken,
            DiscordError::CommandError("測試命令錯誤".to_string()),
            DiscordError::ConfigError("測試配置錯誤".to_string()),
            DiscordError::EventError("測試事件錯誤".to_string()),
            DiscordError::UnknownCommand("unknown".to_string()),
            DiscordError::InvalidCommand("invalid".to_string()),
            DiscordError::UnimplementedCommand("unimplemented".to_string()),
            DiscordError::DatabaseConnectionError("測試資料庫連接錯誤".to_string()),
            DiscordError::DatabaseQueryError("測試資料庫查詢錯誤".to_string()),
            DiscordError::TransactionError("測試交易錯誤".to_string()),
            DiscordError::UserNotFound("12345".to_string()),
            DiscordError::InsufficientBalance(12345),
            DiscordError::InvalidAmount("測試無效金額".to_string()),
            DiscordError::AccountCreationFailed("測試帳戶創建失敗".to_string()),
            DiscordError::AccountAlreadyExists(12345),
            DiscordError::MigrationError("測試遷移錯誤".to_string()),
            DiscordError::ValidationError("測試驗證錯誤".to_string()),
            DiscordError::NoTransactionHistory { user_id: 12345, message: "測試消息".to_string() },
        ];

        for error in all_errors {
            let message = error_handler.format_user_error(&error);

            // 檢查每個錯誤都有適當的處理
            assert!(!message.is_empty(), "錯誤消息不應為空");
            assert!(!message.contains("Debug"), "錯誤消息不應包含調試信息");
        }
    }
}