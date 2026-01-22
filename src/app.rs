// App module - contains AppMode state machine
use crate::engine::timing::Token;

pub enum AppMode {
    Reading,
    Paused,
    Repl,
    Peek,
    Quit,
}

pub struct ReadingState {
    pub tokens: Vec<Token>,
    pub current_index: usize,
    pub wpm: u32,
}

impl ReadingState {
    pub fn new(tokens: Vec<Token>, wpm: u32) -> Self {
        Self {
            tokens,
            current_index: 0,
            wpm,
        }
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current_index)
    }

    pub fn advance(&mut self) {
        if self.current_index < self.tokens.len().saturating_sub(1) {
            self.current_index += 1;
        }
    }
}

pub struct App {
    pub mode: AppMode,
    pub reading_state: Option<ReadingState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Repl,
            reading_state: None,
        }
    }

    pub fn start_reading(&mut self, text: &str, wpm: u32) {
        use crate::engine::timing::tokenize_text;
        let tokens = tokenize_text(text);
        self.reading_state = Some(ReadingState::new(tokens, wpm));
        self.mode = AppMode::Reading;
    }

    pub fn toggle_pause(&mut self) {
        match self.mode {
            AppMode::Reading => {
                self.mode = AppMode::Paused;
            }
            AppMode::Paused => {
                self.mode = AppMode::Reading;
            }
            _ => {}
        }
    }
}

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
        assert_eq!(state.current_token().unwrap().text, "hello");
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
