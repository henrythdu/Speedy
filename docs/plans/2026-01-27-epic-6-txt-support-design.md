# Epic 6: Plain Text File Support

## Overview

Adds `.txt` file support to Speedy, expanding input formats beyond PDF and EPUB to include plain text files.

## Problem Statement

**Current State:**
```rust
// src/app/app.rs
Some(_) | None => {
    eprintln!("Unsupported format: {}", filename);
    eprintln!("Supported formats: .pdf, .epub");  // No .txt!
}
```

**User Need:**
- Many documents are in plain text format
- Users want to read .txt files without converting to PDF/EPUB
- Natural extension of existing file loading infrastructure

**Solution:**
- Add `src/input/text.rs` module
- Integrate with existing `handle_load_file()` logic
- Use existing `tokenize_text()` which works for any plain text

## Detailed Design

### Epic Scope

**2 Beads (Minimal Implementation)**

### Bead 6-1: Create Text File Loader

**New Module:** `src/input/text.rs`
```rust
use std::fs;
use std::path::Path;
use crate::engine::timing::{tokenize_text, Token};
use crate::input::{LoadedDocument, LoadError};

/// Load plain text file and tokenize its contents
pub fn load(path: &str) -> Result<LoadedDocument, LoadError> {
    let path_obj = Path::new(path);
    
    // Validate file exists
    if !path_obj.exists() {
        return Err(LoadError::FileNotFound(path_obj.to_path_buf()));
    }
    
    // Read file content
    let content = fs::read_to_string(path_obj)
        .map_err(|e| LoadError::ParseError(format!("Failed to read file: {}", e)))?;
    
    // Check if empty
    if content.trim().is_empty() {
        return Err(LoadError::ParseError("File is empty".to_string()));
    }
    
    // Tokenize using existing engine
    let tokens = tokenize_text(&content);
    
    // Check if tokenization produced any tokens
    if tokens.is_empty() {
        return Err(LoadError::ParseError("No readable content found".to_string()));
    }
    
    Ok(LoadedDocument {
        tokens,
        source: path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_valid_txt() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello world. This is a test.").unwrap();
        
        let result = load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let doc = result.unwrap();
        assert_eq!(doc.tokens.len(), 8); // Hello, world, This, is, a, test
    }
    
    #[test]
    fn test_load_nonexistent_file() {
        let result = load("/nonexistent/file.txt");
        assert!(matches!(result, Err(LoadError::FileNotFound(_))));
    }
    
    #[test]
    fn test_load_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = load(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(LoadError::ParseError(msg)) if msg.contains("empty")));
    }
}
```

**Key Implementation Details:**

1. **Error Handling:**
   - `FileNotFound` - File doesn't exist
   - `ParseError` - IO error or empty file
   - Consistent with PDF/EPUB error types

2. **Tokenization:**
   - Reuses `engine::timing::tokenize_text()`
   - No custom parsing needed for plain text
   - Gets sentence boundary detection for free

3. **Testing:**
   - Use `tempfile` crate for test files
   - Test success, not found, empty cases
   - Verify token count matches expectations

**Acceptance Criteria:**
- Module compiles without errors
- Can load .txt file and produce tokens
- Proper error types returned
- Tests pass with good coverage

### Bead 6-2: Integrate txt Loader

**Update Input Module:** `src/input/mod.rs`
```rust
pub mod pdf;
pub mod epub;
pub mod clipboard;
pub mod text;  // NEW: Add text module

// Re-export loaders with consistent interface
pub use pdf::load as load_pdf;
pub use epub::load as load_epub;
pub use clipboard::load as load_clipboard;
pub use text::load as load_text;  // NEW: Export text loader
```

**Update File Handler:** `src/app/app.rs`
```rust
fn handle_load_file(&mut self, path: &str) {
    let path_obj = Path::new(path);
    let ext = path_obj.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    
    match ext.as_deref() {
        Some("pdf") => {
            match pdf::load(path.to_str().unwrap_or("")) {
                Ok(doc) => self.apply_loaded_document(doc),
                Err(e) => self.handle_load_error(&e),
            }
        }
        Some("epub") => {
            match epub::load(path.to_str().unwrap_or("")) {
                Ok(doc) => self.apply_loaded_document(doc),
                Err(e) => self.handle_load_error(&e),
            }
        }
        Some("txt") => {  // NEW: .txt file support
            match text::load(path.to_str().unwrap_or("")) {
                Ok(doc) => self.apply_loaded_document(doc),
                Err(e) => self.handle_load_error(&e),
            }
        }
        Some(_) | None => {
            let filename = path_obj.file_name()
                .map_or_else(|| "unknown".to_string(), |n| n.to_string_lossy().to_string());
            eprintln!("Unsupported format: {}", filename);
            eprintln!("Supported formats: .pdf, .epub, .txt");  // UPDATED: Now includes .txt
            eprintln!("For clipboard, use @@ command");
        }
    }
}
```

**Key Changes:**
1. Add `"txt"` match arm in `handle_load_file()`
2. Update error message to include `.txt`
3. Use existing `apply_loaded_document()` (unified interface)

**Integration Tests:**
```rust
// tests/integration_test.rs
#[test]
fn test_load_txt_file() {
    let mut app = App::new();
    
    // Use a fixture text file
    let test_path = "tests/fixtures/sample.txt";
    app.handle_event(AppEvent::LoadFile(test_path.to_string()));
    
    assert_eq!(app.mode(), AppMode::Reading);
    assert!(app.reading_state.is_some());
    
    let state = app.get_render_state();
    assert!(state.tokens.len() > 0);
}

#[test]
fn test_txt_command_in_repl() {
    let mut app = App::new();
    let event = parse_repl_input("@sample.txt");
    
    app.handle_event(event);
    assert_eq!(app.mode(), AppMode::Reading);
}
```

**Acceptance Criteria:**
- Can type `@file.txt` and load successfully
- Same behavior as `@file.pdf` or `@file.epub`
- Error message lists .txt as supported format
- All existing tests still pass

## Implementation Order

**Depends on:** Nothing (can be done in parallel with Epic 3)
- No dependencies on visual changes
- No dependencies on command mode
- Purely additive file format support

**Within Epic 6:**
1. Create text.rs module (1 hour)
2. Update integration points (30 minutes)
3. Write tests (30 minutes)

## Testing Checklist

- [ ] Can load `tests/fixtures/sample.txt` with multiple sentences
- [ ] Tokenization matches expected word count
- [ ] Error message shows .txt as supported format
- [ ] @file.txt syntax works in REPL
- [ ] Works with files containing unicode characters
- [ ] Handles files with Windows/Unix line endings
- [ ] Rejects binary files gracefully

## Test Files Needed

Create in `tests/fixtures/`:
- `sample.txt` - Simple multi-sentence text
- `unicode.txt` - Unicode character test
- `empty_lines.txt` - Multiple paragraphs with spacing
- `single_sentence.txt` - Edge case

## Dependencies

**No external crates:**
- Uses `std::fs` (already available)
- Reuses `tempfile` for testing (dev dependency, may already exist)

**Dev Dependency Addition:**
```toml
# Cargo.toml (dev-dependencies)
[dev-dependencies]
tempfile = "3.0"  # For tests that need temporary files
```

## Risks

**Very Low Risk** - Simple file I/O

**Potential Issues:**
- Encoding detection (assume UTF-8 for MVP)
- Large files (test with 1MB+ text file)
- Very long lines (tokenization performance)

**Mitigations:**
- Document UTF-8 requirement
- Test with realistic file sizes
- Tokenizer already handles long lines

## Backwards Compatibility

**No Breaking Changes:**
- New capability, existing flows unchanged
- PDF/EPUB/clipboard still work as before
- Existing error message updated (additive)

## PRD Alignment

- **PRD 2.2 (Formats)**: ✅ Extends "Supported Formats" table
- **PRD 2.3 (Discovery)**: Works with tab completion, recursive search
- **PRD 6.1 (Project Structure)**: Adds text.rs to input/ module

## Future Extensions (Not in Epic 6)

- Encoding detection (UTF-8, UTF-16, etc.)
- Markdown parsing (headings, emphasis)
- Very large file handling (streaming tokenizer)
- Compressed text files (.txt.gz)
- Remote text files (HTTP URLs)

---

## Summary

Epic 6 adds essential .txt file support:

1. **Text Loader Module** - Reuses existing tokenization
2. **Format Integration** - Works seamlessly with existing commands
3. **Error Handling** - Consistent with PDF/EPUB loading

This removes a major friction point and makes Speedy viable for everyday use with common document formats.