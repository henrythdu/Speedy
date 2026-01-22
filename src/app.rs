// App module - contains AppMode state machine
use crate::engine::timing::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appmode_enum_exists() {
        let _mode = AppMode::Reading;
        let _mode = AppMode::Paused;
        let _mode = AppMode::Repl;
        let _mode = AppMode::Peek;
        let _mode = AppMode::Quit;
    }

    #[test]
    fn test_readingstate_initialization() {
        let tokens = vec![
            Token {
                text: "hello".to_string(),
                duration_ms: 200,
            },
            Token {
                text: "world".to_string(),
                duration_ms: 200,
            },
        ];
        let state = ReadingState::new(tokens, 300);
        assert_eq!(state.current_index, 0);
        assert_eq!(state.wpm, 300);
    }

    #[test]
    fn test_readingstate_current_token() {
        let tokens = vec![
            Token {
                text: "hello".to_string(),
                duration_ms: 200,
            },
            Token {
                text: "world".to_string(),
                duration_ms: 200,
            },
        ];
        let state = ReadingState::new(tokens, 300);
        assert_eq!(state.current_token().text, "hello");
    }

    #[test]
    fn test_readingstate_advance() {
        let tokens = vec![
            Token {
                text: "hello".to_string(),
                duration_ms: 200,
            },
            Token {
                text: "world".to_string(),
                duration_ms: 200,
            },
        ];
        let mut state = ReadingState::new(tokens, 300);
        state.advance();
        assert_eq!(state.current_index, 1);
    }
}
