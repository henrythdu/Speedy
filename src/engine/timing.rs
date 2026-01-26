// Timing engine - WPM calculation and tokenization

pub struct Token {
    pub text: String,
    /// Trailing punctuation characters (e.g., ['?', '!'] for "word?!") per PRD Section 3.2 max stacking rule.
    pub punctuation: Vec<char>,
}

fn extract_punctuation(word: &str) -> (String, Vec<char>) {
    if word.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut chars: Vec<char> = word.chars().collect();
    let mut punctuation_chars = Vec::new();

    // Collect all trailing punctuation characters
    while let Some(&last_char) = chars.last() {
        if is_sentence_terminator(last_char) || is_comma(last_char) {
            punctuation_chars.push(chars.pop().unwrap());
        } else {
            break;
        }
    }

    // Reverse to maintain original order
    punctuation_chars.reverse();

    (chars.into_iter().collect(), punctuation_chars)
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

/// Tokenizes text line-by-line; PRD Section 3.2.
/// Note: duration is calculated dynamically in ReadingState, not stored in Token.
pub fn tokenize_text(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in text.lines() {
        // Process words in current line
        for word in line.split_whitespace() {
            if !word.is_empty() {
                let (text, punctuation) = extract_punctuation(word);

                tokens.push(Token { text, punctuation });
            }
        }

        // Create newline token after each line (except if line was empty/whitespace only)
        tokens.push(Token {
            text: String::new(),
            punctuation: vec!['\n'],
        });
    }

    // Remove trailing newline token if it exists (last line doesn't need newline after it)
    if tokens
        .last()
        .map_or(false, |t| t.punctuation == vec!['\n'] && t.text.is_empty())
    {
        tokens.pop();
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::config::TimingConfig;

    #[test]
    fn test_stacked_punctuation_question_exclamation() {
        // PRD Section 3.2: Max stacking rule - "word?!" should have both ? and !
        let text = "hello?!";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec!['?', '!']);
    }

    #[test]
    fn test_stacked_punctuation_multiple_periods() {
        // PRD Section 3.2: Max stacking rule - "word..." should have all three periods
        let text = "wait...";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "wait");
        assert_eq!(tokens[0].punctuation, vec!['.', '.', '.']);
    }

    #[test]
    fn test_stacked_punctuation_comma_period() {
        // PRD Section 3.2: Max stacking rule - "word.," should have both . and ,
        let text = "hello.,";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec!['.', ',']);
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

    #[test]
    fn test_tokenize_single_word() {
        let text = "hello";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_tokenize_multiple_words() {
        let text = "hello world";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    #[test]
    fn test_tokenize_with_wpm_300() {
        let text = "hello world";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_with_wpm_600() {
        let text = "hello world";
        let mut config = TimingConfig::default();
        config.wpm = 600;
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_with_period() {
        let text = "hello world.";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, vec!['.']);
    }

    #[test]
    fn test_tokenize_with_exclamation() {
        let text = "hello world!";
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].punctuation, vec!['!']);
    }

    #[test]
    fn test_tokenize_long_word() {
        let text = "extraordinarily"; // 14 chars > 10
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "extraordinarily");
        assert_eq!(tokens[0].punctuation, vec![]);
    }

    #[test]
    fn test_tokenize_short_word() {
        let text = "hello"; // 5 chars <= 10
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
    }

    #[test]
    fn test_tokenize_newline() {
        let text = "hello\nworld"; // Two words separated by newline
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
        assert_eq!(tokens[1].text, ""); // Newline token
        assert_eq!(tokens[1].punctuation, vec!['\n']); // Newline as punctuation
        assert_eq!(tokens[2].text, "world");
        assert_eq!(tokens[2].punctuation, vec![]);
    }

    #[test]
    fn test_tokenize_long_word_with_punctuation() {
        let text = "extraordinarily."; // 14 chars word + punctuation
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "extraordinarily");
        assert_eq!(tokens[0].punctuation, vec!['.']);
    }

    #[test]
    fn test_tokenize_single_word_no_phantom_newline() {
        let text = "hello"; // Single word with no newline
        let config = TimingConfig::default();
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
    }
}
