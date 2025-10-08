use std::collections::HashMap;
use super::command_parser::Command;

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    descriptions: HashMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        let mut descriptions = HashMap::new();

        // Register commands
        commands.insert("balance".to_string(), Command::Balance);
        commands.insert("transfer".to_string(), Command::Transfer);
        commands.insert("history".to_string(), Command::History);
        commands.insert("help".to_string(), Command::Help);
        commands.insert("sync_members".to_string(), Command::SyncMembers);
        commands.insert("sync_member".to_string(), Command::SyncMembers);

        // Add descriptions
        descriptions.insert("balance".to_string(), "Check your account balance".to_string());
        descriptions.insert("transfer".to_string(), "Transfer money to another user: !transfer @user amount".to_string());
        descriptions.insert("history".to_string(), "View your transaction history".to_string());
        descriptions.insert("help".to_string(), "Show this help message".to_string());
        descriptions.insert("sync_members".to_string(), "Sync all server members (Admin only): !sync_members".to_string());
        descriptions.insert("sync_member".to_string(), "Sync all server members (Admin only): !sync_member".to_string());

        Self { commands, descriptions }
    }

    pub fn register_command(&mut self, name: String, command: Command, description: String) {
        self.commands.insert(name.clone(), command);
        self.descriptions.insert(name, description);
    }

    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    pub fn get_description(&self, name: &str) -> Option<&String> {
        self.descriptions.get(name)
    }

    pub fn list_commands(&self) -> Vec<&String> {
        self.commands.keys().collect()
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn get_help_text(&self) -> String {
        let mut help_text = String::from("Available commands:\n");

        for command_name in self.list_commands() {
            if let Some(description) = self.get_description(command_name) {
                help_text.push_str(&format!("• {} - {}\n", command_name, description));
            }
        }

        help_text
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}