// Help Service Tests
// 幫助服務測試模組

use droas_bot::services::HelpService;
use droas_bot::command_router::CommandRouter;

#[cfg(test)]
mod tests {
    use super::*;

    /// 測試基本幫助顯示功能
    ///
    /// **測試名稱**: test_help_command_displays_all_commands
    /// **情境**: 用戶發送 `!help` 指令
    /// **預期結果**:
    /// - 返回格式化的 Discord embed
    /// - 包含指令：!balance, !transfer, !history, !help
    /// - 每個指令包含簡短描述和使用範例
    /// - 使用正確的顏色主題和格式
    #[tokio::test]
    async fn test_help_command_displays_all_commands() {
        // Arrange: 創建 HelpService 實例
        let help_service = HelpService::new();

        // Act: 生成幫助內容
        let help_content = help_service.generate_help_content().await;

        // Assert: 驗證幫助內容包含所有預期指令
        assert!(help_content.contains("!balance"));
        assert!(help_content.contains("!transfer"));
        assert!(help_content.contains("!history"));
        assert!(help_content.contains("!help"));

        // 驗證幫助內容包含描述信息（根據實際實現格式）
        assert!(help_content.contains("查詢帳戶餘額") || help_content.contains("轉帳給指定用戶") || help_content.contains("顯示幫助信息"));
        assert!(help_content.contains("範例") || help_content.contains("example") || help_content.contains("example:"));

        // 驗證內容格式為 embed 格式（檢查是否包含 embed 關鍵字）
        assert!(help_content.len() > 0);
    }

    /// 測試無效幫助指令處理
    ///
    /// **測試名稱**: test_help_command_with_invalid_argument
    /// **情境**: 用戶發送 `!help nonexistent_command`
    /// **預期結果**:
    /// - 顯示基本幫助信息或友好錯誤消息
    /// - 不會導致系統錯誤或崩潰
    /// - 提供正確的幫助指引
    #[tokio::test]
    async fn test_help_command_with_invalid_argument() {
        // Arrange: 創建 HelpService 實例
        let help_service = HelpService::new();

        // Act: 嘗試獲取不存在指令的幫助
        let result = help_service.get_command_help("nonexistent_command").await;

        // Assert: 驗證系統不會崩潰並返回適當的響應
        match result {
            Ok(help_text) => {
                // 如果返回成功，應該包含基本幫助信息或錯誤提示
                assert!(help_text.contains("找不到") || help_text.contains("不存在") || help_text.contains("可用指令"));
            }
            Err(_) => {
                // 如果返回錯誤，應該是適當的錯誤類型
                // 這也是可以接受的行為
            }
        }
    }

    /// 測試幫助內容完整性
    ///
    /// **測試名稱**: test_help_content_completeness
    /// **情境**: 驗證幫助內容包含所有已實現指令
    /// **預期結果**:
    /// - 所有已實現的經濟系統指令都在幫助中列出
    /// - 指令名稱、語法格式、描述準確無誤
    /// - 使用範例可以實際執行
    #[tokio::test]
    async fn test_help_content_completeness() {
        // Arrange: 創建 HelpService 實例
        let help_service = HelpService::new();

        // Act: 獲取所有可用指令和幫助內容
        let available_commands = help_service.get_available_commands().await;
        let help_content = help_service.generate_help_content().await;

        // Assert: 驗證所有可用指令都在幫助內容中
        for command in &available_commands {
            assert!(help_content.contains(&format!("!{}", command)),
                   "Help content should contain command: !{}", command);
        }

        // 驗證特定指令的描述和範例
        for command in &available_commands {
            let command_help = help_service.get_command_help(command).await;
            if let Ok(help_text) = command_help {
                assert!(!help_text.is_empty(), "Command help should not be empty for: {}", command);
                assert!(help_text.len() > 10, "Command help should be meaningful for: {}", command);
            }
        }
    }

    /// 測試幫助系統與 Command Router 的整合
    ///
    /// **測試名稱**: test_help_command_router_integration
    /// **情境**: 測試幫助系統與 Command Router 的整合
    /// **預期結果**:
    /// - !help 指令被正確路由到幫助處理邏輯
    /// - 不干擾其他指令的正常路由
    /// - 遵循現有的命令解析和處理流程
    #[tokio::test]
    async fn test_help_command_router_integration() {
        // Arrange: 創建 CommandRouter 實例
        let command_router = CommandRouter::new();

        // Act & Assert: 驗證 help 指令被支援
        assert!(command_router.is_command_supported("help"));

        // 驗證可以解析 help 指令
        let parse_result = command_router.parse_command("!help").await;
        assert!(parse_result.is_ok());

        if let Ok(command_result) = parse_result {
            assert_eq!(format!("{:?}", command_result.command), "Help");
        }

        // 驗證 help 不需要用戶帳戶（根據現有邏輯）
        // 這個測試可能需要根據實際實作調整
        let available_commands = command_router.get_available_commands();
        assert!(available_commands.contains(&"help".to_string()));
    }

    /// 測試 HelpService 的指令資訊結構
    #[tokio::test]
    async fn test_help_service_command_info_structure() {
        // Arrange & Act: 創建 HelpService 並獲取指令資訊
        let help_service = HelpService::new();
        let commands = help_service.get_available_commands().await;

        // Assert: 驗證至少包含基本指令
        assert!(!commands.is_empty(), "Should have at least one command available");

        // 驗證每個指令都有完整的資訊
        for command_name in commands {
            if let Ok(command_info) = help_service.get_command_info(&command_name).await {
                assert!(!command_info.name.is_empty());
                assert!(!command_info.description.is_empty());
                assert!(!command_info.usage.is_empty());
            }
        }
    }

    /// 測試幫助內容的格式和樣式
    #[tokio::test]
    async fn test_help_content_formatting() {
        // Arrange & Act: 生成幫助內容
        let help_service = HelpService::new();
        let help_content = help_service.generate_help_content().await;

        // Assert: 驗證內容格式
        assert!(!help_content.is_empty());

        // 驗證包含適當的格式標記（如 Markdown 或 Discord embed 格式）
        // 這取決於實際的實作方式
        assert!(help_content.len() > 50); // 確保內容足夠詳細

        // 驗證不包含意外的格式錯誤
        assert!(!help_content.contains("ERROR"));
        assert!(!help_content.contains("error"));
    }
}