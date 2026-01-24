use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("PDF parse error: {0}")]
    PdfParse(String),

    #[error("EPUB parse error: {0}")]
    EpubParse(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub struct LoadedDocument {
    pub tokens: Vec<crate::engine::timing::Token>,
    pub source: String,
}

pub mod clipboard;
pub mod epub;
pub mod pdf;
