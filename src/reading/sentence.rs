//! Sentence boundary detection module
//!
//! Handles detection of sentence boundaries, abbreviations, and decimal numbers
//! that should not trigger sentence breaks.

use crate::reading::Token;

/// Check if word is a known abbreviation
pub fn is_abbreviation(word: &str) -> bool {
    const ABBREVIATIONS: &[&str] = &[
        "Dr.", "Mr.", "Mrs.", "Ms.", "St.", "Jr.", "e.g.", "i.e.", "vs.", "etc.",
    ];
    ABBREVIATIONS.contains(&word)
}

/// Check if word is a decimal number (e.g., "3.14")
pub fn is_decimal_number(word: &str) -> bool {
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        let has_digit_before = parts[0].chars().all(|c| c.is_ascii_digit());
        let has_digit_after = parts[1].chars().all(|c| c.is_ascii_digit());
        has_digit_before && has_digit_after
    } else {
        false
    }
}

/// Detects if current word starts a new sentence based on previous token.
/// MVP: Period/question/exclamation followed by capital letter A-Z, or newline.
/// First token always returns true (PRD Section 3.3 requirement).
/// Exceptions:
/// - Abbreviations (Dr., Mr., Mrs., etc.) do NOT end sentences
/// - Decimal numbers (3.14, 2.5) do NOT end sentences
pub fn detect_sentence_boundary(prev_token: Option<&Token>, current_word: &str) -> bool {
    let prev = match prev_token {
        None => return true,
        Some(token) => token,
    };
    let has_newline = prev.punctuation().contains(&'\n');

    if has_newline {
        return true;
    }

    let has_terminator = prev
        .punctuation()
        .iter()
        .any(|&p| p == '.' || p == '?' || p == '!');

    if !has_terminator {
        return false;
    }

    // Check for citations like [5] after period
    if has_terminator && current_word.starts_with('[') {
        return false;
    }

    // Reconstruct full word with punctuation for abbreviation/decimal checking
    let mut full_prev_word = prev.text().to_string();
    for &p in prev.punctuation() {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Abbreviation tests
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
    fn test_is_abbreviation_negative() {
        assert!(!is_abbreviation("hello."));
    }

    // Decimal number tests
    #[test]
    fn test_is_decimal_number_simple() {
        assert!(is_decimal_number("3.14"));
    }

    #[test]
    fn test_is_decimal_number_negative() {
        assert!(!is_decimal_number("hello."));
    }

    // Sentence boundary tests
    #[test]
    fn test_sentence_boundary_first_token() {
        let result = detect_sentence_boundary(None, "Hello");
        assert!(result, "First token should always be sentence start");
    }

    #[test]
    fn test_sentence_boundary_with_newline() {
        let prev = Token::new("hello".to_string(), vec!['\n'], false, 5, 1.0, 0, 1);
        let result = detect_sentence_boundary(Some(&prev), "World");
        assert!(result, "Newline should trigger sentence boundary");
    }
}
