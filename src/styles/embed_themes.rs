/// Embed 主題定義
/// 提供 Discord embed 的顏色主題和樣式配置

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmbedTheme {
    /// 成功主題 - 綠色
    Success,
    /// 信息主題 - 藍色
    Info,
    /// 警告主題 - 黃色
    Warning,
    /// 錯誤主題 - 紅色
    Error,
}

impl EmbedTheme {
    /// 獲取主題對應的顏色值
    pub fn color(self) -> u32 {
        match self {
            EmbedTheme::Success => 0x00FF00,     // 綠色
            EmbedTheme::Info => 0x0099FF,       // 藍色
            EmbedTheme::Warning => 0xFFFF00,    // 黃色
            EmbedTheme::Error => 0xFF0000,      // 紅色
        }
    }

    /// 獲取主題的名稱
    pub fn name(self) -> &'static str {
        match self {
            EmbedTheme::Success => "success",
            EmbedTheme::Info => "info",
            EmbedTheme::Warning => "warning",
            EmbedTheme::Error => "error",
        }
    }

    /// 獲取主題對應的表情符號
    pub fn emoji(self) -> &'static str {
        match self {
            EmbedTheme::Success => "✅",
            EmbedTheme::Info => "ℹ️",
            EmbedTheme::Warning => "⚠️",
            EmbedTheme::Error => "❌",
        }
    }

    /// 根據消息內容自動選擇合適的主題
    pub fn from_message_content(content: &str) -> Self {
        let content_lower = content.to_lowercase();

        if content_lower.contains("錯誤") ||
           content_lower.contains("失敗") ||
           content_lower.contains("error") ||
           content_lower.contains("失敗") {
            EmbedTheme::Error
        } else if content_lower.contains("警告") ||
                  content_lower.contains("注意") ||
                  content_lower.contains("warning") ||
                  content_lower.contains("warn") {
            EmbedTheme::Warning
        } else if content_lower.contains("成功") ||
                  content_lower.contains("完成") ||
                  content_lower.contains("success") ||
                  content_lower.contains("完成") {
            EmbedTheme::Success
        } else {
            EmbedTheme::Info
        }
    }
}

/// Embed 主題配置
#[derive(Debug, Clone)]
pub struct EmbedThemeConfig {
    /// 預設主題
    pub default_theme: EmbedTheme,
    /// 品牌顏色
    pub brand_color: u32,
    /// 是否顯示品牌標識
    pub show_branding: bool,
    /// 品牌名稱
    pub brand_name: String,
    /// 品牌圖標 URL
    pub brand_icon_url: String,
}

impl Default for EmbedThemeConfig {
    fn default() -> Self {
        Self {
            default_theme: EmbedTheme::Info,
            brand_color: 0x0099FF, // 藍色
            show_branding: true,
            brand_name: "DROAS Bot".to_string(),
            brand_icon_url: "https://example.com/bot-avatar.png".to_string(),
        }
    }
}

impl EmbedThemeConfig {
    /// 創建新的主題配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 設置預設主題
    pub fn with_default_theme(mut self, theme: EmbedTheme) -> Self {
        self.default_theme = theme;
        self
    }

    /// 設置品牌顏色
    pub fn with_brand_color(mut self, color: u32) -> Self {
        self.brand_color = color;
        self
    }

    /// 設置是否顯示品牌標識
    pub fn with_branding(mut self, show_branding: bool) -> Self {
        self.show_branding = show_branding;
        self
    }

    /// 設置品牌名稱
    pub fn with_brand_name(mut self, name: &str) -> Self {
        self.brand_name = name.to_string();
        self
    }

    /// 設置品牌圖標 URL
    pub fn with_brand_icon(mut self, icon_url: &str) -> Self {
        self.brand_icon_url = icon_url.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_theme_colors() {
        assert_eq!(EmbedTheme::Success.color(), 0x00FF00);
        assert_eq!(EmbedTheme::Info.color(), 0x0099FF);
        assert_eq!(EmbedTheme::Warning.color(), 0xFFFF00);
        assert_eq!(EmbedTheme::Error.color(), 0xFF0000);
    }

    #[test]
    fn test_embed_theme_names() {
        assert_eq!(EmbedTheme::Success.name(), "success");
        assert_eq!(EmbedTheme::Info.name(), "info");
        assert_eq!(EmbedTheme::Warning.name(), "warning");
        assert_eq!(EmbedTheme::Error.name(), "error");
    }

    #[test]
    fn test_embed_theme_emojis() {
        assert_eq!(EmbedTheme::Success.emoji(), "✅");
        assert_eq!(EmbedTheme::Info.emoji(), "ℹ️");
        assert_eq!(EmbedTheme::Warning.emoji(), "⚠️");
        assert_eq!(EmbedTheme::Error.emoji(), "❌");
    }

    #[test]
    fn test_theme_from_message_content() {
        // 測試成功消息
        assert_eq!(EmbedTheme::from_message_content("操作成功"), EmbedTheme::Success);
        assert_eq!(EmbedTheme::from_message_content("Transfer completed successfully"), EmbedTheme::Success);

        // 測試錯誤消息
        assert_eq!(EmbedTheme::from_message_content("操作失敗"), EmbedTheme::Error);
        assert_eq!(EmbedTheme::from_message_content("System error occurred"), EmbedTheme::Error);

        // 測試警告消息
        assert_eq!(EmbedTheme::from_message_content("警告：餘額不足"), EmbedTheme::Warning);
        assert_eq!(EmbedTheme::from_message_content("Warning: low balance"), EmbedTheme::Warning);

        // 測試信息消息（預設）
        assert_eq!(EmbedTheme::from_message_content("一般信息"), EmbedTheme::Info);
        assert_eq!(EmbedTheme::from_message_content("General information"), EmbedTheme::Info);
    }

    #[test]
    fn test_embed_theme_config_default() {
        let config = EmbedThemeConfig::default();
        assert_eq!(config.default_theme, EmbedTheme::Info);
        assert_eq!(config.brand_color, 0x0099FF);
        assert!(config.show_branding);
        assert_eq!(config.brand_name, "DROAS Bot");
        assert_eq!(config.brand_icon_url, "https://example.com/bot-avatar.png");
    }

    #[test]
    fn test_embed_theme_config_builder() {
        let config = EmbedThemeConfig::new()
            .with_default_theme(EmbedTheme::Success)
            .with_brand_color(0xFF0000)
            .with_branding(false)
            .with_brand_name("Custom Bot")
            .with_brand_icon("https://example.com/custom-icon.png");

        assert_eq!(config.default_theme, EmbedTheme::Success);
        assert_eq!(config.brand_color, 0xFF0000);
        assert!(!config.show_branding);
        assert_eq!(config.brand_name, "Custom Bot");
        assert_eq!(config.brand_icon_url, "https://example.com/custom-icon.png");
    }
}