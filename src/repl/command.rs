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

    /// Load a file (PDF, EPUB supported)
    LoadFile(String),

    /// Load from clipboard
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
        ReplCommand::LoadClipboard => AppEvent::LoadClipboard,
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
        assert_eq!(event, AppEvent::LoadClipboard);
    }

    #[test]
    fn test_command_to_app_event_unknown() {
        let event = command_to_app_event(ReplCommand::Unknown("invalid".to_string()));
        // Unknown commands now map to AppEvent::InvalidCommand
        assert!(matches!(event, AppEvent::InvalidCommand(_)));
    }
}
