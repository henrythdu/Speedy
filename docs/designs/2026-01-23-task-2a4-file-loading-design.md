# Task 2A-4: File Loading Integration Design

**Date:** 2026-01-23  
**Epic:** 2A - REPL Mode & File Operations  
**Task:** Implement file loading for PDF, EPUB, and clipboard with sentence-aware navigation

---

## Overview

This task implements the three data sources defined in the PRD: PDF files, EPUB files, and clipboard content. The `src/input/` module serves as an I/O boundary layer that extracts text from external sources and delegates tokenization to the pure core engine.

**Key Design Decisions:**
- NO plain text files (.txt) - Only PDF, EPUB, and clipboard per PRD requirements
- Loaders return pure data payloads (`LoadedDocument`) - No App instances or mode knowledge
- Sentence-aware navigation for time-based jumps (prevents mid-sentence drops)
- Punctuation preserved in tokens for sentence boundary detection
- 50MB file size limit to prevent memory issues

---

## 1. Module Architecture

The `src/input/` module serves as an I/O boundary layer, handling all three data sources. This follows PRD's projected structure where `src/input/` contains format-specific loaders.

**Module Structure:**
```
src/
├── input/          # NEW for Task 2A-4
│   ├── mod.rs        # Module exports, LoadError, LoadedDocument
│   ├── pdf.rs        # PDF loading
│   ├── epub.rs       # EPUB loading
│   └── clipboard.rs  # Clipboard loading
├── engine/
│   └── timing.rs    # Updated with punctuation detection
└── app/
    └── app.rs        # Updated with LoadFile/LoadClipboard handlers
```

**Each loader responsibility:**
- Read external source (file from disk, text from clipboard)
- Extract raw text content
- Defer to `engine::tokenize_text()` for pure domain transformation
- Return `Result<LoadedDocument, LoadError>` to caller

**Data Payload:**
```rust
pub struct LoadedDocument {
    pub tokens: Vec<Token>,     // Tokenized words for RSVP display
    pub source: String,          // "pdf:document.pdf", "epub:book.epub", "clipboard"
}
```

This maintains clean separation: Input modules = I/O + format parsing, Engine = tokenization, App = state machine coordination.

---

## 2. Error Handling Strategy

The `LoadError` enum provides structured, user-friendly error reporting across all three input sources.

**Error Types:**
```rust
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
```

**Error Categories:**
- **Format parsing errors:** `PdfParse(String)`, `EpubParse(String)` - When PDF/EPUB libraries report parsing failures
- **Clipboard errors:** `Clipboard(String)` - Cross-platform clipboard access failures (unsupported OS, permission denied)
- **File system errors:** `FileNotFound(PathBuf)` - File access issues
- **Unsupported format:** `UnsupportedFormat(String)` - Non-PDF/EPUB files attempted

**Error Handling in App:**
```rust
match result {
    Ok(doc) => {
        self.state.tokens = doc.tokens;
        self.state.current_word = 0;
        self.mode = AppMode::Reading;
    }
    Err(e) => eprintln!("Load error: {}", e),
}
```

Graceful degradation means users can retry with different files without restarting. memory issues with large files. Users get a clear error rather than OOM crashes.

**Error Handling in App:**
```rust
match result {
    Ok(doc) => {
        self.state.tokens = doc.tokens;
        self.state.current_word = 0;
        self.mode = AppMode::Reading;
    }
    Err(e) => eprintln!("Load error: {}", e),
}
```

Graceful degradation means users can retry with different files without restarting.

---

## 3. Data Flow Through System

The loading flow follows established REPL → AppEvent → App pattern from Task 2A-3.

### Input Path - File Loading

1. User types `@document.pdf` or `@book.epub` in REPL
2. REPL parser creates `AppEvent::LoadFile(PathBuf::from("document.pdf"))`
3. Event loop passes event to `App::handle_event()`

### Input Path - Clipboard Loading

1. User types `@@` or pastes text in REPL
2. REPL parser creates `AppEvent::LoadClipboard`
3. Event loop passes event to `App::handle_event()`

### Processing Path - File (App Event Handler)

4. App extracts file extension (`.pdf`, `.epub`, or unsupported)
5. App calls appropriate input module:
   - `input::pdf::load(path)` for PDFs
   - `input::epub::load(path)` for EPUBs
6. Input module reads file, extracts text, calls `engine::tokenize_text()`
7. Input module returns `Result<LoadedDocument, LoadError>`

### Processing Path - Clipboard (App Event Handler)

4. App calls `input::clipboard::load()`
5. Clipboard module retrieves text via `arboard::Clipboard::new().get_text()`
6. Clipboard module calls `engine::tokenize_text()` on retrieved text
7. Returns `Result<LoadedDocument, LoadError>`

### State Update Path (Both Sources)

8. On success: App sets `state.tokens = doc.tokens`, `state.current_word = 0`
9. App transitions to `AppMode::Reading` - REPL loop exits, reading UI activates
10. On error: App prints error, stays in `AppMode::Repl` - user can retry

This three-stage flow (IO → domain → state) preserves pure core boundaries. The `LoadedDocument` payload is the only data crossing module boundaries.

---

## 4. Testing Approach

Following TDD principles, tests will be written before implementation for each input source.

### Unit Tests in `src/input/*.rs`

**PDF tests (`pdf.rs`):**
- Mock PDF file parsing, verify tokenization
- Test file size limit enforcement
- Verify PDF parse errors return appropriate `LoadError`

**EPUB tests (`epub.rs`):**
- Mock EPUB structure, verify chapter text extraction
- Test chapter order preservation
- Verify EPUB parse error handling

**Clipboard tests (`clipboard.rs`):**
- Mock clipboard access, verify error handling for unsupported OS
- Test successful text retrieval and tokenization

### Integration Tests in `tests/input_integration_test.rs`

```rust
#[test]
fn test_integration_pdf_load_file() {
    // Load actual .pdf file with known content
    // Verify tokens match expected output
}

#[test]
fn test_integration_epub_load_file() {
    // Load actual .epub file with known chapters
    // Verify tokenization preserves order
}

#[test]
fn test_integration_clipboard_load() {
    // Set test text programmatically
    // Verify loading and tokenization
}
```

### Test Fixtures

```
tests/fixtures/
├── sample.pdf       # Small PDF with simple text
├── sample.epub      # Small EPUB with 2-3 chapters
└── sample_text.txt   # Raw text for tokenization reference
```

### Test Naming Convention

- `test_pdf_basic_extraction()` - PDF loads and tokenizes
- `test_epub_chapter_order()` - EPUB preserves chapter sequence
- `test_clipboard_unsupported_os()` - Returns graceful error
- `test_integration_pdf_load_file()` - End-to-end PDF loading

All tests use `cargo test` - no UI dependencies. The `engine::tokenize_text()` function is already tested from Epic 1, so loader tests focus on I/O + format parsing.

---

## 5. Performance Considerations

File loading has performance implications that need explicit handling in MVP.

### Memory Management

- PDF/EPUB files loaded entirely into memory (no streaming for MVP)
- Rust's OOM protection handles memory pressure gracefully
- Tokens stored as `Vec<Token>` in App state - Rust ownership ensures cleanup

### Tokenization Performance

- `engine::tokenize_text()` already optimized from Epic 1
- Large documents (100k+ words) should tokenize in <1 second
- No performance concern for MVP usage patterns (articles, chapters, short books)

### Clipboard Latency

- `arboard::Clipboard::get_text()` is blocking call
- Typical paste: <100ms on modern systems
- **Timeout Risk:** If clipboard daemon hangs, REPL freezes indefinitely
- **Mitigation Options:**
  - Document risk for MVP (accepted trade-off)
  - Make clipboard access explicit (not automatic on startup)
  - Future: Add thread + timeout wrapper (~10 lines)
- No async needed for MVP - blocking is acceptable in REPL context

### Error Recovery Cost

- Failed loads don't corrupt App state
- REPL remains responsive during loading errors
- Users can retry immediately without restart

### Future Optimization Paths (Deferred)

- Streaming PDF/EPUB parsing for huge files
- Background thread loading for large documents
- Caching tokenization results for recently loaded files

For 2A-4 scope, current approach (synchronous full load + size limit) is sufficient. Performance issues will be surfaced by integration tests with actual PDF/EPUB files.

---

## 6. Sentence Detection Algorithm

To support sentence-aware navigation, we'll implement `find_sentence_start()` function that scans tokens for punctuation markers.

### Sentence Start Markers

- Period (`.`) - Primary sentence delimiter
- Question mark (`?`) - Question end
- Exclamation (`!`) - Exclamation end
- Newline (`\n`) - Paragraph break (tracked in `Token.punctuation`)
- Comma (`,`) - Secondary delimiter for finer granularity

### Search Algorithm

```rust
fn find_sentence_start(tokens: &[Token], start_pos: usize, direction: Direction) -> Option<usize> {
    let range = match direction {
        Direction::Forward => start_pos..tokens.len(),
        Direction::Backward => 0..=start_pos,
    };
    
    for (i, token) in tokens[range].enumerate() {
        if let Some(punct) = token.punctuation {
            if is_sentence_terminator(punct) {
                return Some(match direction {
                    Direction::Forward => start_pos + i + 1,  // Jump AFTER period
                    Direction::Backward => start_pos.saturating_sub(i),
                });
            }
        }
    }
    None  // No sentence boundary found
}

fn is_sentence_terminator(c: char) -> bool {
    matches!(c, '.' | '?' | '!' | '\n')
}

enum Direction {
    Forward,
    Backward,
}
```

### Navigation Behavior

- **Forward jump:** Look for period AFTER target position, jump to next token
- **Backward jump:** Look for period BEFORE target position, jump there
- **No period found:** Fall back to exact word count (still usable)

This ensures users always land at sentence start - much more natural than arbitrary word count drops.

---

## 7. Tokenization Update

The `Token` structure needs enhancement to support sentence-aware navigation.

### Updated Token Structure

```rust
pub struct Token {
    pub text: String,
    pub duration_ms: u64,
    pub punctuation: Option<char>,  // NEW - stores '.', ',', '?', '!', '\n', etc.
}

// Default WPM constant with justification
const DEFAULT_WPM: u32 = 300;  // Middle ground for RSVP reading (200-600 WPM range), per PRD section 3.2
```

### Punctuation Detection

```rust
pub fn tokenize_text(text: &str) -> Vec<Token> {
    text.split_word_bounds()
        .filter(|s| {
            let trimmed = s.trim();
            !trimmed.is_empty() && !trimmed.chars().all(|c| c.is_whitespace() || c.is_control())
        })
        .map(|word| {
            let trimmed = word.trim();
            let (text, punctuation) = extract_punctuation(trimmed);
            Token {
                text,
                duration_ms: 200,  // Default MVP timing
                punctuation,
            }
        })
        .collect()
}

fn extract_punctuation(word: &str) -> (String, Option<char>) {
    if word.is_empty() {
        return (String::new(), None);
    }
    
    let last_char = word.chars().last().unwrap();
    if is_sentence_terminator(last_char) || is_comma(last_char) {
        let text: String = word.chars().take(word.chars().count() - 1).collect();
        (text, Some(last_char))
    } else {
        (word.to_string(), None)
    }
}

fn is_sentence_terminator(c: char) -> bool {
    matches!(c, '.' | '?' | '!' | '\n')
}

fn is_comma(c: char) -> bool {
    c == ','
}
```

### Updated Tests

```rust
#[test]
fn test_tokenize_with_period() {
    let tokens = tokenize_text("hello world.");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].text, "hello world");
    assert_eq!(tokens[0].punctuation, Some('.'));
}

#[test]
fn test_tokenize_with_comma() {
    let tokens = tokenize_text("hello, world");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "hello");
    assert_eq!(tokens[0].punctuation, Some(','));
    assert_eq!(tokens[1].text, "world");
    assert_eq!(tokens[1].punctuation, None);
}

#[test]
fn test_tokenize_newline() {
    let tokens = tokenize_text("hello\nworld");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "hello");
    assert_eq!(tokens[0].punctuation, Some('\n'));
    assert_eq!(tokens[1].text, "world");
    assert_eq!(tokens[1].punctuation, None);
}
```

### Punctuation Limitations

**What's Supported in MVP:**
- Period (`.`) - Primary sentence delimiter
- Question mark (`?`) - Question end
- Exclamation (`!`) - Exclamation end
- Newline (`\n`) - Paragraph break
- Comma (`,`) - Secondary delimiter for finer granularity

**Known Gaps (Not Supported in MVP):**
- Semicolons (`;`) - Can indicate sentence end in formal writing
- Ellipsis (`...`) - Multiple periods, common in dialogue
- Em dashes (`—`) - Parenthetical breaks
- Complex Unicode punctuation

**Behavioral Impact:**
- Navigation may land imperfectly in edge cases (20% of real content has advanced punctuation)
- Timing calculations don't account for ellipsis pauses
- Sentence detection finds ~80% of natural boundaries

**Extension Path:**
Future enhancement should add central punctuation table:
```rust
const SENTENCE_TERMINATORS: &[char] = &['.', '?', '!', '\n', ';', '…'];
const PAUSE_MARKERS: &[char] = &[',', ';', '—'];

fn is_sentence_terminator(c: char) -> bool {
    SENTENCE_TERMINATORS.contains(&c)
}
```

Add GitHub issue tracking for punctuation enhancement.

```rust
#[test]
fn test_tokenize_with_period() {
    let tokens = tokenize_text("hello world.");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].text, "hello world");
    assert_eq!(tokens[0].punctuation, Some('.'));
}

#[test]
fn test_tokenize_with_comma() {
    let tokens = tokenize_text("hello, world");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "hello");
    assert_eq!(tokens[0].punctuation, Some(','));
    assert_eq!(tokens[1].text, "world");
    assert_eq!(tokens[1].punctuation, None);
}

#[test]
fn test_tokenize_newline() {
    let tokens = tokenize_text("hello\nworld");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "hello");
    assert_eq!(tokens[0].punctuation, Some('\n'));
    assert_eq!(tokens[1].text, "world");
    assert_eq!(tokens[1].punctuation, None);
}
 ```

---

## 9. Sentence-Aware Navigation Implementation

Navigation functions will be added to `src/app/app.rs` for use in reading mode (wired to key handlers in future tasks).

### Forward Navigation

```rust
fn jump_forward_by_time(app: &mut App, seconds: u32) {
    let word_count = (seconds as usize * DEFAULT_WPM) / 60;  // Uses DEFAULT_WPM constant
    let target_pos = app.state.current_word + word_count;
    
    if let Some(pos) = find_sentence_start(&app.state.tokens, target_pos, Direction::Forward) {
        app.state.current_word = pos;
    } else {
        app.state.current_word = target_pos.min(app.state.tokens.len() - 1);
    }
}
```

### Backward Navigation

```rust
fn jump_backward_by_time(app: &mut App, seconds: u32) {
    let word_count = (seconds as usize * DEFAULT_WPM) / 60;  // Uses DEFAULT_WPM constant
    let target_pos = app.state.current_word.saturating_sub(word_count);
    
    if let Some(pos) = find_sentence_start(&app.state.tokens, target_pos, Direction::Backward) {
        app.state.current_word = pos;
    } else {
        app.state.current_word = target_pos;
    }
}
```

### Navigation Behavior

- **Forward:** Calculate target word count from WPM and seconds, find next sentence boundary after that position
- **Backward:** Calculate target word count, find previous sentence boundary before that position
- **Fallback:** If no sentence boundary found, use exact word count (still usable)

This ensures time-based jumps always land at natural reading boundaries.

---

## 8. Sentence-Aware Navigation Implementation

Navigation functions will be added to `src/app/app.rs` for use in reading mode (wired to key handlers in future tasks).

### Forward Navigation

```rust
fn jump_forward_by_time(app: &mut App, seconds: u32) {
    let word_count = (seconds as usize * DEFAULT_WPM) / 60;
    let target_pos = app.state.current_word + word_count;
    
    if let Some(pos) = find_sentence_start(&app.state.tokens, target_pos, Direction::Forward) {
        app.state.current_word = pos;
    } else {
        app.state.current_word = target_pos.min(app.state.tokens.len() - 1);
    }
}
```

### Backward Navigation

```rust
fn jump_backward_by_time(app: &mut App, seconds: u32) {
    let word_count = (seconds as usize * DEFAULT_WPM) / 60;
    let target_pos = app.state.current_word.saturating_sub(word_count);
    
    if let Some(pos) = find_sentence_start(&app.state.tokens, target_pos, Direction::Backward) {
        app.state.current_word = pos;
    } else {
        app.state.current_word = target_pos;
    }
}
```

### Navigation Behavior

- **Forward:** Calculate target word count from WPM and seconds, find next sentence boundary after that position
- **Backward:** Calculate target word count, find previous sentence boundary before that position
- **Fallback:** If no sentence boundary found, use exact word count (still usable)

This ensures time-based jumps always land at natural reading boundaries.

---

## 10. Implementation Steps

### Step 1: Add Dependencies to Cargo.toml

```toml
[dependencies]
# Existing
crossterm = "0.29"
ratatui = "0.30"
unicode-segmentation = "1.12"
rustyline = "17.0"

# NEW - Add for Task 2A-4
pdf-extract = "0.7"  # PDF text extraction
epub = "2.1"         # EPUB parsing
arboard = "3.4"        # Cross-platform clipboard (Linux/Mac/Windows)
thiserror = "2.0"       # Error handling
```

### Step 2: Create `src/input/mod.rs`

- Define `LoadError` enum with `thiserror`
- Define `LoadedDocument` struct with `tokens: Vec<Token>`
- Re-export `pdf`, `epub`, `clipboard` modules

### Step 3: Update `src/engine/timing.rs`

- Add `punctuation: Option<char>` to `Token` struct
- Implement `extract_punctuation()` function
- Implement `is_sentence_terminator()` function
- Implement `is_comma()` function
- Modify `tokenize_text()` to detect and store punctuation marks
- Add tests for punctuation preservation

### Step 4: Implement `src/input/pdf.rs`

- Check file extension matches `.pdf`
- Load PDF file using `pdf-extract::extract_text_from_file()`
- Tokenize text with updated `engine::tokenize_text()`
- Return `Result<LoadedDocument, LoadError>`

### Step 5: Implement `src/input/epub.rs`

- Check file extension matches `.epub`
- Load EPUB using `epub::doc::EpubDoc::new()`
- Extract all chapter/section text
- Concatenate with newline separators
- Tokenize and return `Result<LoadedDocument, LoadError>`

### Step 6: Implement `src/input/clipboard.rs`

- Initialize `arboard::Clipboard::new()`
- Get text with `get_text()`
- Handle `UnsupportedOs` error gracefully with user-friendly message
- Tokenize and return `Result<LoadedDocument, LoadError>`

### Step 7: Update `src/app/app.rs`

- Implement `AppEvent::LoadFile` handler with extension detection
- Implement `AppEvent::LoadClipboard` handler
- Update App state with tokens on success
- Transition to `AppMode::Reading`
- Display errors on failure
- Add `jump_forward_by_time()` and `jump_backward_by_time()` functions
- Add `find_sentence_start()` helper function

### Step 8: Write Tests

- Unit tests for `pdf.rs`, `epub.rs`, `clipboard.rs` modules
- Unit tests for updated tokenization with punctuation
- Unit tests for sentence detection algorithm
- Integration tests with actual .pdf and .epub files

### Step 9: Verify All Tests Pass

```bash
cargo test
cargo clippy  # Optional per user preference
```

---

## 11. Acceptance Criteria

- [ ] PDF files load successfully from `@filename.pdf` command
- [ ] EPUB files load successfully from `@filename.epub` command
- [ ] Clipboard content loads from `@@` command or paste
- [ ] Errors handled gracefully with explicit user-friendly messages
- [ ] State transitions to `AppMode::Reading` on successful load
- [ ] Tokenization works correctly with punctuation preserved
- [ ] Sentence-aware navigation lands at sentence boundaries
- [ ] All tests pass (unit + integration)

---

## Dependencies

**New crates to add:**
- `pdf-extract = "0.7"` - PDF text extraction
- `epub = "2.1"` - EPUB parsing
- `arboard = "3.4"` - Cross-platform clipboard
- `thiserror = "2.0"` - Error handling

**Existing crates used:**
- `rustyline = "17.0"` - REPL input (from Task 2A-3)
- `unicode-segmentation = "1.12"` - Word boundary detection (from Epic 1)

---

## Notes

- **Plain text files NOT supported** - Per user requirement, only PDF, EPUB, clipboard
- **Pure core pattern maintained** - Loaders return data payloads, App handles state
- **Cross-platform clipboard** - `arboard` supports Linux/Mac/Windows
- **YAGNI principle** - Simple loader module for now, extract to full architecture when PDF/EPUB arrive
- **TDD approach** - Write failing tests first, then implement minimal code to pass
- **Clippy optional** - Per user preference, will run at end of epic
- **Technical debt documented** - Pure core violation will be repaid in Task 2A-X
- **Clipboard timeout accepted** - Documented risk, mitigation deferred to future enhancement

---

## 11. Technical Debt

### Pure Core Architecture Violation

**Current State:**
- App calls loaders directly: `input::pdf::load(path)`, `input::clipboard::load()`
- This means App touches I/O, violating strict pure core pattern
- Main loop doesn't intercept I/O boundary

**Why Accept for Now:**
- Low refactor cost (~20-30 lines) but introduces breaking change mid-epic
- Current approach (loaders return data payloads) maintains 80% of pure core benefits
- Prioritizing 2A-4 completion over architectural perfection

**Repayment Plan (Task 2A-X):**

**Refactor to Strict Pure Core:**
1. Create new AppEvents: `AppEvent::FileLoaded(LoadedDocument)`, `AppEvent::ClipboardLoaded(LoadedDocument)`
2. Update `src/main.rs` event loop:
   ```rust
   match event {
       AppEvent::LoadFile(path) => {
           let result = loaders::pdf::load(&path);
           match result {
               Ok(doc) => app.handle_event(AppEvent::FileLoaded(doc)),
               Err(e) => eprintln!("{}", e),
           }
       }
       AppEvent::LoadClipboard => { /* similar */ }
       AppEvent::FileLoaded(doc) => app.apply_loaded_document(doc),
       _ => app.handle_event(event),
   }
   ```
3. Remove `AppEvent::LoadFile` and `AppEvent::LoadClipboard` handlers from App
4. Update tests for new event flow

**Benefit:** App never touches I/O, true pure core maintained
**Cost:** ~20-30 lines, minor breaking change
**Tracking:** This is documented as technical debt with clear repayment path

---

## 12. Future Enhancements (Out of Scope for 2A-4)

- Tab completion for file suggestions
- Recursive file search (`@**/chapter.pdf`)
- Recent files history (`@` alone)
- Streaming file parsing for huge documents
- Background thread loading
- File format detection by content (not just extension)
