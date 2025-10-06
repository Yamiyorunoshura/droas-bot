// Command Router 集成測試
// 測試修復後的 Command Router 整合功能

use droas_bot::command_router::CommandRouter;
use droas_bot::services::{MessageService, HelpService};
use droas_bot::error::DiscordError;
use std::sync::Arc;
use std::str::FromStr;
use bigdecimal::BigDecimal;
use chrono::Utc;

#[tokio::test]
async fn test_command_router_balance_integration() {
    // 測試 Command Router 與 Balance Service 的整合

    // 創建 Message Service
    let message_service = Arc::new(MessageService::new());

    // 創建 Command Router 並設置服務
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 測試解析 balance 指令
    let command_result = command_router.parse_command("!balance").await;
    assert!(command_result.is_ok(), "解析 !balance 指令應該成功");

    let command_result = command_result.unwrap();
    assert_eq!(format!("{:?}", command_result.command), "Balance");

    // 測試路由 balance 指令（沒有設置 BalanceService，應該返回錯誤）
    let route_result = command_router.route_command(&command_result).await;
    assert!(route_result.is_err(), "沒有設置 BalanceService 時應該返回錯誤");

    match route_result.unwrap_err() {
        DiscordError::UnimplementedCommand(msg) => {
            assert!(msg.contains("餘額服務未初始化"));
        }
        _ => panic!("應該返回 UnimplementedCommand 錯誤"),
    }
}

#[tokio::test]
async fn test_command_router_help_integration() {
    // 測試 Command Router 的 help 指令整合

    let message_service = Arc::new(MessageService::new());
    let help_service = Arc::new(HelpService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service))
        .with_help_service(Arc::clone(&help_service));

    // 測試解析 help 指令
    let command_result = command_router.parse_command("!help").await;
    assert!(command_result.is_ok(), "解析 !help 指令應該成功");

    let command_result = command_result.unwrap();

    // 測試路由 help 指令
    let route_result = command_router.route_command(&command_result).await;
    assert!(route_result.is_ok(), "路由 !help 指令應該成功");

    let response = route_result.unwrap();
    assert!(response.contains("DROAS 經濟機器人幫助"));
    assert!(response.contains("balance"));
    assert!(response.contains("help"));
}

#[tokio::test]
async fn test_command_router_unknown_command() {
    // 測試未知指令的處理

    let message_service = Arc::new(MessageService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 測試解析未知指令
    let command_result = command_router.parse_command("!unknown").await;
    assert!(command_result.is_err(), "解析未知指令應該失敗");

    match command_result.unwrap_err() {
        DiscordError::UnknownCommand(cmd) => {
            assert_eq!(cmd, "unknown");
        }
        _ => panic!("應該返回 UnknownCommand 錯誤"),
    }
}

#[tokio::test]
async fn test_command_router_balance_with_user_id() {
    // 測試帶有用戶 ID 的 balance 指令

    let message_service = Arc::new(MessageService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 手動創建一個帶有用戶 ID 的 CommandResult
    let command_text = "!balance";
    let mut command_result = command_router.parse_command(command_text).await.unwrap();
    command_result.user_id = Some(12345);
    command_result.username = Some("TestUser".to_string());

    // 測試路由（沒有 BalanceService，應該返回服務未初始化錯誤）
    let route_result = command_router.route_command(&command_result).await;
    assert!(route_result.is_err(), "沒有 BalanceService 應該失敗");

    // 錯誤應該是服務未初始化，而不是缺少用戶 ID
    match route_result.unwrap_err() {
        DiscordError::UnimplementedCommand(msg) => {
            assert!(msg.contains("餘額服務未初始化"));
        }
        _ => panic!("應該返回 UnimplementedCommand 錯誤"),
    }
}

#[tokio::test]
async fn test_message_service_balance_format() {
    // 測試 Message Service 的餘額格式化功能

    let message_service = MessageService::new();
    let balance = BigDecimal::from_str("1000.50").unwrap();
    let created_at = Utc::now();

    let response = message_service.format_balance_response(
        12345,
        "TestUser",
        &balance,
        Some(created_at),
    );

    assert!(response.is_ok(), "格式化餘額響應應該成功");

    let response = response.unwrap();
    assert!(response.is_embed);
    assert_eq!(response.title, Some("💰 帳戶餘額查詢".to_string()));
    assert_eq!(response.fields.len(), 4);

    // 測試轉換為 Discord 字符串
    let discord_str = message_service.to_discord_string(&response);
    assert!(discord_str.contains("帳戶餘額查詢"));
    assert!(discord_str.contains("TestUser"));
    assert!(discord_str.contains("1000.50"));
}

#[tokio::test]
async fn test_message_service_error_format() {
    // 測試 Message Service 的錯誤格式化功能

    let message_service = MessageService::new();

    // 測試用戶未找到錯誤
    let user_error = DiscordError::UserNotFound("用戶不存在".to_string());
    let response = message_service.format_error_response(&user_error);

    assert!(response.is_embed);
    assert_eq!(response.title, Some("❌ 帳戶錯誤".to_string()));
    assert_eq!(response.description, Some("用戶不存在".to_string()));
    assert_eq!(response.color, Some(0xFF0000));

    // 測試未知指令錯誤
    let unknown_error = DiscordError::UnknownCommand("!test".to_string());
    let response = message_service.format_error_response(&unknown_error);

    assert_eq!(response.title, Some("❓ 未知指令".to_string()));
    assert!(response.description.unwrap().contains("!test"));
}

#[tokio::test]
async fn test_command_router_transfer_integration() {
    // 測試 Command Router 與 Transfer Service 的整合

    let message_service = Arc::new(MessageService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 測試解析 transfer 指令 - 基本格式
    let command_result = command_router.parse_command("!transfer @user 100").await;
    assert!(command_result.is_ok(), "解析 !transfer @user 100 指令應該成功");

    let command_result = command_result.unwrap();
    assert_eq!(format!("{:?}", command_result.command), "Transfer");

    // 測試路由 transfer 指令（沒有設置 TransferService，應該返回錯誤）
    let route_result = command_router.route_command(&command_result).await;
    assert!(route_result.is_err(), "沒有設置 TransferService 時應該返回錯誤");

    match route_result.unwrap_err() {
        DiscordError::UnimplementedCommand(msg) => {
            assert!(msg.contains("轉帳服務未初始化") || msg.contains("Transfer Service"));
        }
        _ => panic!("應該返回 UnimplementedCommand 錯誤"),
    }
}

#[tokio::test]
async fn test_command_router_transfer_parse_variations() {
    // 測試不同格式的 transfer 指令解析

    let message_service = Arc::new(MessageService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 測試帶有數字 ID 的格式
    let command_result = command_router.parse_command("!transfer 123456789 50.5").await;
    assert!(command_result.is_ok(), "解析 !transfer 123456789 50.5 應該成功");

    // 測試帶有用戶名的格式
    let command_result = command_router.parse_command("!transfer @Username 25").await;
    assert!(command_result.is_ok(), "解析 !transfer @Username 25 應該成功");

    // 測試無效的 transfer 格式 - 缺少參數
    let command_result = command_router.parse_command("!transfer").await;
    assert!(command_result.is_ok(), "解析 !transfer 應該成功，但後續路由會驗證參數");

    // 測試路由時應該失敗，因為缺少參數
    let route_result = command_router.route_command(&command_result.unwrap()).await;
    assert!(route_result.is_err(), "路由缺少參數的 transfer 指令應該失敗");

    // 測試無效的 transfer 格式 - 無效金額
    let _command_result = command_router.parse_command("!transfer @user abc").await;
    // 這個可能會成功解析，但在執行時驗證金額
    // 具體行為取決於實作
}

#[tokio::test]
async fn test_command_router_transfer_with_user_context() {
    // 測試帶有用戶上下文的 transfer 指令

    let message_service = Arc::new(MessageService::new());
    let command_router = CommandRouter::new()
        .with_message_service(Arc::clone(&message_service));

    // 手動創建帶有用戶信息的 CommandResult
    let command_text = "!transfer @target_user 75";
    let mut command_result = command_router.parse_command(command_text).await.unwrap();
    command_result.user_id = Some(99999);
    command_result.username = Some("TestSender".to_string());

    // 測試路由（沒有 TransferService，應該返回服務未初始化錯誤）
    let route_result = command_router.route_command(&command_result).await;
    assert!(route_result.is_err(), "沒有 TransferService 應該失敗");

    // 錯誤應該是服務未初始化，而不是缺少用戶 ID
    match route_result.unwrap_err() {
        DiscordError::UnimplementedCommand(msg) => {
            assert!(msg.contains("轉帳服務未初始化") || msg.contains("Transfer Service"));
        }
        _ => panic!("應該返回 UnimplementedCommand 錯誤"),
    }
}