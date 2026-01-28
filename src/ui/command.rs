//! Command parsing for TUI command deck
//!
//! Parses user input in Command mode, supporting:
//! - `:q` or `:quit` → Quit command
//! - `:h` or `:help` → Help command
//! - `@filename.pdf` or `@filename.epub` → Load file command
//! - `@@` → Load clipboard
//!
//! ## Migration from REPL
//!
//! This module replaces the obsolete `repl::parser` and `repl::command` modules.
//! The parsing logic is identical but designed for integration with the TUI
//! command deck instead of a separate REPL loop.

use crate::app::AppEvent;

/// Commands that can be parsed from command deck input
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Quit,
    Help,
    LoadFile(String),
    LoadClipboard,
    Unknown(String),
}

/// Parse command deck input string into a Command
///
/// Supports:
/// - `:q` or `:quit` → Quit command
/// - `:h` or `:help` → Help command
/// - `@filename.pdf` or `@filename.epub` → Load file command
/// - `@@` → Load clipboard
/// - Unknown command → Error message
pub fn parse_command(input: &str) -> Command {
    let input = input.trim();

    // Check for empty input first
    if input.is_empty() {
        return Command::Unknown(input.to_string());
    }

    // Handle system commands starting with ':'
    if let Some(cmd) = input.strip_prefix(':') {
        match cmd {
            "q" | "quit" => Command::Quit,
            "h" | "help" => Command::Help,
            _ => Command::Unknown(input.to_string()),
        }
    } else if let Some(rest) = input.strip_prefix('@') {
        let filename = rest.trim();
        if filename.is_empty() || filename == "@" {
            Command::LoadClipboard
        } else {
            Command::LoadFile(filename.to_string())
        }
    } else {
        // Unknown command pattern
        Command::Unknown(input.to_string())
    }
}

/// Convert a parsed command into an AppEvent
///
/// This is the translation layer between command deck input and App core.
pub fn command_to_app_event(command: Command) -> AppEvent {
    match command {
        Command::Quit => AppEvent::Quit,
        Command::Help => AppEvent::Help,
        Command::LoadFile(path) => AppEvent::LoadFile(path),
        Command::LoadClipboard => AppEvent::LoadClipboard,
        Command::Unknown(input) => AppEvent::InvalidCommand(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quit_variants() {
        assert_eq!(parse_command(":q"), Command::Quit);
        assert_eq!(parse_command(":quit"), Command::Quit);
    }

    #[test]
    fn test_parse_help_variants() {
        assert_eq!(parse_command(":h"), Command::Help);
        assert_eq!(parse_command(":help"), Command::Help);
    }

    #[test]
    fn test_parse_load_file() {
        assert_eq!(
            parse_command("@test.txt"),
            Command::LoadFile("test.txt".to_string())
        );
    }

    #[test]
    fn test_parse_load_file_with_spaces() {
        assert_eq!(
            parse_command("@  test.txt"),
            Command::LoadFile("test.txt".to_string())
        );
    }

    #[test]
    fn test_parse_load_clipboard() {
        assert_eq!(parse_command("@@"), Command::LoadClipboard);
    }

    #[test]
    fn test_parse_empty_input() {
        assert!(matches!(parse_command(""), Command::Unknown(_)));
    }

    #[test]
    fn test_parse_invalid_command() {
        assert!(matches!(parse_command("invalid"), Command::Unknown(_)));
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(matches!(parse_command("   "), Command::Unknown(_)));
    }

    #[test]
    fn test_command_to_app_event_quit() {
        let event = command_to_app_event(Command::Quit);
        assert_eq!(event, AppEvent::Quit);
    }

    #[test]
    fn test_command_to_app_event_help() {
        let event = command_to_app_event(Command::Help);
        assert_eq!(event, AppEvent::Help);
    }

    #[test]
    fn test_command_to_app_event_load_file() {
        let event = command_to_app_event(Command::LoadFile("test.txt".to_string()));
        assert_eq!(event, AppEvent::LoadFile("test.txt".to_string()));
    }

    #[test]
    fn test_command_to_app_event_load_clipboard() {
        let event = command_to_app_event(Command::LoadClipboard);
        assert_eq!(event, AppEvent::LoadClipboard);
    }

    #[test]
    fn test_command_to_app_event_unknown() {
        let event = command_to_app_event(Command::Unknown("invalid".to_string()));
        assert!(matches!(event, AppEvent::InvalidCommand(_)));
    }
}
