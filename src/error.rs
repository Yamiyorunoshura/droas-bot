use thiserror::Error;
use sqlx;

/// 錯誤分類
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// 業務邏輯錯誤：與業務規則相關的錯誤
    Business,
    /// 系統錯誤：基礎設施或外部依賴問題
    System,
    /// 用戶輸入錯誤：用戶輸入格式或內容錯誤
    UserInput,
    /// 安全錯誤：權限或身份驗證相關錯誤
    Security,
    /// 網路錯誤：網路連接或通信問題
    Network,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Business => write!(f, "業務邏輯錯誤"),
            ErrorCategory::System => write!(f, "系統錯誤"),
            ErrorCategory::UserInput => write!(f, "用戶輸入錯誤"),
            ErrorCategory::Security => write!(f, "安全錯誤"),
            ErrorCategory::Network => write!(f, "網路錯誤"),
        }
    }
}

/// 錯誤嚴重性等級
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// 信息：一般提示信息
    Info,
    /// 警告：潛在問題，但不影響功能
    Warning,
    /// 錯誤：功能無法正常執行
    Error,
    /// 嚴重：系統無法繼續運行
    Critical,
}

#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("Discord API connection failed: {0}")]
    ConnectionError(String),

    #[error("Invalid Discord token")]
    InvalidToken,

    #[error("Command processing failed: {0}")]
    CommandError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Event handling error: {0}")]
    EventError(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Invalid command format: {0}")]
    InvalidCommand(String),

    #[error("Unimplemented command: {0}")]
    UnimplementedCommand(String),

    #[error("Database connection failed: {0}")]
    DatabaseConnectionError(String),

    #[error("Database query failed: {0}")]
    DatabaseQueryError(String),

    #[error("Transaction failed: {0}")]
    TransactionError(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Insufficient balance for user {0}")]
    InsufficientBalance(i64),

    #[error("Invalid transaction amount: {0}")]
    InvalidAmount(String),

    #[error("Account creation failed: {0}")]
    AccountCreationFailed(String),

    #[error("User account already exists: {0}")]
    AccountAlreadyExists(i64),

    #[error("Database migration failed: {0}")]
    MigrationError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("No transaction history found for user {user_id}: {message}")]
    NoTransactionHistory { user_id: i64, message: String },

    #[error("Unauthorized access attempt by user {user_id}: {message}")]
    UnauthorizedAccess { user_id: i64, message: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Confirmation required: {0}")]
    ConfirmationRequired(String),

    #[error("Additional verification required: {0}")]
    AdditionalVerificationRequired(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Network error: {message}")]
    NetworkError { message: String },
}

impl DiscordError {
    /// 獲取錯誤分類
    pub fn category(&self) -> ErrorCategory {
        match self {
            DiscordError::InsufficientBalance(_) |
            DiscordError::InvalidAmount(_) |
            DiscordError::AccountAlreadyExists(_) |
            DiscordError::UserNotFound(_) |
            DiscordError::NoTransactionHistory { .. } => ErrorCategory::Business,

            DiscordError::DatabaseConnectionError(_) |
            DiscordError::DatabaseQueryError(_) |
            DiscordError::TransactionError(_) |
            DiscordError::AccountCreationFailed(_) |
            DiscordError::MigrationError(_) |
            DiscordError::ConnectionError(_) => ErrorCategory::System,

            DiscordError::InvalidCommand(_) |
            DiscordError::UnknownCommand(_) |
            DiscordError::UnimplementedCommand(_) |
            DiscordError::ValidationError(_) |
            DiscordError::InvalidToken |
            DiscordError::ConfigError(_) => ErrorCategory::UserInput,

            DiscordError::UnauthorizedAccess { .. } |
            DiscordError::PermissionDenied(_) |
            DiscordError::ConfirmationRequired(_) |
            DiscordError::AdditionalVerificationRequired(_) |
            DiscordError::SecurityViolation(_) |
            DiscordError::RateLimited(_) |
            DiscordError::EventError(_) => ErrorCategory::Security,

            DiscordError::CommandError(_) |
            DiscordError::NetworkError { .. } => ErrorCategory::Network,
        }
    }

    /// 獲取錯誤嚴重性
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            DiscordError::InsufficientBalance(_) |
            DiscordError::InvalidAmount(_) |
            DiscordError::NoTransactionHistory { .. } |
            DiscordError::ValidationError(_) |
            DiscordError::InvalidCommand(_) |
            DiscordError::UnknownCommand(_) |
            DiscordError::UnimplementedCommand(_) => ErrorSeverity::Warning,

            DiscordError::UserNotFound(_) |
            DiscordError::AccountAlreadyExists(_) |
            DiscordError::ConfigError(_) |
            DiscordError::CommandError(_) |
            DiscordError::EventError(_) |
            DiscordError::NetworkError { .. } |
            DiscordError::ConfirmationRequired(_) |
            DiscordError::AdditionalVerificationRequired(_) => ErrorSeverity::Error,

            DiscordError::DatabaseConnectionError(_) |
            DiscordError::DatabaseQueryError(_) |
            DiscordError::TransactionError(_) |
            DiscordError::AccountCreationFailed(_) |
            DiscordError::MigrationError(_) |
            DiscordError::ConnectionError(_) |
            DiscordError::InvalidToken |
            DiscordError::UnauthorizedAccess { .. } |
            DiscordError::PermissionDenied(_) |
            DiscordError::SecurityViolation(_) |
            DiscordError::RateLimited(_) => ErrorSeverity::Critical,
        }
    }

    /// 獲取用戶友好的錯誤建議
    pub fn user_suggestion(&self) -> Option<&'static str> {
        match self {
            DiscordError::InsufficientBalance(_) => Some("請先充值或參與活動賺取更多幣值。您可以使用 !balance 查詢當前餘額。"),
            DiscordError::InvalidAmount(_) => Some("請輸入有效的正數金額。格式範例：!transfer @user 100"),
            DiscordError::UserNotFound(_) => Some("請確認用戶 ID 正確，或請對方先發送任何指令以創建帳戶。"),
            DiscordError::NoTransactionHistory { .. } => Some("您還沒有任何交易記錄。可以嘗試進行一些轉帳操作。"),
            DiscordError::InvalidCommand(_) => Some("請檢查指令格式是否正確。使用 !help 查看所有可用指令。"),
            DiscordError::UnknownCommand(_) => Some("這個指令不存在。使用 !help 查看所有可用指令。"),
            DiscordError::UnimplementedCommand(_) => Some("這個功能還在開發中，請期待未來更新。"),
            DiscordError::DatabaseConnectionError(_) => Some("系統正在維護中，請稍後再試。如果問題持續，請聯繫管理員。"),
            DiscordError::DatabaseQueryError(_) => Some("數據處理出現問題，請稍後再試。"),
            DiscordError::TransactionError(_) => Some("交易處理失敗，請稍後再試。"),
            DiscordError::AccountCreationFailed(_) => Some("帳戶創建失敗，請稍後再試或聯繫管理員。"),
            DiscordError::MigrationError(_) => Some("系統正在更新中，請稍後再試。"),
            DiscordError::ConnectionError(_) => Some("無法連接到 Discord 服務器，請檢查網路連接後再試。"),
            DiscordError::ValidationError(_) => Some("輸入的數據有問題，請檢查後重試。"),
            DiscordError::ConfigError(_) => Some("系統配置有問題，請聯繫管理員。"),
            DiscordError::CommandError(_) => Some("指令執行失敗，請稍後再試。"),
            DiscordError::EventError(_) => Some("事件處理失敗，請稍後再試。"),
            DiscordError::AccountAlreadyExists(_) => Some("您的帳戶已經存在，可以直接開始使用！"),
            DiscordError::InvalidToken => Some("系統認證有問題，請聯繫管理員。"),
            DiscordError::UnauthorizedAccess { .. } => Some("您沒有權限執行此操作，請聯繫管理員獲取幫助。"),
            DiscordError::PermissionDenied(_) => Some("您沒有權限執行此操作，請聯繫管理員獲取幫助。"),
            DiscordError::ConfirmationRequired(_) => Some("此操作需要確認，請重新執行並確認操作。"),
            DiscordError::AdditionalVerificationRequired(_) => Some("此操作需要額外驗證，請按照指示完成驗證。"),
            DiscordError::SecurityViolation(_) => Some("檢測到安全問題，操作被拒絕。如誤判，請聯繫管理員。"),
            DiscordError::RateLimited(_) => Some("操作過於頻繁，請稍後再試。"),
            DiscordError::NetworkError { .. } => Some("網路連接有問題，請檢查網路後再試。"),
        }
    }

    /// 是否為用戶錯誤（用戶可以通過修改行為解決）
    pub fn is_user_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::UserInput | ErrorCategory::Business)
    }

    /// 是否為系統錯誤（需要系統管理員處理）
    pub fn is_system_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::System | ErrorCategory::Network)
    }
}

impl From<sqlx::Error> for DiscordError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => {
                DiscordError::DatabaseQueryError(db_err.message().to_string())
            }
            sqlx::Error::PoolTimedOut => {
                DiscordError::DatabaseConnectionError("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                DiscordError::DatabaseConnectionError("Connection pool closed".to_string())
            }
            sqlx::Error::Migrate(_) => {
                DiscordError::MigrationError(format!("Migration error: {}", err))
            }
            _ => DiscordError::DatabaseQueryError(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, DiscordError>;