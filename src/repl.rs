// REPL module - command parsing and interface

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    LoadFile(String),
    LoadClipboard,
    Quit,
    Help,
}

pub fn parse_repl_input(input: &str) -> ReplCommand {
    let trimmed = input.trim();

    if trimmed.starts_with("@@") {
        return ReplCommand::LoadClipboard;
    }

    if trimmed.starts_with('@') && trimmed.len() > 1 {
        return ReplCommand::LoadFile(trimmed[1..].to_string());
    }

    if trimmed == ":q" || trimmed == ":quit" {
        return ReplCommand::Quit;
    }

    if trimmed == ":h" || trimmed == ":help" {
        return ReplCommand::Help;
    }

    ReplCommand::LoadFile(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repl_input_load_file() {
        let input = "@test.txt";
        let command = parse_repl_input(input);
        assert_eq!(command, ReplCommand::LoadFile("test.txt".to_string()));
    }

    #[test]
    fn test_parse_repl_input_load_clipboard() {
        let input = "@@";
        let command = parse_repl_input(input);
        assert_eq!(command, ReplCommand::LoadClipboard);
    }

    #[test]
    fn test_parse_repl_input_quit() {
        let input = ":q";
        let command = parse_repl_input(input);
        assert_eq!(command, ReplCommand::Quit);
    }

    #[test]
    fn test_parse_repl_input_help() {
        let input = ":h";
        let command = parse_repl_input(input);
        assert_eq!(command, ReplCommand::Help);
    }
}
