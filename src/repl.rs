// REPL module - command parsing and interface

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    LoadFile(String),
    LoadClipboard,
    Quit,
}

pub fn parse_repl_input(input: &str) -> ReplCommand {
    unimplemented!("parse_repl_input to be implemented")
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
}
