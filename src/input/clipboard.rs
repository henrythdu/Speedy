use super::{LoadError, LoadedDocument};
use crate::engine::tokenize_text;

/// Load text from system clipboard using arboard crate.
///
/// Purpose: Provides clipboard content as input source per PRD Section 2.2.
/// Big Picture: Enables @@ command in REPL to load clipboard content.
/// PRD Reference: Section 2.2 (Clipboard support), Section 7.1 (@@ command)
/// Connections: Depends on engine::tokenize_text() for tokenization.
pub fn load() -> Result<LoadedDocument, LoadError> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.get_text())
        .map_err(|e| LoadError::Clipboard(e.to_string()))
        .map(|text| LoadedDocument {
            tokens: tokenize_text(&text),
            source: "clipboard".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::token::Token;

    /// Test successful clipboard load with mocked arboard.
    /// This test verifies tokenization works correctly with clipboard content.
    #[test]
    fn test_clipboard_load_success() {
        let mock_text = "Hello world. This is a test.";
        let tokens = tokenize_text(mock_text);

        // Verify tokens are created correctly
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].text, "Hello");
        assert!(tokens[0].is_sentence_start);
    }

    /// Test that LoadedDocument has correct source field.
    #[test]
    fn test_loaded_document_source() {
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "clipboard".to_string(),
        };

        assert_eq!(doc.source, "clipboard");
    }

    /// Test that LoadedDocument tokens preserve punctuation.
    #[test]
    fn test_loaded_document_tokens_preserve_punctuation() {
        let doc = LoadedDocument {
            tokens: tokenize_text("Hello, world!"),
            source: "clipboard".to_string(),
        };

        // First token should be "Hello" with comma punctuation
        assert_eq!(doc.tokens[0].text, "Hello");
        assert_eq!(doc.tokens[0].punctuation, vec![',']);
        assert!(doc.tokens[0].is_sentence_start);

        // Second token should be "world" with exclamation punctuation
        assert_eq!(doc.tokens[1].text, "world");
        assert_eq!(doc.tokens[1].punctuation, vec!['!']);
        assert!(!doc.tokens[1].is_sentence_start);
    }
}
