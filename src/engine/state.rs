use crate::engine::config::TimingConfig;
use crate::engine::timing::{wpm_to_milliseconds, Token};

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

    pub fn adjust_wpm(&mut self, delta: i32) {
        let new_wpm = self.wpm as i32 + delta;
        self.wpm = new_wpm.clamp(
            *self.config.wpm_range.start() as i32,
            *self.config.wpm_range.end() as i32,
        ) as u32;
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
                punctuation: vec![],
            },
            Token {
                text: "world".to_string(),
                punctuation: vec![],
            },
        ];
        let state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.current_index, 0);
        assert_eq!(state.wpm, 300);
    }

    #[test]
    fn test_readingstate_current_token() {
        let tokens = vec![
            Token {
                text: "hello".to_string(),
                punctuation: vec![],
            },
            Token {
                text: "world".to_string(),
                punctuation: vec![],
            },
        ];
        let state = ReadingState::new_with_default_config(tokens, 300);
        assert_eq!(state.current_token().unwrap().text, "hello");
    }

    #[test]
    fn test_readingstate_advance() {
        let tokens = vec![
            Token {
                text: "hello".to_string(),
                punctuation: vec![],
            },
            Token {
                text: "world".to_string(),
                punctuation: vec![],
            },
        ];
        let mut state = ReadingState::new_with_default_config(tokens, 300);
        state.advance();
        assert_eq!(state.current_index, 1);
    }

    #[test]
    fn test_readingstate_adjust_wpm_decrease() {
        let mut state = ReadingState::new_with_default_config(vec![], 300);
        state.adjust_wpm(-50);
        assert_eq!(state.wpm, 250);
    }

    #[test]
    fn test_readingstate_adjust_wpm_increase() {
        let mut state = ReadingState::new_with_default_config(vec![], 300);
        state.adjust_wpm(50);
        assert_eq!(state.wpm, 350);
    }

    #[test]
    fn test_readingstate_adjust_wpm_minimum_bound() {
        let mut state = ReadingState::new_with_default_config(vec![], 300);
        state.adjust_wpm(-100);
        assert!(state.wpm >= 50);
    }

    #[test]
    fn test_readingstate_adjust_wpm_maximum_bound() {
        let mut state = ReadingState::new_with_default_config(vec![], 100);
        state.adjust_wpm(100);
        assert!(state.wpm <= 1000);
    }

    #[test]
    fn test_dynamic_wpm_changes_token_duration() {
        // Test that changing WPM actually changes the duration of tokens
        // This is the core bug that Speedy-0dr addresses
        let tokens = vec![Token {
            text: "hello".to_string(),
            punctuation: vec![],
        }];
        let mut state = ReadingState::new_with_default_config(tokens, 300); // 300 WPM = 200ms base

        // After fix: duration should be calculated dynamically from self.wpm
        let duration_300 = state.current_token_duration();
        assert_eq!(duration_300, 200, "300 WPM should give 200ms per word");

        // Adjust WPM to 600 (should give 100ms per word)
        state.adjust_wpm(300);
        assert_eq!(state.wpm, 600);

        // After fix: duration should now be 100ms (600 WPM)
        let duration_600 = state.current_token_duration();
        assert_eq!(duration_600, 100, "600 WPM should give 100ms per word");
    }

    #[test]
    fn test_dynamic_wpm_with_punctuation_multiplier() {
        // Test that punctuation multipliers work with dynamic WPM
        let tokens = vec![Token {
            text: "hello".to_string(),
            punctuation: vec!['.'], // 3.0x multiplier
        }];
        let mut state = ReadingState::new_with_default_config(tokens, 100); // 100 WPM = 600ms base

        // Expected: 600ms * 3.0 = 1800ms
        assert_eq!(state.current_token_duration(), 1800);

        // Increase WPM to 200 (300ms base)
        state.adjust_wpm(100);
        // Expected: 300ms * 3.0 = 900ms
        assert_eq!(state.current_token_duration(), 900);
    }
}
