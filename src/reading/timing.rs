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

pub fn wpm_to_milliseconds(wpm: u32) -> u64 {
    (60_000.0 / wpm.max(1) as f64).round() as u64
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

    // Check for citations like [5] after period
    if has_terminator && current_word.starts_with('[') {
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

/// Tokenizes text; PRD Section 3.2.
/// Only creates pause tokens for paragraph breaks (2+ consecutive newlines), not single newlines.
/// Single newlines (line wrapping) are treated as word separators only.
pub fn tokenize_text(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut consecutive_empty_lines = 0;

    for line in text.lines() {
        let is_empty = line.trim().is_empty();
        
        if is_empty {
            consecutive_empty_lines += 1;
            // Only create a paragraph break token after 2+ consecutive empty lines
            if consecutive_empty_lines == 2 {
                let prev_token = tokens.last().cloned();
                let is_start = true;  // Paragraph breaks indicate sentence boundaries
                tokens.push(Token {
                    text: String::new(),
                    punctuation: vec!['\n'],
                    is_sentence_start: is_start,
                });
            }
        } else {
            // Reset empty line counter when we hit content
            consecutive_empty_lines = 0;
            
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
        }
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
    fn test_tokenize_single_newline_no_pause() {
        // Single newlines (line wrapping) should NOT create pause tokens
        let text = "hello\nworld"; // Two words separated by single newline
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 2, "Single newline should not create a pause token");
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    fn test_tokenize_paragraph_break_creates_pause() {
        // Double newlines (paragraph breaks) SHOULD create pause tokens
        let text = "hello\n\nworld"; // Two words with paragraph break
        let tokens = tokenize_text(text);
        assert_eq!(tokens.len(), 3, "Paragraph break should create a pause token");
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, ""); // Paragraph break token
        assert_eq!(tokens[1].punctuation, vec!['\n']);
        assert_eq!(tokens[2].text, "world");
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


    #[test]
    fn test_clipboard_text_rendering_sequence() {
        // Simulate the user's clipboard text
        let text = "Pastas are divided into two broad categories: dried (Italian: pasta secca) and fresh (Italian: pasta fresca). Most dried pasta is produced commercially via an extrusion process, although it can be produced at home. Fresh pasta is traditionally produced by hand, sometimes with the aid of simple machines.[5] Fresh pastas available in grocery stores are produced commercially by large-scale machines.\n\nBoth dried and fresh pastas come in a number of shapes and varieties, with 310 specific forms known by over 1,300 documented names.[6] In Italy, the names of specific pasta shapes or types often vary by locale. For example, the pasta form cavatelli is known by 28 different names depending upon the town and region. Common forms of pasta include long and short shapes, tubes, flat shapes or sheets, miniature shapes for soup, those meant to be filled or stuffed, and specialty or decorative shapes";
        
        let tokens = tokenize_text(text);
        
        // Print first 30 tokens to see the sequence
        println!("\nFirst 30 tokens:");
        for (i, token) in tokens.iter().take(30).enumerate() {
            let text_display = if token.text.is_empty() { "<EMPTY>".to_string() } else { token.text.clone() };
            println!("  {}: '{}' [punct={:?}, sent_start={}]", 
                i, text_display, token.punctuation, token.is_sentence_start);
        }
        
        // Count empty tokens
        let empty_count = tokens.iter().filter(|t| t.text.is_empty()).count();
        println!("\nTotal tokens: {}", tokens.len());
        println!("Empty (newline) tokens: {}", empty_count);
        
        // Show what words would be rendered (non-empty tokens)
        println!("\nWords that would be rendered (first 20):");
        let rendered_words: Vec<&str> = tokens.iter()
            .filter(|t| !t.text.is_empty())
            .map(|t| t.text.as_str())
            .take(20)
            .collect();
        println!("  {}", rendered_words.join(" "));
    }


    #[test]
    fn test_trace_rendering_simulation() {
        // Simulate exactly what happens in the event loop
        let text = "Pastas are divided into two broad categories: dried (Italian: pasta secca) and fresh (Italian: pasta fresca). Most dried pasta is produced commercially via an extrusion process, although it can be produced at home. Fresh pasta is traditionally produced by hand, sometimes with the aid of simple machines.[5] Fresh pastas available in grocery stores are produced commercially by large-scale machines.\n\nBoth dried and fresh pastas come in a number of shapes and varieties, with 310 specific forms known by over 1,300 documented names.[6] In Italy, the names of specific pasta shapes or types often vary by locale. For example, the pasta form cavatelli is known by 28 different names depending upon the town and region. Common forms of pasta include long and short shapes, tubes, flat shapes or sheets, miniature shapes for soup, those meant to be filled or stuffed, and specialty or decorative shapes";
        
        let tokens = tokenize_text(text);
        
        // Find indices of words that appear in the "rendered" output
        let rendered_words = vec!["Pastas", "dried", "pasta", "Both", "example", "forms", "of", "include", "long", "and", "short", "shapes", "tubes", "flat", "shape", "or", "sheet", "miniature", "shapes", "for", "soup", "those", "meant", "to", "be", "filled", "or", "stuffed", "and", "speciality", "or", "decorative", "shapes"];
        
        println!("\n=== Searching for rendered words in token sequence ===");
        for search_word in &rendered_words {
            let positions: Vec<usize> = tokens.iter()
                .enumerate()
                .filter(|(_, t)| t.text == *search_word)
                .map(|(i, _)| i)
                .collect();
            
            if !positions.is_empty() {
                println!("  '{}' found at indices: {:?}", search_word, positions);
            }
        }
        
        // Simulate 10 rendering iterations
        println!("\n=== Simulating first 10 render cycles ===");
        let mut current_index = 0;
        for cycle in 0..10 {
            if current_index >= tokens.len() {
                break;
            }
            
            let token = &tokens[current_index];
            let display = if token.text.is_empty() { 
                "<NEWLINE>".to_string() 
            } else { 
                token.text.clone() 
            };
            
            println!("  Cycle {}: index={}, word='{}', empty={}", 
                cycle, current_index, display, token.text.is_empty());
            
            current_index += 1;
        }
    }


    #[test]
    fn test_word_characteristics() {
        let text = "Pastas are divided into two broad categories: dried (Italian: pasta secca) and fresh (Italian: pasta fresca).";
        let tokens = tokenize_text(text);
        
        println!("\n=== Analyzing word characteristics ===");
        for (i, token) in tokens.iter().enumerate() {
            if token.text.is_empty() {
                continue;
            }
            
            let has_punctuation_in_text = token.text.chars().any(|c| !c.is_alphanumeric());
            let starts_with_paren = token.text.starts_with('(');
            let ends_with_paren = token.text.ends_with(')');
            let ends_with_colon = token.text.ends_with(':');
            
            println!("  {:2}: '{:15}' len={:2} punct_in_text={} starts_paren={} ends_paren={} ends_colon={}",
                i, token.text, token.text.len(), 
                has_punctuation_in_text, starts_with_paren, ends_with_paren, ends_with_colon);
        }
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

#[test]
fn debug_tokenize_pasta_text() {
    let text = "Pastas are divided into two broad categories: dried (Italian: pasta secca) and fresh (Italian: pasta fresca).";
    let tokens = tokenize_text(text);
    eprintln!("=== Tokenization Debug ===");
    eprintln!("Text: {}", text);
    eprintln!("Token count: {}", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        eprintln!("Token {}: text='{}' punct={:?} is_sentence_start={}", 
            i, token.text, token.punctuation, token.is_sentence_start);
    }
}


    // Tokenizer filtering tests - PRD requirement: no empty/whitespace-only tokens

    #[test]
    fn test_tokenize_filters_empty_tokens_from_blank_lines() {
        let text = "hello\n\nworld"; // Two newlines = blank line
        let tokens = tokenize_text(text);
        
        // Should have: "hello", "world" - but NOT an empty token for the blank line
        let empty_tokens: Vec<&Token> = tokens.iter()
            .filter(|t| t.text.trim().is_empty() && t.punctuation.is_empty())
            .collect();
        
        assert!(empty_tokens.is_empty(), 
            "Tokenizer should not produce empty tokens for blank lines. Found: {:?}", 
            empty_tokens);
    }

    #[test]
    fn test_tokenize_filters_whitespace_only_tokens() {
        let text = "hello\n   \nworld"; // Line with only spaces
        let tokens = tokenize_text(text);
        
        // Should not have any tokens with only whitespace
        let whitespace_tokens: Vec<&Token> = tokens.iter()
            .filter(|t| t.text.trim().is_empty() && !t.text.is_empty())
            .collect();
        
        assert!(whitespace_tokens.is_empty(), 
            "Tokenizer should not produce whitespace-only tokens. Found: {:?}", 
            whitespace_tokens);
    }

    #[test]
    fn test_tokenize_preserves_newline_punctuation() {
        let text = "hello\nworld"; // Single newline between words
        let tokens = tokenize_text(text);
        
        // Should have 3 tokens: "hello", newline marker, "world"
        assert_eq!(tokens.len(), 3, "Should have hello, newline, world");
        assert_eq!(tokens[0].text, "hello");
        assert!(tokens[1].text.is_empty(), "Newline token should have empty text");
        assert_eq!(tokens[1].punctuation, vec!['\n'], "Newline token should have newline punctuation");
        assert_eq!(tokens[2].text, "world");
    }

    #[test]
    fn test_tokenize_no_duplicate_empty_tokens() {
        let text = "a\n\n\nb"; // Multiple blank lines
        let tokens = tokenize_text(text);
        
        // Count empty tokens
        let empty_count = tokens.iter()
            .filter(|t| t.text.trim().is_empty())
            .count();
        
        // Should have at most 1 newline token (for the actual line break, not blank lines)
        assert!(empty_count <= 2, 
            "Should not have multiple empty tokens for blank lines. Found {} empty tokens", 
            empty_count);
    }

    #[test]
    fn test_all_tokens_have_meaningful_content() {
        let text = "Hello world.\n\nThis is a test.\n\n\nEnd.";
        let tokens = tokenize_text(text);
        
        // Every token should have either text content OR meaningful punctuation
        for (i, token) in tokens.iter().enumerate() {
            let has_text = !token.text.trim().is_empty();
            let has_meaningful_punct = token.punctuation.iter().any(|&p| p != '\n');
            let is_newline = token.punctuation == vec!['\n'];
            
            assert!(
                has_text || has_meaningful_punct || is_newline,
                "Token {} should have meaningful content. Got: text='{}' punct={:?}",
                i, token.text, token.punctuation
            );
        }
    }
