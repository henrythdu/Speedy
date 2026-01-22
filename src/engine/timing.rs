// Timing engine - WPM calculation and tokenization
use unicode_segmentation::UnicodeSegmentation;

pub struct Token {
    pub text: String,
    pub duration_ms: u64,
}

pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    60_000 / wpm as u64
}

pub fn tokenize_text(text: &str) -> Vec<Token> {
    text.split_word_bounds()
        .filter(|s| {
            let trimmed = s.trim();
            !trimmed.is_empty() && !trimmed.chars().all(|c| c.is_whitespace() || c.is_control())
        })
        .map(|word| Token {
            text: word.trim().to_string(),
            duration_ms: 200,
        })
        .collect()
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
    fn test_tokenize_single_word() {
        let text = "hello";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_tokenize_multiple_words() {
        let text = "hello world";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }
}
