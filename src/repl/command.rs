use crate::app::AppEvent;

/// Commands that can be parsed from REPL input
///
/// These commands map to AppEvent for handling in the App core.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    /// Quit the application
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
/// This is the translation layer between REPL input and the App core.
pub fn command_to_app_event(command: ReplCommand) -> AppEvent {
    match command {
        ReplCommand::Quit => AppEvent::Quit,
        ReplCommand::Help => AppEvent::Help,
        ReplCommand::LoadFile(path) => AppEvent::LoadFile(path),
        ReplCommand::LoadClipboard => {
            // TODO: Implement clipboard support in future epic
            eprintln!("Clipboard input not yet supported.");
            eprintln!("Type :q to quit or :h for help.");
            AppEvent::None
        }
        ReplCommand::Unknown(input) => {
            eprintln!("Unknown command: {}", input);
            AppEvent::None
        }
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
        // For now, LoadClipboard maps to AppEvent::None with message
        assert!(matches!(event, AppEvent::None));
    }

    #[test]
    fn test_command_to_app_event_unknown() {
        let event = command_to_app_event(ReplCommand::Unknown("invalid".to_string()));
        // Unknown commands map to AppEvent::None with error message
        assert!(matches!(event, AppEvent::None));
    }
}
