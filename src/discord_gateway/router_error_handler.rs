use crate::error::DiscordError;

pub struct RouterErrorHandler;

impl RouterErrorHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_error(&self, error: &DiscordError) -> String {
        match error {
            DiscordError::InvalidCommand(msg) => {
                format!("❌ Invalid command: {}.\nType `!help` to see available commands.", msg)
            },
            DiscordError::UnknownCommand(cmd) => {
                format!("❌ Unknown command: `{}`\nType `!help` to see available commands.", cmd)
            },
            DiscordError::UnimplementedCommand(msg) => {
                format!("⚠️ Command not yet implemented: {}", msg)
            },
            DiscordError::CommandError(msg) => {
                format!("❌ Command processing failed: {}", msg)
            },
            DiscordError::ConnectionError(msg) => {
                format!("❌ Connection error: {}. Please try again later.", msg)
            },
            DiscordError::ConfigError(msg) => {
                format!("❌ Configuration error: {}. Please contact an administrator.", msg)
            },
            DiscordError::EventError(msg) => {
                format!("❌ Event handling error: {}", msg)
            },
            DiscordError::InvalidToken => {
                "❌ Authentication failed: Invalid Discord token. Please contact an administrator.".to_string()
            },
            DiscordError::DatabaseConnectionError(msg) => {
                format!("❌ Database connection error: {}. Please try again later.", msg)
            },
            DiscordError::DatabaseQueryError(msg) => {
                format!("❌ Database operation failed: {}. Please try again later.", msg)
            },
            DiscordError::TransactionError(msg) => {
                format!("❌ Transaction failed: {}. Please try again.", msg)
            },
            DiscordError::UserNotFound(msg) => {
                format!("❌ User not found: {}. User may need to register first.", msg)
            },
            DiscordError::InsufficientBalance(user_id) => {
                format!("❌ Insufficient balance for user: {}. Check your balance with `!balance`.", user_id)
            },
            DiscordError::InvalidAmount(msg) => {
                format!("❌ Invalid amount: {}. Please enter a valid positive number.", msg)
            },
            DiscordError::AccountCreationFailed(msg) => {
                format!("❌ Account creation failed: {}. Please try again later.", msg)
            },
            DiscordError::AccountAlreadyExists(user_id) => {
                format!("✅ Account already exists for user: {}. You can start using economic features.", user_id)
            },
            DiscordError::MigrationError(msg) => {
                format!("❌ System initialization error: {}. Please contact an administrator.", msg)
            },
            DiscordError::ValidationError(msg) => {
                format!("❌ Validation failed: {}", msg)
            },
            DiscordError::NoTransactionHistory { user_id, message } => {
                format!("📊 No transaction history for user {}: {}", user_id, message)
            },
            DiscordError::UnauthorizedAccess { user_id, message } => {
                format!("🔒 Unauthorized access attempt by user {}: {}", user_id, message)
            },
            DiscordError::NetworkError { message } => {
                format!("🌐 Network error: {}. Please check your connection and try again.", message)
            },
        }
    }

    pub fn format_usage_info(_command: &str, usage: &str) -> String {
        format!("💡 Usage: `{}`", usage)
    }

    pub fn format_command_suggestion(_command: &str, suggestion: &str) -> String {
        format!("💡 Did you mean `{}`?", suggestion)
    }

    pub fn get_general_help_message() -> String {
        "🤖 Welcome to DROAS Bot! 🤖\n\n\
        Available commands:\n\
        • `!balance` - Check your account balance\n\
        • `!transfer @user amount` - Transfer money to another user\n\
        • `!history` - View your transaction history\n\
        • `!help` - Show this help message\n\n\
        Need more help? Contact an administrator!"
        .to_string()
    }
}

impl Default for RouterErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}