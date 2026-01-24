// Timing engine - WPM calculation and tokenization
const DEFAULT_WPM: u32 = 300;

pub struct Token {
    pub text: String,
    pub duration_ms: u64,
    /// Trailing punctuation (., ?, !, or ,) if detected at word end; None otherwise.
    pub punctuation: Option<char>,
}

fn extract_punctuation(word: &str) -> (String, Option<char>) {
    if word.is_empty() {
        return (String::new(), None);
    }

    let mut chars = word.chars();
    if let Some(last_char) = chars.next_back() {
        if is_sentence_terminator(last_char) || is_comma(last_char) {
            return (chars.collect(), Some(last_char));
        }
    }
    (word.to_string(), None)
}

fn is_sentence_terminator(c: char) -> bool {
    c == '.' || c == '?' || c == '!'
}

fn is_comma(c: char) -> bool {
    c == ','
}

pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    60_000 / wpm.max(1) as u64
}

/// Tokenizes text on whitespace; extracts trailing punctuation as metadata.
pub fn tokenize_text(text: &str, wpm: u32) -> Vec<Token> {
    let duration_ms = wpm_to_milliseconds(wpm);
    text.split_whitespace()
        .map(|word| {
            let (text, punctuation) = extract_punctuation(word);
            Token {
                text,
                duration_ms,
                punctuation,
            }
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
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_tokenize_multiple_words() {
        let text = "hello world";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    #[test]
    fn test_tokenize_with_wpm_300() {
        let text = "hello world";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].duration_ms, 200);
        assert_eq!(tokens[1].duration_ms, 200);
    }

    #[test]
    fn test_tokenize_with_wpm_600() {
        let text = "hello world";
        let tokens = tokenize_text(text, 600);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].duration_ms, 100);
        assert_eq!(tokens[1].duration_ms, 100);
    }

    #[test]
    fn test_tokenize_with_period() {
        let text = "hello world.";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, None);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, Some('.'));
    }

    #[test]
    fn test_tokenize_with_question_mark() {
        let text = "hello world?";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, None);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, Some('?'));
    }

    #[test]
    fn test_tokenize_with_exclamation() {
        let text = "hello world!";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, None);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, Some('!'));
    }

    #[test]
    fn test_tokenize_newline() {
        let text = "hello\nworld";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, None);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, None);
    }

    #[test]
    fn test_tokenize_with_comma() {
        let text = "hello, world";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, Some(','));
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, None);
    }
}
