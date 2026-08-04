use crate::reading::sentence::detect_sentence_boundary;
use crate::reading::Token;

fn extract_punctuation(word: &str) -> (String, Vec<char>) {
    if word.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut chars: Vec<char> = word.chars().collect();
    let mut punctuation_chars = Vec::new();

    // Collect all trailing punctuation characters
    while let Some(&last_char) = chars.last() {
        if is_sentence_terminator(last_char) || is_comma(last_char) {
            // SAFETY: chars.pop() is guaranteed to return Some because we just
            // verified chars.last() is Some in the while condition
            punctuation_chars.push(chars.pop().expect("chars should have last element"));
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

/// Tokenizes text; PRD Section 3.2.
/// Only creates pause tokens for paragraph breaks (2+ consecutive newlines), not single newlines.
/// Single newlines (line wrapping) are treated as word separators only.
pub fn tokenize_text(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut consecutive_empty_lines = 0;
    let mut current_sentence_tokens: Vec<usize> = Vec::new(); // Indices of tokens in current sentence

    for line in text.lines() {
        let is_empty = line.trim().is_empty();

        if is_empty {
            consecutive_empty_lines += 1;
            // Only create a paragraph break token after 2+ consecutive empty lines
            if consecutive_empty_lines == 2 {
                // Finalize previous sentence before adding paragraph break
                finalize_sentence(&mut tokens, &mut current_sentence_tokens);

                let is_start = true; // Paragraph breaks indicate sentence boundaries
                let char_count = 0;
                let sentence_index = 0;
                let sentence_length = 1;
                tokens.push(Token::new(
                    String::new(),
                    vec!['\n'],
                    is_start,
                    char_count,
                    sentence_index,
                    sentence_length,
                ));
            }
        } else {
            // Reset empty line counter when we hit content
            consecutive_empty_lines = 0;

            // Process words in current line
            for word in line.split_whitespace() {
                if !word.is_empty() {
                    let (text, punctuation) = extract_punctuation(word);
                    let prev_token = tokens.last();
                    let is_start = detect_sentence_boundary(prev_token, word);

                    // Start of new sentence detected
                    if is_start && !current_sentence_tokens.is_empty() {
                        finalize_sentence(&mut tokens, &mut current_sentence_tokens);
                    }

                    let char_count = text.chars().count();

                    // Add token with placeholder sentence info (will be updated)
                    let token_index = tokens.len();
                    current_sentence_tokens.push(token_index);

                    tokens.push(Token::new(
                        text,
                        punctuation,
                        is_start,
                        char_count,
                        0, // placeholder
                        0, // placeholder
                    ));
                }
            }
        }
    }

    // Finalize the last sentence
    finalize_sentence(&mut tokens, &mut current_sentence_tokens);

    // Remove trailing newline token if it exists (last line doesn't need newline after it)
    if tokens
        .last()
        .is_some_and(|t| t.punctuation() == ['\n'] && t.text().is_empty())
    {
        tokens.pop();
    }

    tokens
}

/// Finalizes sentence metadata for all tokens in the current sentence.
fn finalize_sentence(tokens: &mut [Token], sentence_token_indices: &mut Vec<usize>) {
    let sentence_length = sentence_token_indices.len();

    for (idx, &token_idx) in sentence_token_indices.iter().enumerate() {
        if let Some(token) = tokens.get_mut(token_idx) {
            // We need to reconstruct the token with updated sentence info
            // Since Token fields are private and there's no setter, we replace the token
            let new_token = Token::new(
                token.text().to_string(),
                token.punctuation().to_vec(),
                token.is_sentence_start(),
                token.char_count(),
                idx,             // sentence_index
                sentence_length, // sentence_length
            );
            tokens[token_idx] = new_token;
        }
    }

    sentence_token_indices.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_single_word() {
        let text = "hello";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text(), "hello");
    }

    #[test]
    fn test_tokenize_multiple_words() {
        let text = "hello world";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text(), "hello");
        assert_eq!(tokens[1].text(), "world");
    }

    #[test]
    fn test_tokenize_with_period() {
        let text = "hello world.";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text(), "hello");
        assert_eq!(tokens[0].punctuation(), Vec::<char>::new());
        assert_eq!(tokens[1].text(), "world");
        assert_eq!(tokens[1].punctuation(), vec!['.']);
    }

    #[test]
    fn test_tokenize_with_exclamation() {
        let text = "hello world!";
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text(), "hello");
        assert_eq!(tokens[0].punctuation(), Vec::<char>::new());
        assert_eq!(tokens[1].text(), "world");
        assert_eq!(tokens[1].punctuation(), vec!['!']);
    }

    #[test]
    fn test_tokenize_single_newline_no_pause() {
        // Single newlines (line wrapping) should NOT create pause tokens
        let text = "hello\nworld"; // Two words separated by single newline
        let tokens = tokenize_text(text);
        assert_eq!(
            tokens.len(),
            2,
            "Single newline should not create a pause token"
        );
        assert_eq!(tokens[0].text(), "hello");
        assert_eq!(tokens[1].text(), "world");
    }

    #[test]
    fn test_tokenize_single_word_no_phantom_newline() {
        let text = "hello"; // Single word with no newline
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text(), "hello");
        assert_eq!(tokens[0].punctuation(), Vec::<char>::new());
    }

    #[test]
    fn test_is_sentence_start_set_correctly() {
        let text = "Hello. World? Good!";
        let tokens = tokenize_text(text);

        // Expected: [Hello., World?, Good!]
        // Sentence starts: Hello (first token), World (period before), Good (question mark before)
        assert_eq!(tokens.len(), 3);
        assert!(
            tokens[0].is_sentence_start(),
            "First token should be marked as sentence start"
        );
        assert!(
            tokens[1].is_sentence_start(),
            "World should be marked as sentence start (period before)"
        );
        assert!(
            tokens[2].is_sentence_start(),
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
            tokens[0].is_sentence_start(),
            "First token should be marked as sentence start"
        );
        assert!(
            !tokens[1].is_sentence_start(),
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
                token.is_sentence_start(),
                "Token {} should be marked as sentence start",
                i + 1
            );
        }
    }

    #[test]
    fn test_sentence_boundary_abbreviation() {
        // PRD Section 3.3: Don't break sentences at abbreviations
        let text = "Dr. Smith went to St. Paul.";
        let tokens = tokenize_text(text);

        // Expected: [Dr., Smith, went, to, St., Paul.]
        // Only first token should be sentence start (Dr.)
        assert!(tokens[0].is_sentence_start());
        assert!(!tokens[1].is_sentence_start()); // Smith should NOT be sentence start after "Dr."
        assert!(!tokens[2].is_sentence_start());
        assert!(!tokens[3].is_sentence_start());
        assert!(!tokens[4].is_sentence_start()); // Paul should NOT be sentence start after "St."
    }

    #[test]
    fn test_sentence_boundary_decimal_number() {
        // PRD Section 3.3: Period after number is NOT sentence terminator
        let text = "The value is 3.14. Another sentence.";
        let tokens = tokenize_text(text);

        // Expected: [The, value, is, 3.14., Another, sentence.]
        // 3.14 should NOT cause sentence boundary (it's a decimal)
        assert!(tokens[0].is_sentence_start());
        assert!(!tokens[1].is_sentence_start());
        assert!(!tokens[2].is_sentence_start());
        assert!(!tokens[3].is_sentence_start()); // 3.14. - period after decimal, not sentence terminator
        assert!(tokens[4].is_sentence_start()); // Another starts new sentence
    }
}
