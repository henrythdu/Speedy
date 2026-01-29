use super::{LoadError, LoadedDocument};
use crate::engine::tokenize_text;
use std::path::Path;

/// Load text from EPUB file using epub crate.
///
/// Purpose: Provides EPUB file content as input source per PRD Section 2.2.
/// Big Picture: Enables @filename.epub command in REPL to load EPUB content.
/// PRD Reference: Section 2.2 (EPUB support), Section 7.1 (@filename command)
/// Connections: Depends on engine::tokenize_text() for tokenization.
pub fn load(path: &str) -> Result<LoadedDocument, LoadError> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(LoadError::FileNotFound(path.to_path_buf()));
    }

    let mut doc = epub::doc::EpubDoc::new(path).map_err(|e| LoadError::EpubParse(e.to_string()))?;

    let num_chapters = doc.get_num_chapters();

    if num_chapters == 0 {
        return Err(LoadError::EpubParse(
            "No chapters found in EPUB".to_string(),
        ));
    }

    // Extract all chapter content and concatenate with double newlines
    let mut content = String::new();

    for chapter_idx in 0..num_chapters {
        if !doc.set_current_chapter(chapter_idx) {
            continue;
        }

        if let Some((chapter_content, _mime)) = doc.get_current_str() {
            if !chapter_content.is_empty() {
                if !content.is_empty() {
                    content.push_str("\n\n");
                }
                // Extract plain text from HTML content
                let plain_text = extract_plain_text(&chapter_content);
                content.push_str(&plain_text);
            }
        }
    }

    if content.is_empty() {
        return Err(LoadError::EpubParse(
            "No extractable text content found in EPUB".to_string(),
        ));
    }

    Ok(LoadedDocument {
        tokens: tokenize_text(&content),
        source: format!("epub:{}", path.display()),
    })
}

/// Extract plain text from HTML content by removing tags.
fn extract_plain_text(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    // Clean up extra whitespace
    result
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Token;

    /// Test that load returns FileNotFound for non-existent files.
    #[test]
    fn test_epub_load_nonexistent_file() {
        let result = load("/nonexistent/path/book.epub");
        assert!(result.is_err());
        assert!(matches!(result, Err(LoadError::FileNotFound(_))));
    }

    /// Test that LoadedDocument has correct source field for EPUB.
    #[test]
    fn test_loaded_document_source_epub() {
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "epub:/path/to/book.epub".to_string(),
        };

        assert!(doc.source.starts_with("epub:"));
    }

    /// Test that LoadedDocument tokens preserve sentence boundaries from EPUB text.
    #[test]
    fn test_loaded_document_sentence_boundaries() {
        let doc = LoadedDocument {
            tokens: tokenize_text("Chapter One. This is the first sentence. And another! Yes?"),
            source: "epub:test.epub".to_string(),
        };

        // Verify sentence boundaries are detected
        assert!(doc.tokens.len() >= 4);

        // Find tokens that should be sentence starts
        let sentence_starts: Vec<&Token> =
            doc.tokens.iter().filter(|t| t.is_sentence_start).collect();

        // Should have multiple sentence starts (first token + after each terminator)
        assert!(
            sentence_starts.len() >= 2,
            "Should have at least 2 sentence starts, got {}",
            sentence_starts.len()
        );

        // First token should always be sentence start
        assert!(doc.tokens[0].is_sentence_start);
    }

    /// Test EPUB-specific error type.
    #[test]
    fn test_epub_parse_error() {
        let err = LoadError::EpubParse("Invalid EPUB structure".to_string());
        assert!(matches!(err, LoadError::EpubParse(msg) if msg.contains("Invalid")));
    }

    /// Test plain text extraction from HTML.
    #[test]
    fn test_extract_plain_text() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let result = extract_plain_text(html);
        assert!(result.contains("Hello World"));
        assert!(!result.contains("<html>"));
        assert!(!result.contains("<p>"));
    }
}
