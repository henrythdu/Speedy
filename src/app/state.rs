use crate::engine::timing::Token;

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

    pub fn adjust_wpm(&mut self, delta: i32) {
        const MIN_WPM: u32 = 50;
        const MAX_WPM: u32 = 1000;
        let new_wpm = self.wpm as i32 + delta;
        self.wpm = new_wpm.clamp(MIN_WPM as i32, MAX_WPM as i32) as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_readingstate_adjust_wpm_decrease() {
        let mut state = ReadingState::new(vec![], 300);
        state.adjust_wpm(-50);
        assert_eq!(state.wpm, 250);
    }

    #[test]
    fn test_readingstate_adjust_wpm_increase() {
        let mut state = ReadingState::new(vec![], 300);
        state.adjust_wpm(50);
        assert_eq!(state.wpm, 350);
    }

    #[test]
    fn test_readingstate_adjust_wpm_minimum_bound() {
        let mut state = ReadingState::new(vec![], 100);
        state.adjust_wpm(-100);
        assert!(state.wpm >= 50);
    }

    #[test]
    fn test_readingstate_adjust_wpm_maximum_bound() {
        let mut state = ReadingState::new(vec![], 1000);
        state.adjust_wpm(100);
        assert!(state.wpm <= 1000);
    }
}
