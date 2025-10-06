use droas_bot::command_router::{CommandRouter, PublicCommand as Command, PublicCommandResult as CommandResult};
use droas_bot::error::DiscordError;

#[tokio::test]
async fn test_parse_balance_command() {
    let router = CommandRouter::new();
    let result = router.parse_command("!balance").await.unwrap();
    assert_eq!(result.command, Command::Balance);
    assert_eq!(result.args.len(), 0);
}

#[tokio::test]
async fn test_parse_transfer_command_with_valid_args() {
    let router = CommandRouter::new();
    let result = router.parse_command("!transfer @user 100").await.unwrap();
    assert_eq!(result.command, Command::Transfer);
    assert_eq!(result.args.len(), 2);
    assert_eq!(result.args[0], "@user");
    assert_eq!(result.args[1], "100");
}

#[tokio::test]
async fn test_parse_history_command() {
    let router = CommandRouter::new();
    let result = router.parse_command("!history").await.unwrap();
    assert_eq!(result.command, Command::History);
    assert_eq!(result.args.len(), 0);
}

#[tokio::test]
async fn test_parse_help_command() {
    let router = CommandRouter::new();
    let result = router.parse_command("!help").await.unwrap();
    assert_eq!(result.command, Command::Help);
    assert_eq!(result.args.len(), 0);
}

#[tokio::test]
async fn test_parse_unknown_command() {
    let router = CommandRouter::new();
    let result = router.parse_command("!unknown").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::UnknownCommand(_)));
}

#[tokio::test]
async fn test_parse_empty_command() {
    let router = CommandRouter::new();
    let result = router.parse_command("").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}

#[tokio::test]
async fn test_parse_command_without_prefix() {
    let router = CommandRouter::new();
    let result = router.parse_command("balance").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}

#[tokio::test]
async fn test_route_balance_command() {
    let router = CommandRouter::new();
    let command_result = CommandResult {
        command: Command::Balance,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
    };
    let response = router.route_command(&command_result).await.unwrap();
    assert!(response.contains("balance"));
}

#[tokio::test]
async fn test_route_transfer_command() {
    let router = CommandRouter::new();
    let command_result = CommandResult {
        command: Command::Transfer,
        args: vec!["@user".to_string(), "100".to_string()],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
    };
    let response = router.route_command(&command_result).await.unwrap();
    assert!(response.contains("transfer"));
}

#[tokio::test]
async fn test_route_help_command() {
    let router = CommandRouter::new();
    let command_result = CommandResult {
        command: Command::Help,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
    };
    let response = router.route_command(&command_result).await.unwrap();
    assert!(response.contains("Available commands"));
    assert!(response.contains("balance"));
    assert!(response.contains("transfer"));
    assert!(response.contains("history"));
    assert!(response.contains("help"));
}

#[tokio::test]
async fn test_route_unimplemented_command() {
    let router = CommandRouter::new();
    let command_result = CommandResult {
        command: Command::History,
        args: vec![],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
    };
    let result = router.route_command(&command_result).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::UnimplementedCommand(_)));
}

#[tokio::test]
async fn test_route_transfer_with_insufficient_args() {
    let router = CommandRouter::new();
    let command_result = CommandResult {
        command: Command::Transfer,
        args: vec!["@user".to_string()],
        user_id: Some(12345),
        username: Some("testuser".to_string()),
    };
    let result = router.route_command(&command_result).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}

#[tokio::test]
async fn test_get_available_commands() {
    let router = CommandRouter::new();
    let commands = router.get_available_commands();
    assert_eq!(commands.len(), 4);
    assert!(commands.contains(&"balance".to_string()));
    assert!(commands.contains(&"transfer".to_string()));
    assert!(commands.contains(&"history".to_string()));
    assert!(commands.contains(&"help".to_string()));
}

#[tokio::test]
async fn test_is_command_supported() {
    let router = CommandRouter::new();
    assert!(router.is_command_supported("balance"));
    assert!(router.is_command_supported("transfer"));
    assert!(router.is_command_supported("history"));
    assert!(router.is_command_supported("help"));
    assert!(!router.is_command_supported("unknown"));
}

#[tokio::test]
async fn test_custom_prefix() {
    let router = CommandRouter::with_prefix("$".to_string());
    assert_eq!(router.get_prefix(), "$");

    let result = router.parse_command("!balance").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DiscordError::InvalidCommand(_)));
}