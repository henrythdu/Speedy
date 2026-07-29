//! File autocomplete module for TUI command deck
//!
//! Provides file picker functionality triggered by typing `@` in the command deck.
//! Supports PDF and EPUB files from the current directory and subdirectories.
//!
//! ## Architecture
//!
//! The autocomplete system consists of four main components:
//!
//! - **Discovery** (`discovery.rs`): Threaded file scanning that runs asynchronously
//! - **Cache** (`cache.rs`): Per-directory cache to avoid repeated scans
//! - **State** (`state.rs`): Manages autocomplete popup state and user interactions
//! - **Render** (`render.rs`): Renders the popup using ratatui
//!
//! ## Usage
//!
//! The autocomplete is triggered when the user types `@` at the start of the
//! command buffer or after whitespace. A popup appears showing matching files,
//! which can be navigated with arrow keys and selected with Enter or Tab.
//!
//! ## Thread Safety
//!
//! File discovery runs in a background thread and communicates with the main
//! thread via mpsc channels. The cache uses Arc<Mutex<>> for thread-safe access.

pub mod cache;
pub mod controller;
pub mod discovery;
pub mod render;
pub mod state;

use std::path::Path;

/// Supported file extensions for autocomplete
pub const SUPPORTED_EXTENSIONS: &[&str] = &["pdf", "epub"];

/// Maximum directory depth for file scanning
pub const MAX_SCAN_DEPTH: usize = 5;

/// Maximum number of files to discover (prevents UI overflow)
pub const MAX_FILES: usize = 1000;

/// Cache time-to-live in seconds
pub const CACHE_TTL_SECONDS: u64 = 30;

/// Maximum number of visible items in the popup
pub const MAX_VISIBLE_ITEMS: usize = 10;

/// Check if a file has a supported extension
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `true` if the file extension is in SUPPORTED_EXTENSIONS
pub fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|&supported| ext.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

/// Get a display prefix for a file based on its extension
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// A string prefix like "[PDF]" or "[EPUB]"
pub fn get_file_prefix(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("pdf") => "[PDF]",
        Some(ext) if ext.eq_ignore_ascii_case("epub") => "[EPUB]",
        _ => "[FILE]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_file_pdf() {
        assert!(is_supported_file(Path::new("document.pdf")));
        assert!(is_supported_file(Path::new("document.PDF")));
    }

    #[test]
    fn test_is_supported_file_epub() {
        assert!(is_supported_file(Path::new("book.epub")));
        assert!(is_supported_file(Path::new("book.EPUB")));
    }

    #[test]
    fn test_is_supported_file_unsupported() {
        assert!(!is_supported_file(Path::new("document.txt")));
        assert!(!is_supported_file(Path::new("script.js")));
        assert!(!is_supported_file(Path::new("no_extension")));
    }

    #[test]
    fn test_get_file_prefix() {
        assert_eq!(get_file_prefix(Path::new("doc.pdf")), "[PDF]");
        assert_eq!(get_file_prefix(Path::new("doc.PDF")), "[PDF]");
        assert_eq!(get_file_prefix(Path::new("book.epub")), "[EPUB]");
        assert_eq!(get_file_prefix(Path::new("book.EPUB")), "[EPUB]");
        assert_eq!(get_file_prefix(Path::new("other.txt")), "[FILE]");
    }
}
