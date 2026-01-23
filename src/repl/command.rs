use crate::app::AppEvent;

/// Commands that can be parsed from REPL input
///
/// These commands map to AppEvent for handling in App core.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    /// Quit to application
    Quit,

    /// Show help information
    Help,

    /// Load a file (text files only for now)
    LoadFile(String),

    /// Load from clipboard (not yet supported)
    LoadClipboard,

    /// Unknown/invalid command
    Unknown(String),
}

/// Convert a parsed REPL command into an AppEvent
///
/// This is the translation layer between REPL input and App core.
pub fn command_to_app_event(command: ReplCommand) -> AppEvent {
    match command {
        ReplCommand::Quit => AppEvent::Quit,
        ReplCommand::Help => AppEvent::Help,
        ReplCommand::LoadFile(path) => AppEvent::LoadFile(path),
        ReplCommand::LoadClipboard => AppEvent::Warning(
            "Clipboard input not yet supported. Type :q to quit or :h for help.".to_string(),
        ),
        ReplCommand::Unknown(input) => AppEvent::InvalidCommand(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_app_event_quit() {
        let event = command_to_app_event(ReplCommand::Quit);
        assert_eq!(event, AppEvent::Quit);
    }

    #[test]
    fn test_command_to_app_event_help() {
        let event = command_to_app_event(ReplCommand::Help);
        assert_eq!(event, AppEvent::Help);
    }

    #[test]
    fn test_command_to_app_event_load_file() {
        let event = command_to_app_event(ReplCommand::LoadFile("test.txt".to_string()));
        assert_eq!(event, AppEvent::LoadFile("test.txt".to_string()));
    }

    #[test]
    fn test_command_to_app_event_load_clipboard() {
        let event = command_to_app_event(ReplCommand::LoadClipboard);
        // LoadClipboard now maps to AppEvent::Warning with message
        assert!(matches!(event, AppEvent::Warning(_)));
    }

    #[test]
    fn test_command_to_app_event_unknown() {
        let event = command_to_app_event(ReplCommand::Unknown("invalid".to_string()));
        // Unknown commands now map to AppEvent::InvalidCommand
        assert!(matches!(event, AppEvent::InvalidCommand(_)));
    }
}
