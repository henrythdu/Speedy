use crate::engine::config::TimingConfig;
use crate::engine::{wpm_to_milliseconds, Token};

pub struct ReadingState {
    pub tokens: Vec<Token>,
    pub current_index: usize,
    pub wpm: u32,
    config: TimingConfig,
}

impl ReadingState {
    pub fn new(tokens: Vec<Token>, wpm: u32, config: TimingConfig) -> Self {
        Self {
            tokens,
            current_index: 0,
            wpm,
            config,
        }
    }

    pub fn new_with_default_config(tokens: Vec<Token>, wpm: u32) -> Self {
        Self::new(tokens, wpm, TimingConfig::default())
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current_index)
    }

    pub fn current_token_duration(&self) -> u64 {
        match self.current_token() {
            Some(token) => self.calculate_token_duration(token),
            None => 0,
        }
    }

    /// Returns the current WPM setting for timeout calculation.
    ///
    /// Used by TuiManager to calculate timing tick interval.
    pub fn get_wpm(&self) -> u32 {
        self.wpm
    }

    pub fn adjust_wpm(&mut self, delta: i32) {
        let new_wpm = self.wpm as i32 + delta;
        self.wpm = new_wpm.clamp(
            *self.config.wpm_range.start() as i32,
            *self.config.wpm_range.end() as i32,
        ) as u32;
    }

    fn calculate_token_duration(&self, token: &Token) -> u64 {
        let base_delay_ms = wpm_to_milliseconds(self.wpm);

        // PRD Section 3.2: Punctuation Multipliers with max stacking rule
        let punctuation_multiplier = if token.punctuation.is_empty() {
            1.0
        } else {
            token
                .punctuation
                .iter()
                .map(|&p| match p {
                    '.' => self.config.period_multiplier,
                    '?' => self.config.question_multiplier,
                    '!' => self.config.exclamation_multiplier,
                    ',' => self.config.comma_multiplier,
                    '\n' => self.config.newline_multiplier,
                    _ => 1.0,
                })
                .fold(1.0, f64::max)
        };

        // PRD Section 3.2: Word Length Penalty
        let word_length = token.text.chars().count();
        let length_penalty = if word_length > self.config.long_word_threshold {
            self.config.long_word_penalty
        } else {
            1.0
        };

        (base_delay_ms as f64 * punctuation_multiplier * length_penalty) as u64
    }

    pub fn advance(&mut self) {
        if self.current_index < self.tokens.len().saturating_sub(1) {
            self.current_index += 1;
        }
    }

    pub fn find_next_sentence_start(&self) -> Option<usize> {
        let start = self.current_index.saturating_add(1);
        if start >= self.tokens.len() {
            return None;
        }
        self.tokens[start..]
            .iter()
            .position(|token| token.is_sentence_start)
            .map(|pos| pos + start)
    }

    pub fn jump_to_next_sentence(&mut self) -> bool {
        match self.find_next_sentence_start() {
            Some(index) => {
                self.current_index = index;
                true
            }
            None => false,
        }
    }

    pub fn find_previous_sentence_start(&self) -> Option<usize> {
        if self.current_index == 0 {
            return None;
        }

        // Search backwards from current_index - 1
        let end = self.current_index;
        self.tokens[..end]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, token)| token.is_sentence_start)
            .map(|(idx, _)| idx)
    }

    pub fn jump_to_previous_sentence(&mut self) -> bool {
        match self.find_previous_sentence_start() {
            Some(index) => {
                self.current_index = index;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token(text: &str, is_sentence_start: bool) -> Token {
        Token {
            text: text.to_string(),
            punctuation: vec![],
            is_sentence_start,
        }
    }

    #[test]
    fn test_find_next_sentence_start() {
        let tokens = vec![
            create_test_token("First", true),
            create_test_token("sentence", false),
            create_test_token("Second", true),
            create_test_token("sentence", false),
        ];
        let state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.find_next_sentence_start(), Some(2));
    }

    #[test]
    fn test_find_next_sentence_start_none() {
        let tokens = vec![
            create_test_token("Only", true),
            create_test_token("sentence", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.current_index = 1; // At "sentence"
        assert_eq!(state.find_next_sentence_start(), None);
    }

    #[test]
    fn test_jump_to_next_sentence() {
        let tokens = vec![
            create_test_token("First", true),
            create_test_token("sentence", false),
            create_test_token("Second", true),
            create_test_token("sentence", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        assert!(state.jump_to_next_sentence());
        assert_eq!(state.current_index, 2);
    }

    #[test]
    fn test_find_previous_sentence_start() {
        let tokens = vec![
            create_test_token("First", true),
            create_test_token("sentence", false),
            create_test_token("Second", true),
            create_test_token("here", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.current_index = 3; // At "here"
        assert_eq!(state.find_previous_sentence_start(), Some(2));
    }

    #[test]
    fn test_find_previous_sentence_start_none() {
        let tokens = vec![
            create_test_token("First", true),
            create_test_token("sentence", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.current_index = 0; // At start
        assert_eq!(state.find_previous_sentence_start(), None);
    }

    #[test]
    fn test_jump_to_previous_sentence() {
        let tokens = vec![
            create_test_token("First", true),
            create_test_token("sentence", false),
            create_test_token("Second", true),
            create_test_token("here", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.current_index = 3; // At "here"
        assert!(state.jump_to_previous_sentence());
        assert_eq!(state.current_index, 2);
    }

    #[test]
    fn test_current_token() {
        let tokens = vec![
            create_test_token("hello", true),
            create_test_token("world", false),
        ];
        let state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.current_token().unwrap().text, "hello");
    }

    #[test]
    fn test_current_token_duration() {
        let tokens = vec![create_test_token("hello", true)];
        let state = ReadingState::new_with_default_config(tokens, 300);
        // 300 WPM = 200ms per word
        assert_eq!(state.current_token_duration(), 200);
    }

    #[test]
    fn test_current_token_duration_long_word() {
        // Create a token with a long word (> 10 chars)
        let tokens = vec![Token {
            text: "extraordinarily".to_string(),
            punctuation: vec![],
            is_sentence_start: true,
        }];
        let state = ReadingState::new_with_default_config(tokens, 300);
        // 300 WPM = 200ms per word * 1.15 (long word penalty) = 229ms (rounded)
        assert_eq!(state.current_token_duration(), 229);
    }

    #[test]
    fn test_current_token_duration_with_punctuation() {
        let tokens = vec![Token {
            text: "hello".to_string(),
            punctuation: vec!['.'],
            is_sentence_start: true,
        }];
        let state = ReadingState::new_with_default_config(tokens, 300);
        // 300 WPM = 200ms per word * 3.0 (period multiplier) = 600ms
        assert_eq!(state.current_token_duration(), 600);
    }

    #[test]
    fn test_adjust_wpm() {
        let tokens = vec![create_test_token("test", true)];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.adjust_wpm(50);
        assert_eq!(state.wpm, 350);
    }

    #[test]
    fn test_adjust_wpm_clamp_min() {
        let tokens = vec![create_test_token("test", true)];
        let mut state = ReadingState::new_with_default_config(tokens, 100);
        state.adjust_wpm(-200);
        assert_eq!(state.wpm, 50); // Should clamp to minimum 50
    }

    #[test]
    fn test_adjust_wpm_clamp_max() {
        let tokens = vec![create_test_token("test", true)];
        let mut state = ReadingState::new_with_default_config(tokens, 1000);
        state.adjust_wpm(500);
        assert_eq!(state.wpm, 1000); // Should clamp to maximum 1000
    }

    #[test]
    fn test_advance() {
        let tokens = vec![
            create_test_token("hello", true),
            create_test_token("world", false),
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.current_index, 0);
        state.advance();
        assert_eq!(state.current_index, 1);
    }

    #[test]
    fn test_advance_at_end() {
        let tokens = vec![create_test_token("hello", true)];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.advance();
        // At end, should stay at 0 (can't go past end)
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_new_with_default_config() {
        let tokens = vec![create_test_token("test", true)];
        let state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.wpm, 300);
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_get_wpm() {
        let tokens = vec![create_test_token("test", true)];
        let state = ReadingState::new_with_default_config(tokens, 450);
        assert_eq!(state.get_wpm(), 450);
    }
}
