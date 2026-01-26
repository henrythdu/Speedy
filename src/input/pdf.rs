use super::{LoadError, LoadedDocument};
use crate::engine::timing::tokenize_text;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Load text from PDF file using pdf-extract crate.
///
/// Purpose: Provides PDF file content as input source per PRD Section 2.2.
/// Big Picture: Enables @filename.pdf command in REPL to load PDF content.
/// PRD Reference: Section 2.2 (PDF support), Section 7.1 (@filename command)
/// Connections: Depends on engine::tokenize_text() for tokenization.
pub fn load(path: &str) -> Result<LoadedDocument, LoadError> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(LoadError::FileNotFound(path.to_path_buf()));
    }

    // Read PDF file into memory
    let mut file = File::open(path).map_err(|e| LoadError::PdfParse(e.to_string()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| LoadError::PdfParse(e.to_string()))?;

    // Extract text from memory
    pdf_extract::extract_text_from_mem(&buffer)
        .map_err(|e| LoadError::PdfParse(e.to_string()))
        .map(|text| LoadedDocument {
            tokens: tokenize_text(&text),
            source: format!("pdf:{}", path.display()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::timing::Token;

    /// Test that load returns FileNotFound for non-existent files.
    #[test]
    fn test_pdf_load_nonexistent_file() {
        let result = load("/nonexistent/path/document.pdf");
        assert!(result.is_err());
        assert!(matches!(result, Err(LoadError::FileNotFound(_))));
    }

    /// Test that LoadedDocument has correct source field for PDF.
    #[test]
    fn test_loaded_document_source_pdf() {
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "pdf:/path/to/document.pdf".to_string(),
        };

        assert!(doc.source.starts_with("pdf:"));
    }

    /// Test that LoadedDocument tokens preserve punctuation from PDF text.
    #[test]
    fn test_loaded_document_tokens_preserve_punctuation() {
        let doc = LoadedDocument {
            tokens: tokenize_text("This is a test. It works!"),
            source: "pdf:test.pdf".to_string(),
        };

        // Verify multiple sentences are tokenized correctly
        assert!(doc.tokens.len() >= 3);
        // First token should be sentence start
        assert!(doc.tokens[0].is_sentence_start);
        // Tokens with periods should mark next token as sentence start
        for (i, token) in doc.tokens.iter().enumerate() {
            if token.text == "test" && i < doc.tokens.len() - 1 {
                assert!(
                    doc.tokens[i + 1].is_sentence_start,
                    "Token after 'test.' should be sentence start"
                );
            }
        }
    }

    /// Test PDF-specific error type.
    #[test]
    fn test_pdf_parse_error() {
        // This would test actual PDF parsing errors if we had a way to trigger them
        // For now, we verify the error type structure
        let err = LoadError::PdfParse("Invalid PDF structure".to_string());
        assert!(matches!(err, LoadError::PdfParse(msg) if msg.contains("Invalid")));
    }
}
