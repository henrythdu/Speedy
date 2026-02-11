//! Duration and timing calculations module
//!
//! Handles WPM to millisecond conversions and sentence progress calculations.

use crate::reading::Token;

/// Convert WPM to milliseconds per word
pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    (60_000.0 / wpm.max(1) as f64).round() as u64
}

/// Calculate sentence progress as percentage (0.0 to 1.0)
/// Phase 2.1: Optimized to O(1) using precomputed sentence_index and sentence_length
pub fn calculate_sentence_progress(current_index: usize, tokens: &[Token]) -> f64 {
    if tokens.is_empty() || current_index >= tokens.len() {
        return 0.0;
    }

    let token = &tokens[current_index];

    // Use precomputed values for O(1) calculation (Phase 2.1 optimization)
    // Previously this required two linear scans through tokens
    let total_words = token.sentence_length;
    let words_read = token.sentence_index + 1;

    if total_words == 0 {
        return 0.0;
    }

    (words_read as f64 / total_words as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_to_milliseconds_300() {
        // 300 WPM = 200ms per word (60,000 / 300 = 200)
        let result = wpm_to_milliseconds(300);
        assert_eq!(result, 200);
    }

    #[test]
    fn test_wpm_to_milliseconds_600() {
        // 600 WPM = 100ms per word (60,000 / 600 = 100)
        let result = wpm_to_milliseconds(600);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_wpm_to_milliseconds_precision_165() {
        // 165 WPM = 363.636... ms per word
        // Should round to 364, not truncate to 363
        let result = wpm_to_milliseconds(165);
        assert_eq!(result, 364);
    }

    #[test]
    fn test_calculate_sentence_progress_basic() {
        let tokens = vec![
            Token {
                text: "First".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
                char_count: 5,
                punctuation_multiplier: 1.0,
                sentence_index: 0,
                sentence_length: 3,
            },
            Token {
                text: "second".to_string(),
                punctuation: vec![],
                is_sentence_start: false,
                char_count: 6,
                punctuation_multiplier: 1.0,
                sentence_index: 1,
                sentence_length: 3,
            },
            Token {
                text: "third".to_string(),
                punctuation: vec!['.'],
                is_sentence_start: false,
                char_count: 5,
                punctuation_multiplier: 3.0,
                sentence_index: 2,
                sentence_length: 3,
            },
        ];

        assert_eq!(calculate_sentence_progress(0, &tokens), 1.0 / 3.0);
        assert_eq!(calculate_sentence_progress(1, &tokens), 2.0 / 3.0);
        assert_eq!(calculate_sentence_progress(2, &tokens), 3.0 / 3.0);
    }

    #[test]
    fn test_calculate_sentence_progress_empty() {
        let tokens: Vec<Token> = vec![];
        assert_eq!(calculate_sentence_progress(0, &tokens), 0.0);
    }

    #[test]
    fn test_calculate_sentence_progress_out_of_bounds() {
        let tokens = vec![Token {
            text: "Only".to_string(),
            punctuation: vec![],
            is_sentence_start: true,
            char_count: 4,
            punctuation_multiplier: 1.0,
            sentence_index: 0,
            sentence_length: 1,
        }];
        assert_eq!(calculate_sentence_progress(5, &tokens), 0.0);
    }
}
