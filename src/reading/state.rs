use crate::engine::config::TimingConfig;
use crate::reading::{wpm_to_milliseconds, Token};

pub struct ReadingState {
    tokens: Vec<Token>,
    current_index: usize,
    wpm: u32,
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

    /// Get the current reading position index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the current words-per-minute setting
    pub fn wpm(&self) -> u32 {
        self.wpm
    }

    /// Get a reference to the tokens vector
    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
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

    pub fn adjust_wpm(&mut self, delta: i32) {
        let new_wpm = self.wpm as i32 + delta;
        self.wpm = new_wpm.clamp(
            *self.config.wpm_range().start() as i32,
            *self.config.wpm_range().end() as i32,
        ) as u32;
    }

    fn calculate_token_duration(&self, token: &Token) -> u64 {
        let base_delay_ms = wpm_to_milliseconds(self.wpm);

        // PRD Section 3.2: Punctuation Multipliers with max stacking rule
        let punctuation_multiplier = if token.punctuation().is_empty() {
            1.0
        } else {
            token
                .punctuation()
                .iter()
                .map(|&p| match p {
                    '.' => self.config.period_multiplier(),
                    '?' => self.config.question_multiplier(),
                    '!' => self.config.exclamation_multiplier(),
                    ',' => self.config.comma_multiplier(),
                    '\n' => self.config.newline_multiplier(),
                    _ => 1.0,
                })
                .fold(1.0, f64::max)
        };

        // PRD Section 3.2: Word Length Penalty - take MAX with punctuation, NOT multiply
        // Use precomputed char_count for O(1) instead of O(n) chars().count()
        let word_length = token.char_count();
        let length_penalty = if word_length > self.config.long_word_threshold() {
            self.config.long_word_penalty()
        } else {
            1.0
        };

        // PRD: Apply MAX of punctuation and length penalty, NOT multiply them
        // Example: 300 WPM = 200ms, period (3.0x), length >10 (1.15x)
        // Wrong: 200 * 3.0 * 1.15 = 690ms
        // Correct: 200 * max(3.0, 1.15) = 200 * 3.0 = 600ms
        let combined_multiplier = punctuation_multiplier.max(length_penalty);
        (base_delay_ms as f64 * combined_multiplier).round() as u64
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
            .position(|token| token.is_sentence_start())
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
            .find(|(_, token)| token.is_sentence_start())
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
        let char_count = text.chars().count();
        Token::new(
            text.to_string(),
            vec![],
            is_sentence_start,
            char_count,
            1.0,
            0,
            1,
        )
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
        assert_eq!(state.current_token().unwrap().text(), "hello");
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
        // Create a token with a long word (> 10 chars), no punctuation
        let tokens = vec![Token::new(
            "extraordinarily".to_string(),
            vec![],
            true,
            15,
            1.0,
            0,
            1,
        )];
        let state = ReadingState::new_with_default_config(tokens, 300);
        // 300 WPM = 200ms per word * max(1.0, 1.15) = 230ms
        // PRD: Apply MAX of punctuation (1.0) and length penalty (1.15)
        assert_eq!(state.current_token_duration(), 230);
    }

    #[test]
    fn test_current_token_duration_with_punctuation() {
        let tokens = vec![Token::new(
            "hello".to_string(),
            vec!['.'],
            true,
            5,
            2.0,
            0,
            1,
        )];
        let state = ReadingState::new_with_default_config(tokens, 300);
        // 300 WPM = 200ms per word * max(3.0, 1.0) = 600ms
        // PRD: Apply MAX of punctuation (3.0) and length penalty (1.0)
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
}
