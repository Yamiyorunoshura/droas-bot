// Test Modules - 測試模塊聲明
// 組織所有測試模塊，提供測試工具和公共功能

pub mod mock_repositories;

// 基本功能測試
pub mod admin_service_test;
pub mod admin_audit_service_test;
pub mod adjust_balance_command_test;
pub mod admin_security_control_test;
pub mod admin_non_functional_test;
pub mod balance_service_test;
pub mod cache_basic_test;
pub mod cache_integration_test;
pub mod command_router_test;
pub mod command_router_integration_test;
pub mod database_schema_test;
pub mod database_unit_test;
pub mod discord_gateway_modules_test;
pub mod discord_gateway_test;
pub mod error_handling_test;
pub mod help_service_test;
pub mod monitoring_service_test;
pub mod security_service_test;
pub mod transaction_repository_test;
pub mod transaction_service_test;
pub mod transaction_service_test_simple;
pub mod transfer_validation_service_test;
pub mod ui_components_test;
pub mod user_account_service_test;
pub mod automatic_member_account_creation_test;

// Cutover 修復測試
pub mod cutover_fixes_test;
pub mod cutover_fixes_simple_test;

// 集成測試
pub mod button_integration_test;
pub mod redis_cache_integration_test;

// 性能測試
pub mod performance_test;
pub mod load_test;
pub mod stability_test;
pub mod cache_performance_test;

// 重新導出測試工具
pub use mock_repositories::*;