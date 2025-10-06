use droas_bot::discord_gateway::{DiscordGateway, ConnectionStatus};
use droas_bot::services::ui_components::{UIComponentFactory, ButtonType};

#[tokio::test]
async fn test_discord_gateway_button_interaction_flow() {
    // 創建 Discord Gateway
    let mut gateway = DiscordGateway::new();

    // 驗證初始狀態
    assert_eq!(gateway.get_status().await, ConnectionStatus::Disconnected);

    // 連接到模擬 Discord
    let result = gateway.connect().await;
    assert!(result.is_ok());
    assert_eq!(gateway.get_status().await, ConnectionStatus::Connected);

    // 測試命令處理（確保原有功能仍然正常）
    let ping_result = gateway.handle_command("!ping").await;
    assert!(ping_result.is_ok());
    assert_eq!(ping_result.unwrap(), "Pong!");
}

#[tokio::test]
async fn test_ui_component_factory_button_creation() {
    let factory = UIComponentFactory::new();

    // 創建轉帳確認按鈕
    let from_user = serenity::model::id::UserId::new(123);
    let to_user = serenity::model::id::UserId::new(456);
    let amount = 100.0;

    let buttons = factory.create_transfer_buttons(from_user, to_user, amount);
    assert_eq!(buttons.len(), 2);

    // 驗證確認按鈕
    let confirm_button = &buttons[0];
    assert_eq!(confirm_button.custom_id, Some("confirm_transfer_123_456_100".to_string()));
    assert!(!confirm_button.disabled);

    // 驗證取消按鈕
    let cancel_button = &buttons[1];
    assert_eq!(cancel_button.custom_id, Some("cancel_transfer_123_456_100".to_string()));
    assert!(!cancel_button.disabled);
}

#[tokio::test]
async fn test_ui_component_factory_interaction_handling() {
    let factory = UIComponentFactory::new();

    // 測試轉帳確認交互
    let custom_id = "confirm_transfer_123_456_500";
    let user_id = serenity::model::id::UserId::new(123);

    let result = factory.handle_button_interaction(custom_id, user_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "操作已確認並執行成功");

    // 測試取消交互
    let cancel_id = "cancel_balance_789";
    let cancel_user = serenity::model::id::UserId::new(789);

    let result = factory.handle_button_interaction(cancel_id, cancel_user).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "操作已取消");
}

#[tokio::test]
async fn test_ui_component_factory_permission_validation() {
    let factory = UIComponentFactory::new();

    // 創建一個來自用戶 123 的交互
    let custom_id = "confirm_transfer_123_456_500";

    // 測試正確的用戶
    let correct_user = serenity::model::id::UserId::new(123);
    let result = factory.handle_button_interaction(custom_id, correct_user).await;
    assert!(result.is_ok());

    // 測試錯誤的用戶
    let wrong_user = serenity::model::id::UserId::new(999);
    let result = factory.handle_button_interaction(custom_id, wrong_user).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("無權限執行此操作"));
}

#[tokio::test]
async fn test_button_interaction_with_timeout() {
    let factory = UIComponentFactory::new();

    let custom_id = "confirm_transfer_123_456_500";
    let user_id = serenity::model::id::UserId::new(123);

    // 設置按鈕超時
    factory.set_button_timeout(custom_id, std::time::Duration::from_secs(10)).await.unwrap();

    // 驗證按鈕已過期（簡化的超時邏輯）
    assert!(factory.is_button_expired(custom_id).await);

    // 嘗試處理過期的按鈕
    let result = factory.handle_button_interaction(custom_id, user_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("按鈕已過期"));
}

#[tokio::test]
async fn test_button_state_updates() {
    let factory = UIComponentFactory::new();

    let button = factory.create_button(ButtonType::Confirm, "test_button");
    assert!(!button.disabled);

    // 更新按鈕狀態為已處理
    let updated_button = factory.update_button_state(&button, true).await;
    assert!(updated_button.disabled);
    assert!(updated_button.label.as_ref().unwrap().contains("已處理"));

    // 更新按鈕狀態為未處理
    let reset_button = factory.update_button_state(&button, false).await;
    assert!(!reset_button.disabled);
}

#[tokio::test]
async fn test_button_id_validation_and_generation() {
    let factory = UIComponentFactory::new();

    // 測試按鈕 ID 驗證
    assert!(factory.validate_button_id("confirm_transfer_123_456"));
    assert!(factory.validate_button_id("cancel_balance_789"));
    assert!(!factory.validate_button_id("invalid_format"));
    assert!(!factory.validate_button_id("unknown_action_123"));

    // 測試按鈕 ID 生成
    let button_id = factory.generate_button_id("confirm", "transfer_123_456");
    assert_eq!(button_id, "confirm_transfer_123_456");

    let button_id = factory.generate_button_id("cancel", "balance_789");
    assert_eq!(button_id, "cancel_balance_789");
}

#[tokio::test]
async fn test_button_interaction_parsing() {
    let factory = UIComponentFactory::new();

    // 測試轉帳交互解析
    let transfer_id = "confirm_transfer_123_456_500";
    let result = factory.parse_button_interaction(transfer_id);
    assert!(result.is_ok());

    let interaction = result.unwrap();
    assert_eq!(interaction.action, "confirm");
    assert_eq!(interaction.from_user, 123);
    assert_eq!(interaction.to_user, Some(456));
    assert_eq!(interaction.amount, Some("500".to_string()));

    // 測試餘額查詢交互解析
    let balance_id = "cancel_balance_789";
    let result = factory.parse_button_interaction(balance_id);
    assert!(result.is_ok());

    let interaction = result.unwrap();
    assert_eq!(interaction.action, "cancel");
    assert_eq!(interaction.from_user, 789);
    assert_eq!(interaction.to_user, None);
    assert_eq!(interaction.amount, None);

    // 測試無效格式
    let invalid_id = "invalid_format";
    let result = factory.parse_button_interaction(invalid_id);
    assert!(result.is_err());
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_button_interaction_workflow() {
        // 1. 創建 Discord Gateway 和 UI 工廠
        let mut gateway = DiscordGateway::new();
        let factory = UIComponentFactory::new();

        // 2. 連接到 Discord
        let connect_result = gateway.connect().await;
        assert!(connect_result.is_ok());

        // 3. 創建轉帳按鈕
        let from_user = serenity::model::id::UserId::new(123);
        let to_user = serenity::model::id::UserId::new(456);
        let amount = 100.0;

        let buttons = factory.create_transfer_buttons(from_user, to_user, amount);
        assert_eq!(buttons.len(), 2);

        // 4. 模擬用戶點擊確認按鈕
        let confirm_button = &buttons[0];
        let custom_id = confirm_button.custom_id.as_ref().unwrap();

        let result = factory.handle_button_interaction(custom_id, from_user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "操作已確認並執行成功");

        // 5. 模擬用戶點擊取消按鈕
        let cancel_button = &buttons[1];
        let cancel_id = cancel_button.custom_id.as_ref().unwrap();

        let result = factory.handle_button_interaction(cancel_id, from_user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "操作已取消");

        // 6. 驗證原有功能仍然正常
        let ping_result = gateway.handle_command("!ping").await;
        assert!(ping_result.is_ok());
        assert_eq!(ping_result.unwrap(), "Pong!");
    }

    #[tokio::test]
    async fn test_button_interaction_with_error_handling() {
        let factory = UIComponentFactory::new();

        // 測試無權限操作
        let custom_id = "confirm_transfer_123_456_500";
        let wrong_user = serenity::model::id::UserId::new(999);

        let result = factory.handle_button_interaction(custom_id, wrong_user).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("無權限執行此操作"));

        // 測試無效按鈕 ID
        let invalid_id = "invalid_button_id";
        let user = serenity::model::id::UserId::new(123);

        let result = factory.handle_button_interaction(invalid_id, user).await;
        assert!(result.is_err());
    }
}