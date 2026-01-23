use super::ReplCommand;

/// Parse REPL input string into a command
///
/// Supports:
/// - `:q` or `:quit` → Quit command
/// - `:h` or `:help` → Help command
/// - `@filename` → Load file command
/// - Unknown command → Error message
pub fn parse_repl_input(input: &str) -> ReplCommand {
    let input = input.trim();

    // Check for empty input first
    if input.is_empty() {
        return ReplCommand::Unknown(input.to_string());
    }

    // Handle system commands starting with ':'
    if let Some(cmd) = input.strip_prefix(':') {
        match cmd {
            "q" | "quit" => ReplCommand::Quit,
            "h" | "help" => ReplCommand::Help,
            _ => ReplCommand::Unknown(input.to_string()),
        }
    } else if let Some(rest) = input.strip_prefix('@') {
        let filename = rest.trim();
        if filename.is_empty() || filename == "@" {
            ReplCommand::LoadClipboard
        } else {
            ReplCommand::LoadFile(filename.to_string())
        }
    } else {
        // Unknown command pattern
        ReplCommand::Unknown(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quit_variants() {
        assert_eq!(parse_repl_input(":q"), ReplCommand::Quit);
        assert_eq!(parse_repl_input(":quit"), ReplCommand::Quit);
    }

    #[test]
    fn test_parse_help_variants() {
        assert_eq!(parse_repl_input(":h"), ReplCommand::Help);
        assert_eq!(parse_repl_input(":help"), ReplCommand::Help);
    }

    #[test]
    fn test_parse_load_file() {
        assert_eq!(
            parse_repl_input("@test.txt"),
            ReplCommand::LoadFile("test.txt".to_string())
        );
    }

    #[test]
    fn test_parse_load_file_with_spaces() {
        assert_eq!(
            parse_repl_input("@  test.txt"),
            ReplCommand::LoadFile("test.txt".to_string())
        );
    }

    #[test]
    fn test_parse_load_clipboard() {
        assert_eq!(parse_repl_input("@@"), ReplCommand::LoadClipboard);
    }

    #[test]
    fn test_parse_empty_input() {
        assert!(matches!(parse_repl_input(""), ReplCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_invalid_command() {
        assert!(matches!(
            parse_repl_input("invalid"),
            ReplCommand::Unknown(_)
        ));
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(matches!(parse_repl_input("   "), ReplCommand::Unknown(_)));
    }
}
