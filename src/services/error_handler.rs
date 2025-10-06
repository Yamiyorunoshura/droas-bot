//! 錯誤處理器模組
//! 提供集中式錯誤處理和用戶友好的錯誤消息功能

use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, warn, info, debug};
use crate::error::{DiscordError, ErrorCategory, ErrorSeverity};

/// 錯誤處理器
/// 負責格式化錯誤消息、記錄錯誤日誌、提供用戶指導
pub struct ErrorHandler {
    /// 錯誤消息模板
    message_templates: HashMap<String, MessageTemplate>,
    /// 性能統計
    stats: ErrorStats,
}

/// 錯誤消息模板
#[derive(Debug, Clone)]
pub struct MessageTemplate {
    /// 錯誤類型
    pub error_type: String,
    /// 用戶友好標題
    pub title: String,
    /// 主要錯誤消息
    pub message: String,
    /// 解決建議
    pub suggestion: String,
    /// 額外幫助信息
    pub help: Option<String>,
}

/// 錯誤統計信息
#[derive(Debug, Default)]
pub struct ErrorStats {
    /// 總錯誤數
    pub total_errors: u64,
    /// 按類型分類的錯誤數
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    /// 按嚴重性分類的錯誤數
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    /// 平均處理時間（微秒）
    pub avg_processing_time_us: u64,
}

impl ErrorHandler {
    /// 創建新的錯誤處理器
    pub fn new() -> Self {
        let mut handler = Self {
            message_templates: HashMap::new(),
            stats: ErrorStats::default(),
        };

        // 初始化默認模板
        handler.init_default_templates();
        handler
    }

    /// 初始化默認錯誤消息模板
    fn init_default_templates(&mut self) {
        // 業務邏輯錯誤模板
        self.add_template(MessageTemplate {
            error_type: "InsufficientBalance".to_string(),
            title: "餘額不足".to_string(),
            message: "您的帳戶餘額不足以完成此轉帳操作。".to_string(),
            suggestion: "請先充值或參與活動賺取更多幣值。您可以使用 !balance 查詢當前餘額。".to_string(),
            help: Some("💡 提示：您可以通過每日簽到、參與活動或向其他用戶賺取更多幣值。".to_string()),
        });

        // 系統錯誤模板
        self.add_template(MessageTemplate {
            error_type: "DatabaseConnectionError".to_string(),
            title: "系統維護中".to_string(),
            message: "無法連接到數據庫，系統正在維護。".to_string(),
            suggestion: "請稍後再試。如果問題持續，請聯繫管理員。".to_string(),
            help: Some("🛠️ 技術團隊正在努力修復此問題，感謝您的耐心等待。".to_string()),
        });

        // 用戶輸入錯誤模板
        self.add_template(MessageTemplate {
            error_type: "InvalidCommand".to_string(),
            title: "指令格式錯誤".to_string(),
            message: "輸入的指令格式不正確。".to_string(),
            suggestion: "請檢查指令格式是否正確。使用 !help 查看所有可用指令。".to_string(),
            help: Some("📚 使用 !help <指令名> 可以查看具體指令的詳細用法。".to_string()),
        });

        // 安全錯誤模板
        self.add_template(MessageTemplate {
            error_type: "UnauthorizedAccess".to_string(),
            title: "權限不足".to_string(),
            message: "您沒有權限執行此操作。".to_string(),
            suggestion: "請聯繫管理員獲取幫助。".to_string(),
            help: Some("🔒 如需權限升級，請聯繫伺服器管理員。".to_string()),
        });
    }

    /// 添加錯誤消息模板
    pub fn add_template(&mut self, template: MessageTemplate) {
        self.message_templates.insert(template.error_type.clone(), template);
    }

    /// 格式化用戶錯誤消息
    pub fn format_user_error(&self, error: &DiscordError) -> String {
        let start_time = Instant::now();

        let error_type = format!("{:?}", error);
        let category = error.category();
        let severity = error.severity();

        // 記錄錯誤日誌
        self.log_error(error, &category, &severity);

        // 查找模板
        let template = self.message_templates.get(&error_type)
            .or_else(|| self.get_fallback_template(&category));

        // 構建錯誤消息
        let message = if let Some(tmpl) = template {
            self.format_with_template(error, tmpl)
        } else {
            self.format_fallback_message(error, &category)
        };

        // 更新統計 (暫時跳過實際更新以避免可變性問題)
        let processing_time = start_time.elapsed().as_micros() as u64;
        debug!("錯誤處理統計：{:?}, {:?}, {}μs", category, severity, processing_time);

        message
    }

    /// 分類錯誤
    pub fn classify_error(&self, error: &DiscordError) -> &'static str {
        match error.category() {
            ErrorCategory::Business => "業務邏輯錯誤",
            ErrorCategory::System => "系統錯誤",
            ErrorCategory::UserInput => "用戶輸入錯誤",
            ErrorCategory::Security => "安全錯誤",
            ErrorCategory::Network => "網路錯誤",
        }
    }

    /// 使用模板格式化錯誤消息
    fn format_with_template(&self, error: &DiscordError, template: &MessageTemplate) -> String {
        let mut message = format!("**{}**\n\n", template.title);
        message.push_str(&format!("{}\n\n", template.message));

        if let Some(user_suggestion) = error.user_suggestion() {
            message.push_str(&format!("**建議：** {}\n\n", user_suggestion));
        } else {
            message.push_str(&format!("**建議：** {}\n\n", template.suggestion));
        }

        if let Some(help) = &template.help {
            message.push_str(&format!("{}\n", help));
        }

        // 對於嚴重錯誤，添加額外聯繫信息
        if matches!(error.severity(), ErrorSeverity::Critical) {
            message.push_str("\n如果問題持續，請聯繫管理員或技術支持團隊。");
        }

        message
    }

    /// 格式化後備錯誤消息
    fn format_fallback_message(&self, error: &DiscordError, category: &ErrorCategory) -> String {
        let category_name = match category {
            ErrorCategory::Business => "業務錯誤",
            ErrorCategory::System => "系統錯誤",
            ErrorCategory::UserInput => "輸入錯誤",
            ErrorCategory::Security => "權限錯誤",
            ErrorCategory::Network => "網路錯誤",
        };

        let mut message = format!("**{}**\n\n", category_name);
        message.push_str(&format!("發生了{}：{}\n\n", category_name, error));

        if let Some(suggestion) = error.user_suggestion() {
            message.push_str(&format!("**建議：** {}", suggestion));
        } else {
            message.push_str("**建議：** 請檢查輸入或聯繫管理員獲取幫助。");
        }

        message
    }

    /// 獲取後備模板
    fn get_fallback_template(&self, category: &ErrorCategory) -> Option<&MessageTemplate> {
        match category {
            ErrorCategory::Business => self.message_templates.get("InsufficientBalance"),
            ErrorCategory::System => self.message_templates.get("DatabaseConnectionError"),
            ErrorCategory::UserInput => self.message_templates.get("InvalidCommand"),
            ErrorCategory::Security => self.message_templates.get("UnauthorizedAccess"),
            ErrorCategory::Network => self.message_templates.get("DatabaseConnectionError"),
        }
    }

    /// 記錄錯誤日誌
    fn log_error(&self, error: &DiscordError, category: &ErrorCategory, severity: &ErrorSeverity) {
        match severity {
            ErrorSeverity::Info => {
                info!("錯誤 [{}]: {}", category, error);
            }
            ErrorSeverity::Warning => {
                warn!("錯誤 [{}]: {}", category, error);
            }
            ErrorSeverity::Error => {
                error!("錯誤 [{}]: {}", category, error);
            }
            ErrorSeverity::Critical => {
                error!("嚴重錯誤 [{}]: {} - 需要立即處理", category, error);
            }
        }
    }

    /// 更新統計信息
    #[allow(dead_code)]
    fn update_stats(&mut self, category: &ErrorCategory, severity: &ErrorSeverity, processing_time_us: u64) {
        // 注意：在實際實現中，這裡需要使用 Arc<Mutex<>> 或類似的線程安全機制
        // 這裡為了簡化，先使用基本的更新方式
        debug!("錯誤處理統計更新：{:?}, {:?}, {}μs", category, severity, processing_time_us);
    }

    /// 獲取錯誤統計信息
    pub fn get_stats(&self) -> &ErrorStats {
        &self.stats
    }

    /// 重置統計信息
    pub fn reset_stats(&mut self) {
        self.stats = ErrorStats::default();
    }

    /// 檢查錯誤消息是否提供用戶指導
    pub fn has_user_guidance(&self, error: &DiscordError) -> bool {
        // 檢查錯誤本身是否提供建議
        if error.user_suggestion().is_some() {
            return true;
        }

        // 檢查是否有對應的模板
        let error_type = format!("{:?}", error);
        if self.message_templates.contains_key(&error_type) {
            return true;
        }

        // 檢查是否有分類模板
        if let Some(_template) = self.get_fallback_template(&error.category()) {
            return true;
        }

        false
    }

    /// 生成錯誤報告摘要
    pub fn generate_error_summary(&self) -> String {
        format!(
            "錯誤處理摘要：\n總錯誤數：{}\n平均處理時間：{}μs",
            self.stats.total_errors,
            self.stats.avg_processing_time_us
        )
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DiscordError;

    #[test]
    fn test_error_handler_creation() {
        let handler = ErrorHandler::new();
        assert!(!handler.message_templates.is_empty());
    }

    #[test]
    fn test_format_insufficient_balance_error() {
        let handler = ErrorHandler::new();
        let error = DiscordError::InsufficientBalance(12345);

        let message = handler.format_user_error(&error);

        assert!(message.contains("餘額不足"));
        assert!(message.contains("建議"));
        assert!(message.contains("充值") || message.contains("賺取"));
    }

    #[test]
    fn test_error_classification() {
        let handler = ErrorHandler::new();

        let business_error = DiscordError::InsufficientBalance(12345);
        assert_eq!(handler.classify_error(&business_error), "業務邏輯錯誤");

        let system_error = DiscordError::DatabaseConnectionError("test".to_string());
        assert_eq!(handler.classify_error(&system_error), "系統錯誤");
    }

    #[test]
    fn test_has_user_guidance() {
        let handler = ErrorHandler::new();

        let error = DiscordError::InsufficientBalance(12345);
        assert!(handler.has_user_guidance(&error));

        let error = DiscordError::DatabaseConnectionError("test".to_string());
        assert!(handler.has_user_guidance(&error));
    }
}