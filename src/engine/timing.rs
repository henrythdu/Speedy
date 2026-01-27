// Timing engine - WPM calculation and tokenization

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub text: String,
    /// Trailing punctuation characters (e.g., ['?', '!'] for "word?!") per PRD Section 3.2 max stacking rule.
    pub punctuation: Vec<char>,
    /// Indicates if this token starts a new sentence (PRD Section 3.3).
    pub is_sentence_start: bool,
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
    (60_000.0 / wpm.max(1) as f64).round() as u64
}

/// Detects if current word starts a new sentence based on previous token.
/// MVP: Period/question/exclamation followed by capital letter A-Z, or newline.
/// First token always returns true (PRD Section 3.3 requirement).
pub fn detect_sentence_boundary(prev_token: Option<&Token>, current_word: &str) -> bool {
    if prev_token.is_none() {
        return true;
    }

    let prev = prev_token.unwrap();
    let has_newline = prev.punctuation.contains(&'\n');

    if has_newline {
        return true;
    }

    let has_terminator = prev
        .punctuation
        .iter()
        .any(|&p| p == '.' || p == '?' || p == '!');

    if !has_terminator {
        return false;
    }

    current_word
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
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
                let prev_token = tokens.last().cloned();
                let is_start = detect_sentence_boundary(prev_token.as_ref(), &word);

                tokens.push(Token {
                    text,
                    punctuation,
                    is_sentence_start: is_start,
                });
            }
        }

        // Create newline token after each line (except if line was empty/whitespace only)
        let prev_token = tokens.last().cloned();
        let is_start = detect_sentence_boundary(prev_token.as_ref(), "");
        tokens.push(Token {
            text: String::new(),
            punctuation: vec!['\n'],
            is_sentence_start: is_start,
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

    // Speedy-ui3: Tokenization Update tests

    #[test]
    fn test_is_sentence_start_set_correctly() {
        let text = "Hello. World? Good!";
        let tokens = tokenize_text(text);

        // Expected: [Hello., World?, Good!]
        // Sentence starts: Hello (first token), World (period before), Good (question mark before)
        assert_eq!(tokens.len(), 3);
        assert!(
            tokens[0].is_sentence_start,
            "First token should be marked as sentence start"
        );
        assert!(
            tokens[1].is_sentence_start,
            "World should be marked as sentence start (period before)"
        );
        assert!(
            tokens[2].is_sentence_start,
            "Good should be marked as sentence start (question mark before)"
        );
    }

    #[test]
    fn test_tokenize_single_sentence() {
        let text = "Hello world";
        let tokens = tokenize_text(text);

        // Expected: [Hello, world]
        // Only first token should be sentence start
        assert_eq!(tokens.len(), 2);
        assert!(
            tokens[0].is_sentence_start,
            "First token should be marked as sentence start"
        );
        assert!(
            !tokens[1].is_sentence_start,
            "Second token without terminator should NOT be sentence start"
        );
    }

    #[test]
    fn test_tokenize_multiple_sentences() {
        let text = "Hello. World! Good? Yes";
        let tokens = tokenize_text(text);

        // Expected: [Hello., World!, Good?, Yes]
        // Sentence starts: Hello, World, Good, Yes (each after terminator)
        assert_eq!(tokens.len(), 4);
        for (i, token) in tokens.iter().enumerate() {
            assert!(
                token.is_sentence_start,
                "Token {} should be marked as sentence start",
                i + 1
            );
        }
    }
}
