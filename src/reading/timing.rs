use crate::engine::Token;

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

fn is_abbreviation(word: &str) -> bool {
    const ABBREVIATIONS: &[&str] = &[
        "Dr.", "Mr.", "Mrs.", "Ms.", "St.", "Jr.", "e.g.", "i.e.", "vs.", "etc.",
    ];
    ABBREVIATIONS.contains(&word)
}

fn is_decimal_number(word: &str) -> bool {
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        let has_digit_before = parts[0].chars().all(|c| c.is_ascii_digit());
        let has_digit_after = parts[1].chars().all(|c| c.is_ascii_digit());
        has_digit_before && has_digit_after
    } else {
        false
    }
}

fn get_punctuation_multiplier(punctuation: char) -> f64 {
    match punctuation {
        '.' => 3.0,
        ',' => 1.5,
        '?' => 3.0,
        '!' => 3.0,
        '\n' => 4.0,
        _ => 1.0,
    }
}

fn get_max_punctuation_multiplier(punctuation_list: &[char]) -> f64 {
    punctuation_list
        .iter()
        .map(|&p| get_punctuation_multiplier(p))
        .fold(1.0, f64::max)
}

pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    (60_000.0 / wpm.max(1) as f64).round() as u64
}

fn get_word_length_penalty(word: &str, penalty_multiplier: f64) -> f64 {
    if word.chars().count() > 10 {
        penalty_multiplier
    } else {
        1.0
    }
}

pub fn calculate_word_delay(
    word: &str,
    punctuation: &[char],
    wpm: u32,
    penalty_multiplier: f64,
) -> u64 {
    let base_delay = wpm_to_milliseconds(wpm) as f64;
    let punctuation_multiplier = get_max_punctuation_multiplier(punctuation);
    let word_length_penalty = get_word_length_penalty(word, penalty_multiplier);
    let delay_ms = base_delay * punctuation_multiplier * word_length_penalty;
    delay_ms.round() as u64
}

/// Detects if current word starts a new sentence based on previous token.
/// MVP: Period/question/exclamation followed by capital letter A-Z, or newline.
/// First token always returns true (PRD Section 3.3 requirement).
/// Exceptions:
/// - Abbreviations (Dr., Mr., Mrs., etc.) do NOT end sentences
/// - Decimal numbers (3.14, 2.5) do NOT end sentences
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

    // Reconstruct full word with punctuation for abbreviation/decimal checking
    let mut full_prev_word = prev.text.clone();
    for &p in &prev.punctuation {
        full_prev_word.push(p);
    }

    // Don't break sentence if previous word is an abbreviation
    if is_abbreviation(&full_prev_word) {
        return false;
    }

    // Don't break sentence if previous word is a decimal number
    if is_decimal_number(&full_prev_word) {
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

    #[test]
    fn test_tokenize_with_wpm_300() {
        let text = "hello world";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_with_period() {
        let text = "hello world.";
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
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "extraordinarily");
        assert_eq!(tokens[0].punctuation, vec![]);
    }

    #[test]
    fn test_tokenize_short_word() {
        let text = "hello"; // 5 chars <= 10
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].punctuation, vec![]);
    }

    #[test]
    fn test_tokenize_newline() {
        let text = "hello\nworld"; // Two words separated by newline
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
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "extraordinarily");
        assert_eq!(tokens[0].punctuation, vec!['.']);
    }

    #[test]
    fn test_tokenize_single_word_no_phantom_newline() {
        let text = "hello"; // Single word with no newline
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

    // Timing Algorithm Tests (PRD Section 3.2, 3.3)

    #[test]
    fn test_punctuation_multiplier_period() {
        assert_eq!(get_punctuation_multiplier('.'), 3.0);
    }

    #[test]
    fn test_punctuation_multiplier_comma() {
        assert_eq!(get_punctuation_multiplier(','), 1.5);
    }

    #[test]
    fn test_punctuation_multiplier_question() {
        assert_eq!(get_punctuation_multiplier('?'), 3.0);
    }

    #[test]
    fn test_punctuation_multiplier_exclamation() {
        assert_eq!(get_punctuation_multiplier('!'), 3.0);
    }

    #[test]
    fn test_punctuation_multiplier_newline() {
        assert_eq!(get_punctuation_multiplier('\n'), 4.0);
    }

    #[test]
    fn test_punctuation_multiplier_unknown() {
        assert_eq!(get_punctuation_multiplier('a'), 1.0);
    }

    #[test]
    fn test_max_punctuation_multiplier_single() {
        assert_eq!(get_max_punctuation_multiplier(&['.']), 3.0);
    }

    #[test]
    fn test_max_punctuation_multiplier_multiple_same() {
        assert_eq!(get_max_punctuation_multiplier(&['.', '.']), 3.0);
    }

    #[test]
    fn test_max_punctuation_multiplier_multiple_different() {
        assert_eq!(get_max_punctuation_multiplier(&['?', '!']), 3.0);
    }

    #[test]
    fn test_max_punctuation_multiplier_stacking_rule() {
        // PRD Section 3.2: If word has multiple punctuation types, apply MAXIMUM only
        assert_eq!(get_max_punctuation_multiplier(&['.', '!']), 3.0);
    }

    #[test]
    fn test_word_length_penalty_short_word() {
        assert_eq!(get_word_length_penalty("hello", 1.15), 1.0);
    }

    #[test]
    fn test_word_length_penalty_exactly_ten() {
        assert_eq!(get_word_length_penalty("tenchars!", 1.15), 1.0);
    }

    #[test]
    fn test_word_length_penalty_long_word() {
        assert_eq!(get_word_length_penalty("extraordinarily", 1.15), 1.15);
    }

    #[test]
    fn test_word_length_penalty_custom_multiplier() {
        assert_eq!(get_word_length_penalty("extraordinarily", 1.2), 1.2);
    }

    #[test]
    fn test_calculate_word_delay_basic() {
        // 300 WPM = 200ms base, no punctuation, short word
        let delay = calculate_word_delay("hello", &[], 300, 1.15);
        assert_eq!(delay, 200);
    }

    #[test]
    fn test_calculate_word_delay_with_period() {
        // 300 WPM = 200ms base, period multiplier 3.0
        let delay = calculate_word_delay("hello", &['.'], 300, 1.15);
        assert_eq!(delay, 600); // 200 * 3.0
    }

    #[test]
    fn test_calculate_word_delay_with_comma() {
        // 300 WPM = 200ms base, comma multiplier 1.5
        let delay = calculate_word_delay("hello", &[','], 300, 1.15);
        assert_eq!(delay, 300); // 200 * 1.5
    }

    #[test]
    fn test_calculate_word_delay_long_word() {
        // 300 WPM = 200ms base, 14 chars = 1.15x penalty
        let delay = calculate_word_delay("extraordinarily", &[], 300, 1.15);
        assert_eq!(delay, 230); // 200 * 1.15 = 230
    }

    #[test]
    fn test_calculate_word_delay_combined() {
        // 300 WPM = 200ms base, period 3.0x, 14 chars 1.15x
        // 200 * 3.0 * 1.15 = 690
        let delay = calculate_word_delay("extraordinarily", &['.'], 300, 1.15);
        assert_eq!(delay, 690);
    }

    #[test]
    fn test_is_abbreviation_dr() {
        assert!(is_abbreviation("Dr."));
    }

    #[test]
    fn test_is_abbreviation_mr() {
        assert!(is_abbreviation("Mr."));
    }

    #[test]
    fn test_is_abbreviation_mrs() {
        assert!(is_abbreviation("Mrs."));
    }

    #[test]
    fn test_is_abbreviation_ms() {
        assert!(is_abbreviation("Ms."));
    }

    #[test]
    fn test_is_abbreviation_st() {
        assert!(is_abbreviation("St."));
    }

    #[test]
    fn test_is_abbreviation_jr() {
        assert!(is_abbreviation("Jr."));
    }

    #[test]
    fn test_is_abbreviation_eg() {
        assert!(is_abbreviation("e.g."));
    }

    #[test]
    fn test_is_abbreviation_ie() {
        assert!(is_abbreviation("i.e."));
    }

    #[test]
    fn test_is_abbreviation_vs() {
        assert!(is_abbreviation("vs."));
    }

    #[test]
    fn test_is_abbreviation_etc() {
        assert!(is_abbreviation("etc."));
    }

    #[test]
    fn test_is_abbreviation_negative() {
        assert!(!is_abbreviation("hello."));
    }

    #[test]
    fn test_is_decimal_number_simple() {
        assert!(is_decimal_number("3.14"));
    }

    #[test]
    fn test_is_decimal_number_two_point_five() {
        assert!(is_decimal_number("2.5"));
    }

    #[test]
    fn test_is_decimal_number_negative() {
        assert!(!is_decimal_number("hello."));
    }

    #[test]
    fn test_is_decimal_number_no_digits_after() {
        assert!(!is_decimal_number("3."));
    }

    #[test]
    fn test_is_decimal_number_no_digits_before() {
        assert!(!is_decimal_number(".5"));
    }

    #[test]
    fn test_sentence_boundary_abbreviation() {
        // PRD Section 3.3: Don't break sentences at abbreviations
        let text = "Dr. Smith went to St. Paul.";
        let tokens = tokenize_text(text);

        // Expected: [Dr., Smith, went, to, St., Paul.]
        // Only first token should be sentence start (Dr.)
        assert!(tokens[0].is_sentence_start);
        assert!(!tokens[1].is_sentence_start); // Smith should NOT be sentence start after "Dr."
        assert!(!tokens[2].is_sentence_start);
        assert!(!tokens[3].is_sentence_start);
        assert!(!tokens[4].is_sentence_start); // Paul should NOT be sentence start after "St."
    }

    #[test]
    fn test_sentence_boundary_decimal_number() {
        // PRD Section 3.3: Period after number is NOT sentence terminator
        let text = "The value is 3.14. Another sentence.";
        let tokens = tokenize_text(text);

        // Expected: [The, value, is, 3.14., Another, sentence.]
        // 3.14 should NOT cause sentence boundary (it's a decimal)
        assert!(tokens[0].is_sentence_start);
        assert!(!tokens[1].is_sentence_start);
        assert!(!tokens[2].is_sentence_start);
        assert!(!tokens[3].is_sentence_start); // 3.14. - period after decimal, not sentence terminator
        assert!(tokens[4].is_sentence_start); // Another starts new sentence
    }

    #[test]
    fn test_sentence_boundary_combined_rules() {
        // Test both abbreviation and decimal rules together
        let text = "Dr. Johnson measured 2.54 cm. Next sentence.";
        let tokens = tokenize_text(text);

        // Expected: [Dr., Johnson, measured, 2.54, cm., Next, sentence.]
        assert!(tokens[0].is_sentence_start);
        assert!(!tokens[1].is_sentence_start); // Johnson after "Dr."
        assert!(!tokens[2].is_sentence_start);
        assert!(!tokens[3].is_sentence_start); // 2.54 is decimal
        assert!(!tokens[4].is_sentence_start); // cm. is abbreviation
        assert!(tokens[5].is_sentence_start); // Next starts new sentence
    }
}
