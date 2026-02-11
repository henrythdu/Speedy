use crate::reading::Token;

/// Calculate sentence progress as percentage (0.0 to 1.0)
///
/// Uses precomputed Token fields (sentence_index, sentence_length) for O(1) performance
/// instead of O(n) scans through tokens.
pub fn calculate_sentence_progress(current_index: usize, tokens: &[Token]) -> f64 {
    if tokens.is_empty() || current_index >= tokens.len() {
        return 0.0;
    }

    // Use precomputed sentence_index and sentence_length for O(1) calculation
    let token = &tokens[current_index];
    let sentence_index = token.sentence_index();
    let sentence_length = token.sentence_length();

    if sentence_length == 0 {
        return 0.0;
    }

    // sentence_index is 0-based, so words_read = sentence_index + 1
    let words_read = sentence_index + 1;
    (words_read as f64 / sentence_length as f64).clamp(0.0, 1.0)
}

/// Convert WPM to milliseconds per word
pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    (60_000.0 / wpm.max(1) as f64).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_to_milliseconds_precision_350() {
        // 350 WPM = 171.428... ms per word
        // PRD Section 3.2: Must use floating-point precision, not integer truncation
        // 60,000 / 350 = 171.428... → should round to 171
        let result = wpm_to_milliseconds(350);
        assert_eq!(result, 171);
    }

    #[test]
    fn test_wpm_to_milliseconds_precision_333() {
        // 333 WPM = 180.18... ms per word
        // 60,000 / 333 = 180.18... → should round to 180
        let result = wpm_to_milliseconds(333);
        assert_eq!(result, 180);
    }

    #[test]
    fn test_wpm_to_milliseconds_precision_165() {
        // 165 WPM = 363.636... ms per word
        // PRD Section 3.2: Must use rounding, not integer truncation
        // 60,000 / 165 = 363.636... → should round to 364
        // Integer truncation gives 363 (BUG), correct rounding gives 364
        let result = wpm_to_milliseconds(165);
        assert_eq!(result, 364);
    }

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
}
