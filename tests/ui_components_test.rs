use droas_bot::services::ui_components::{UIComponentFactory, ButtonType, ButtonInteraction};
use serenity::all::ButtonStyle;
use serenity::model::id::UserId;
use std::time::Duration;
use tokio::time::sleep;

#[cfg(test)]
mod ui_components_tests {
    use super::*;

    #[tokio::test]
    async fn button_creation_test() {
        // 測試按鈕創建功能
        let factory = UIComponentFactory::new();

        // 創建確認按鈕
        let confirm_button = factory.create_button(
            ButtonType::Confirm,
            "confirm_transfer_123_456_500"
        );

        // 驗證確認按鈕屬性
        assert_eq!(confirm_button.style, ButtonStyle::Success);
        assert_eq!(confirm_button.label, Some("✓ 確認".to_string()));
        assert_eq!(confirm_button.custom_id, Some("confirm_transfer_123_456_500".to_string()));

        // 創建取消按鈕
        let cancel_button = factory.create_button(
            ButtonType::Cancel,
            "cancel_transfer_123_456_500"
        );

        // 驗證取消按鈕屬性
        assert_eq!(cancel_button.style, ButtonStyle::Danger);
        assert_eq!(cancel_button.label, Some("✗ 取消".to_string()));
        assert_eq!(cancel_button.custom_id, Some("cancel_transfer_123_456_500".to_string()));
    }

    #[tokio::test]
    async fn button_interaction_parsing_test() {
        // 測試按鈕交互解析
        let factory = UIComponentFactory::new();

        // 測試確認按鈕交互解析
        let confirm_result = factory.parse_button_interaction("confirm_transfer_123_456_500");
        assert!(confirm_result.is_ok(), "確認按鈕交互應該被正確解析");

        let confirm_interaction = confirm_result.unwrap();
        assert_eq!(confirm_interaction.action, "confirm");
        assert_eq!(confirm_interaction.from_user, 123u64);
        assert_eq!(confirm_interaction.to_user, Some(456u64));
        assert_eq!(confirm_interaction.amount, Some("500".to_string()));

        // 測試取消按鈕交互解析
        let cancel_result = factory.parse_button_interaction("cancel_transfer_123_456_500");
        assert!(cancel_result.is_ok(), "取消按鈕交互應該被正確解析");

        let cancel_interaction = cancel_result.unwrap();
        assert_eq!(cancel_interaction.action, "cancel");
        assert_eq!(cancel_interaction.from_user, 123u64);
        assert_eq!(cancel_interaction.to_user, Some(456u64));
        assert_eq!(cancel_interaction.amount, Some("500".to_string()));
    }

    #[tokio::test]
    async fn button_interaction_validation_test() {
        // 測試按鈕交互驗證
        let factory = UIComponentFactory::new();

        // 測試有效的交互ID
        let valid_ids = vec![
            "confirm_transfer_123_456_500",
            "cancel_transfer_123_456_1000",
            "confirm_balance_123",
            "cancel_balance_123"  // 修正：只有支持的格式
        ];

        for id in valid_ids {
            let result = factory.parse_button_interaction(id);
            assert!(result.is_ok(), "有效的交互ID '{}' 應該被正確解析", id);
        }

        // 測試無效的交互ID
        let invalid_ids = vec![
            "invalid_action_123_456_500",
            "confirm_invalid_data",
            "transfer_123_456", // 缺少action和amount
            "", // 空字符串
            "confirm_abc_456_500", // 非數字的用戶ID
            "confirm_123_def_500", // 非數字的目標用戶ID
        ];

        for id in invalid_ids {
            let result = factory.parse_button_interaction(id);
            assert!(result.is_err(), "無效的交互ID '{}' 應該返回錯誤", id);
        }
    }

    #[tokio::test]
    async fn button_permission_test() {
        // 測試按鈕權限驗證
        let factory = UIComponentFactory::new();

        // 創建一個模擬的交互
        let interaction_data = ButtonInteraction {
            action: "confirm".to_string(),
            from_user: 123u64,
            to_user: Some(456u64),
            amount: Some("500".to_string()),
            raw_id: "confirm_transfer_123_456_500".to_string(),
        };

        // 測試用戶權限 - 只有交互發起者可以操作
        assert!(factory.validate_button_permission(&interaction_data, UserId::new(123)),
               "用戶123應該有權限操作自己的交互");
        assert!(!factory.validate_button_permission(&interaction_data, UserId::new(999)),
               "用戶999不應該有權限操作他人的交互");

        // 測試自我轉帳防護
        let self_interaction = ButtonInteraction {
            action: "confirm".to_string(),
            from_user: 123u64,
            to_user: Some(123u64), // 自我轉帳
            amount: Some("500".to_string()),
            raw_id: "confirm_transfer_123_123_500".to_string(),
        };

        assert!(!factory.validate_button_permission(&self_interaction, UserId::new(123)),
               "不應該允許自我轉帳");
    }

    #[tokio::test]
    async fn action_buttons_creation_test() {
        // 測試操作按鈕組創建
        let factory = UIComponentFactory::new();

        let buttons = factory.create_action_buttons("transfer_123_456_500");
        assert_eq!(buttons.len(), 2, "應該創建兩個按鈕（確認和取消）");

        // 驗證按鈕順序和類型
        let confirm_button = &buttons[0];
        let cancel_button = &buttons[1];

        assert_eq!(confirm_button.style, ButtonStyle::Success);
        assert_eq!(cancel_button.style, ButtonStyle::Danger);

        // 驗證按鈕ID格式
        assert_eq!(confirm_button.custom_id, Some("confirm_transfer_123_456_500".to_string()));
        assert_eq!(cancel_button.custom_id, Some("cancel_transfer_123_456_500".to_string()));
    }

    #[tokio::test]
    async fn button_label_localization_test() {
        // 測試按鈕標籤本地化
        let factory = UIComponentFactory::new();

        let confirm_button = factory.create_button(ButtonType::Confirm, "test_id");
        let cancel_button = factory.create_button(ButtonType::Cancel, "test_id");

        // 驗證中文標籤
        assert_eq!(confirm_button.label, Some("✓ 確認".to_string()));
        assert_eq!(cancel_button.label, Some("✗ 取消".to_string()));

        // 測試按鈕標籤包含表情符號
        assert!(confirm_button.label.as_ref().unwrap().contains("✓"));
        assert!(cancel_button.label.as_ref().unwrap().contains("✗"));
    }

    // ========== RED 階段測試 - 按鈕創建功能 ==========

    #[tokio::test]
    async fn test_create_confirmation_buttons() {
        // 測試：為確認操作創建包含「確認」和「取消」按鈕的嵌入消息
        let factory = UIComponentFactory::new();
        let from_user = UserId::new(123);
        let to_user = UserId::new(456);
        let amount = 500.0;

        // 創建確認按鈕（這個功能目前不存在，應該失敗）
        let buttons = factory.create_confirmation_buttons_for_transfer(from_user, to_user, amount);

        // 驗證：返回包含確認和取消按鈕的組件集合
        assert_eq!(buttons.len(), 2, "應該創建兩個按鈕");

        // 驗證確認按鈕屬性
        let confirm_button = &buttons[0];
        assert_eq!(confirm_button.style, ButtonStyle::Success);
        assert!(confirm_button.label.as_ref().unwrap().contains("確認"));
        assert!(confirm_button.custom_id.as_ref().unwrap().starts_with("confirm_transfer_"));

        // 驗證取消按鈕屬性
        let cancel_button = &buttons[1];
        assert_eq!(cancel_button.style, ButtonStyle::Danger);
        assert!(cancel_button.label.as_ref().unwrap().contains("取消"));
        assert!(cancel_button.custom_id.as_ref().unwrap().starts_with("cancel_transfer_"));
    }

    // ========== RED 階段測試 - 按鈕交互處理 ==========

    #[tokio::test]
    async fn test_handle_button_interaction() {
        // 測試：系統能正確處理用戶點擊按鈕的事件
        let factory = UIComponentFactory::new();
        let custom_id = "confirm_transfer_123_456_500";

        // 模擬按鈕交互處理（這個功能目前不存在，應該失敗）
        let result = factory.handle_button_interaction(custom_id, UserId::new(123)).await;

        // 驗證：點擊確認按鈕應該執行對應操作
        assert!(result.is_ok(), "確認按鈕交互處理應該成功");
        let response = result.unwrap();
        assert!(response.contains("成功") || response.contains("確認"), "響應應該包含成功確認信息");

        // 測試：點擊取消按鈕應該取消操作
        let cancel_result = factory.handle_button_interaction("cancel_transfer_123_456_500", UserId::new(123)).await;
        assert!(cancel_result.is_ok(), "取消按鈕交互處理應該成功");
        let cancel_response = cancel_result.unwrap();
        assert!(cancel_response.contains("取消") || cancel_response.contains("已取消"), "響應應該包含取消信息");
    }

    // ========== RED 階段測試 - 按鈕狀態管理 ==========

    #[tokio::test]
    async fn test_button_state_update() {
        // 測試：按鈕在處理後應禁用或顯示適當狀態
        let factory = UIComponentFactory::new();
        let custom_id = "confirm_transfer_123_456_500";

        // 創建按鈕
        let buttons = factory.create_action_buttons("transfer_123_456_500");
        let initial_button = &buttons[0]; // 確認按鈕

        // 測試按鈕狀態更新（這個功能目前不存在，應該失敗）
        let updated_button = factory.update_button_state(initial_button, true).await;

        // 驗證：按鈕狀態應該更新為禁用
        assert!(updated_button.disabled, "按鈕應該被禁用");
        assert!(updated_button.label.as_ref().unwrap().contains("已處理") ||
                updated_button.label.as_ref().unwrap().contains("✓"),
                "按鈕標籤應該顯示處理結果");
    }

    // ========== RED 階段測試 - 超時機制 ==========

    #[tokio::test]
    async fn test_button_timeout() {
        // 測試：確認界面在超時後自動失效
        let factory = UIComponentFactory::new();
        let custom_id = "confirm_transfer_123_456_500";

        // 測試按鈕超時機制（這個功能目前不存在，應該失敗）
        let timeout_duration = Duration::from_millis(100); // 短超時時間用於測試

        // 設置超時
        let timeout_result = factory.set_button_timeout(custom_id, timeout_duration).await;
        assert!(timeout_result.is_ok(), "設置按鈕超時應該成功");

        // 等待超時
        sleep(Duration::from_millis(150)).await;

        // 驗證：按鈕應該自動禁用
        let is_expired = factory.is_button_expired(custom_id).await;
        assert!(is_expired, "按鈕應該已過期");

        // 嘗試使用過期的按鈕應該失敗
        let expired_result = factory.handle_button_interaction(custom_id, UserId::new(123)).await;
        assert!(expired_result.is_err(), "過期的按鈕交互應該失敗");
    }

    // ========== RED 階段測試 - 錯誤處理 ==========

    #[tokio::test]
    async fn test_button_error_handling() {
        // 測試：按鈕交互失敗時顯示適當錯誤消息
        let factory = UIComponentFactory::new();

        // 測試無效的按鈕 ID
        let invalid_ids = vec![
            "invalid_format",
            "confirm_unknown_action_123",
            "",
            "malformed_id_with_no_structure"
        ];

        for invalid_id in invalid_ids {
            let result = factory.handle_button_interaction(invalid_id, UserId::new(123)).await;
            assert!(result.is_err(), "無效的按鈕 ID '{}' 應該返回錯誤", invalid_id);

            // 驗證錯誤消息是用戶友好的
            let error = result.unwrap_err();
            let error_message = format!("{}", error);
            assert!(!error_message.contains("panic"), "錯誤消息不應該包含系統內部錯誤");
            assert!(error_message.len() > 0, "錯誤消息應該有意義的內容");
        }

        // 測試權限錯誤
        let unauthorized_result = factory.handle_button_interaction("confirm_transfer_123_456_500", UserId::new(999)).await;
        assert!(unauthorized_result.is_err(), "未授權的按鈕交互應該失敗");

        let unauthorized_error = unauthorized_result.unwrap_err();
        let error_message = format!("{}", unauthorized_error);
        assert!(error_message.contains("權限") || error_message.contains("授權"),
                "錯誤消息應該提及權限問題");
    }
}