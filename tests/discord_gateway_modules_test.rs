use droas_bot::discord_gateway::{
    CommandParser, ServiceRouter, CommandRegistry,
    RouterErrorHandler, RouterMetrics, OperationTimer,
    Command, CommandResult
};
use droas_bot::error::DiscordError;
use std::time::Duration;

#[tokio::test]
async fn test_command_parser_basic_functionality() {
    let parser = CommandParser::new();

    // Test parsing valid commands
    let result = parser.parse_command("!balance").await.unwrap();
    assert_eq!(result.command, Command::Balance);
    assert_eq!(result.args.len(), 0);

    let result = parser.parse_command("!transfer @user 100").await.unwrap();
    assert_eq!(result.command, Command::Transfer);
    assert_eq!(result.args.len(), 2);
    assert_eq!(result.args[0], "@user");
    assert_eq!(result.args[1], "100");
}

#[tokio::test]
async fn test_command_parser_error_handling() {
    let parser = CommandParser::new();

    // Test empty command
    let result = parser.parse_command("").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));

    // Test unknown command
    let result = parser.parse_command("!unknown").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::UnknownCommand(_)));

    // Test command without prefix
    let result = parser.parse_command("balance").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}

#[test]
fn test_command_registry_functionality() {
    let mut registry = CommandRegistry::new();

    // Test initial commands
    assert!(registry.is_registered("balance"));
    assert!(registry.is_registered("transfer"));
    assert!(registry.is_registered("history"));
    assert!(registry.is_registered("help"));
    assert!(!registry.is_registered("unknown"));

    // Test adding new command
    registry.register_command(
        "test".to_string(),
        Command::Balance,
        "Test command".to_string()
    );
    assert!(registry.is_registered("test"));

    // Test getting description
    let desc = registry.get_description("balance");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Check your account balance"));

    // Test help text generation
    let help_text = registry.get_help_text();
    assert!(help_text.contains("balance"));
    assert!(help_text.contains("transfer"));
    assert!(help_text.contains("history"));
    assert!(help_text.contains("help"));
}

#[tokio::test]
async fn test_service_router_basic_routing() {
    let router = ServiceRouter::new();

    // Test balance command routing
    let command_result = CommandResult {
        command: Command::Balance,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
        guild_id: None,
        discord_context: None,
    };
    let result = router.route_command(&command_result).await.unwrap();
    assert!(result.contains("balance"));

    // Test help command routing
    let command_result = CommandResult {
        command: Command::Help,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
        guild_id: None,
        discord_context: None,
    };
    let result = router.route_command(&command_result).await.unwrap();
    assert!(result.contains("Available commands"));
    assert!(result.contains("balance"));
}

#[tokio::test]
async fn test_service_router_error_handling() {
    let router = ServiceRouter::new();

    // Test unimplemented command
    let command_result = CommandResult {
        command: Command::History,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
        guild_id: None,
        discord_context: None,
    };
    let result = router.route_command(&command_result).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::UnimplementedCommand(_)));

    // Test transfer with insufficient args
    let command_result = CommandResult {
        command: Command::Transfer,
        args: vec!["@user".to_string()],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
        guild_id: None,
        discord_context: None,
    };
    let result = router.route_command(&command_result).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}

#[test]
fn test_router_error_handler() {
    let error_handler = RouterErrorHandler::new();

    // Test error formatting
    let invalid_cmd_error = DiscordError::InvalidCommand("Test error".to_string());
    let formatted = error_handler.handle_error(&invalid_cmd_error);
    assert!(formatted.contains("❌"));
    assert!(formatted.contains("Test error"));
    assert!(formatted.contains("Type `!help`"));

    let unknown_cmd_error = DiscordError::UnknownCommand("testcmd".to_string());
    let formatted = error_handler.handle_error(&unknown_cmd_error);
    assert!(formatted.contains("❌"));
    assert!(formatted.contains("`testcmd`"));
    assert!(formatted.contains("Type `!help`"));

    let unimpl_error = DiscordError::UnimplementedCommand("Test command".to_string());
    let formatted = error_handler.handle_error(&unimpl_error);
    assert!(formatted.contains("⚠️"));
    assert!(formatted.contains("Test command"));

    // Test utility functions (these are associated functions, not methods)
    let usage = RouterErrorHandler::format_usage_info("transfer", "!transfer @user amount");
    assert!(usage.contains("💡"));
    assert!(usage.contains("!transfer @user amount"));

    let suggestion = RouterErrorHandler::format_command_suggestion("balanc", "balance");
    assert!(suggestion.contains("💡"));
    assert!(suggestion.contains("Did you mean `balance`?"));

    // Test general help message
    let help_msg = RouterErrorHandler::get_general_help_message();
    assert!(help_msg.contains("🤖"));
    assert!(help_msg.contains("DROAS Bot"));
    assert!(help_msg.contains("!balance"));
    assert!(help_msg.contains("!transfer"));
    assert!(help_msg.contains("!history"));
    assert!(help_msg.contains("!help"));
}

#[test]
fn test_router_metrics_basic_functionality() {
    let mut metrics = RouterMetrics::new();

    // Test recording command execution
    metrics.record_command_execution("balance", Duration::from_millis(50), false);
    metrics.record_command_execution("transfer", Duration::from_millis(100), false);
    metrics.record_command_execution("unknown", Duration::from_millis(25), true);

    let snapshot = metrics.get_metrics_snapshot();

    // Test snapshot data
    assert_eq!(snapshot.total_requests, 3);
    assert!((snapshot.error_rate - 0.333).abs() < 0.01); // ~33.3% error rate
    assert_eq!(snapshot.command_counts.get("balance"), Some(&1));
    assert_eq!(snapshot.command_counts.get("transfer"), Some(&1));
    assert_eq!(snapshot.command_counts.get("unknown"), Some(&1));

    // Test average response times
    assert_eq!(snapshot.average_response_times.get("balance"), Some(&Duration::from_millis(50)));
    assert_eq!(snapshot.average_response_times.get("transfer"), Some(&Duration::from_millis(100)));
    assert_eq!(snapshot.average_response_times.get("unknown"), Some(&Duration::from_millis(25)));
}

#[test]
fn test_router_metrics_sla_monitoring() {
    let mut metrics = RouterMetrics::new();
    let sla_threshold = Duration::from_millis(100);

    // Test command within SLA
    metrics.record_command_execution("fast_cmd", Duration::from_millis(50), false);
    assert!(metrics.is_within_sla("fast_cmd", sla_threshold));

    // Test command exceeding SLA
    metrics.record_command_execution("slow_cmd", Duration::from_millis(150), false);
    assert!(!metrics.is_within_sla("slow_cmd", sla_threshold));

    // Test unknown command (should be within SLA by default)
    assert!(metrics.is_within_sla("unknown_cmd", sla_threshold));
}

#[test]
fn test_operation_timer() {
    let timer = OperationTimer::new();
    std::thread::sleep(Duration::from_millis(10));
    let elapsed = timer.elapsed();

    // Timer should have elapsed at least 10ms (with some tolerance)
    assert!(elapsed >= Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(100)); // Should be much less than 100ms
}

#[test]
fn test_metrics_reset() {
    let mut metrics = RouterMetrics::new();

    // Add some data
    metrics.record_command_execution("test", Duration::from_millis(50), false);
    assert_eq!(metrics.get_metrics_snapshot().total_requests, 1);

    // Reset metrics
    metrics.reset_metrics();
    let snapshot = metrics.get_metrics_snapshot();

    assert_eq!(snapshot.total_requests, 0);
    assert_eq!(snapshot.error_rate, 0.0);
    assert!(snapshot.command_counts.is_empty());
    assert!(snapshot.average_response_times.is_empty());
}

#[tokio::test]
async fn test_command_parser_custom_prefix() {
    let parser = CommandParser::with_prefix("$".to_string());

    // Test custom prefix works
    let result = parser.parse_command("$balance").await.unwrap();
    assert_eq!(result.command, Command::Balance);

    // Test old prefix doesn't work
    let result = parser.parse_command("!balance").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));

    // Test prefix getter
    assert_eq!(parser.get_prefix(), "$");
}