// Services 模組
// 包含所有業務邏輯服務

pub mod user_account_service;
pub mod admin_service;
pub mod admin_audit_service;
pub mod error_handler;
pub mod security_service;
pub mod transfer_service;
pub mod transfer_validation_service;
pub mod security_middleware;
pub mod validation_pattern;
pub mod balance_service;
pub mod message_service;
pub mod ui_components;
pub mod transaction_service;
pub mod help_service;
pub mod monitoring_service;
pub mod monitoring_config;
pub mod async_metrics_collector;
pub mod monitoring_error_handler;

// 重新導出主要服務
pub use user_account_service::{UserAccountService, AccountCreationResult};
pub use admin_service::{
    AdminService, AdminOperation, AdminOperationType, OperationResult
};
pub use admin_audit_service::{
    AdminAuditService, AdminAuditRecord, AdminAuditQuery, AdminAuditStats
};
pub use security_service::{SecurityService, AdminSecurityCheck};
pub use error_handler::ErrorHandler;
pub use crate::error::ErrorSeverity;
pub use transfer_service::{TransferService, TransferResult};
pub use transfer_validation_service::{
    TransferValidationService, ValidationResult, ValidationError, ValidationRule,
    ValidationContext, ValidationConfig, SelfTransferRule, AmountValidityRule,
    BalanceSufficiencyRule, LargeTransferLimitRule
};
pub use security_middleware::{SecurityMiddleware, SecurityValidationResult};
pub use balance_service::{BalanceService, BalanceResponse};
pub use message_service::{MessageService, MessageResponse, MessageField};
pub use ui_components::{
    UIComponentFactory, ButtonComponent, ButtonType, ButtonInteraction, ButtonLabels
};
pub use transaction_service::{TransactionService, TransactionStats};
pub use help_service::{HelpService, CommandInfo, CommandCategory};
pub use validation_pattern::{
    Validator, CompositeValidator, ValidatorFactory,
    DiscordUserIdValidator, StringInputValidator, AmountValidator,
    UsernameValidator, SelfTransferProtectionValidator,
    TransferInput, TransferInputValidator, AccountCreationInput, AccountCreationInputValidator
};
pub use monitoring_service::{
    MonitoringService, ExtendedHealthStatus,
    create_health_routes, create_metrics_routes, start_monitoring_server
};
pub use monitoring_config::{
    MonitoringConfig, AlertThresholds, ComponentMonitoringConfig
};
pub use async_metrics_collector::{
    AsyncMetricsCollector, BatchMetric, CollectorStats
};
pub use monitoring_error_handler::{
    MonitoringErrorHandler, ErrorStats, AlertState, AlertSeverity
};