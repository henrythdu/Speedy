# Code Base Documentation

**Last Updated:** 2026-01-30 (input module reviewed)

## Overview

Speedy is a high-speed RSVP (Rapid Serial Visual Presentation) reader written in Rust. This document serves as a living audit of the codebase, tracking what's essential, what's potentially redundant, and what can be removed.

## Directory Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── app/                    # Application logic
│   ├── app.rs
│   ├── app_tests.rs
│   ├── event.rs
│   ├── mode.rs
│   ├── mod.rs
│   └── render_state.rs
├── audio/                  # Audio playback
│   └── mod.rs
├── engine/                 # Core engine logic
│   ├── config.rs
│   ├── error.rs
│   └── mod.rs
├── input/                  # Input file handling
│   ├── clipboard.rs
│   ├── epub.rs
│   ├── mod.rs
│   └── pdf.rs
├── reading/                # RSVP reading logic
│   ├── mod.rs
│   ├── ovp.rs
│   ├── state.rs
│   ├── timing.rs
│   └── token.rs
├── rendering/              # Rendering backends
│   ├── capability.rs
│   ├── font.rs
│   ├── kitty.rs            # Kitty Graphics Protocol (796 lines, recently refactored)
│   ├── mod.rs
│   ├── renderer.rs
│   └── viewport.rs
├── storage/                # Data persistence
│   └── mod.rs
└── ui/                     # User interface
    ├── command.rs
    ├── mod.rs
    ├── reader/
    │   ├── mod.rs
    │   └── view.rs
    ├── terminal_guard.rs
    ├── terminal.rs
    └── theme.rs
```

## Review Progress

| Module | Status | Notes |
|--------|--------|-------|
| main.rs | ✅ Reviewed | Essential - entry point (52 lines) |
| lib.rs | ✅ Reviewed | Essential - required for integration tests (9 lines) |
| app/ | ✅ Reviewed | Simplified - removed RenderState, app_tests.rs |
| audio/ | ✅ Stub | Planned feature (PRD Section 5) - 2 lines |
| engine/ | ✅ Reviewed | Simplified config.rs: 260→49 lines, error.rs: 87→27 lines |
| input/ | ✅ Reviewed | PDF/EPUB/Clipboard loading (352 lines) |
| reading/ | ⏳ Pending | 5 files |
| rendering/ | ⏳ Pending | 6 files (kitty.rs already reviewed) |
| storage/ | ✅ Stub | Planned feature (PRD Section 6.1) - 2 lines |
| ui/ | ⏳ Pending | 7 files |

## Decision Framework

For each file, we evaluate:
1. **Purpose:** What does this file do?
2. **Usage:** Is it actually used by other modules?
3. **Complexity:** Is it more complex than needed?
4. **Duplication:** Does functionality exist elsewhere?
5. **Test Coverage:** Are there tests? Are they valuable?

## Actions Taken

### 2026-01-30: Initial Code Base Audit
- Created this documentation file
- Began systematic file-by-file review
- Note: `src/rendering/kitty.rs` was already refactored from 1720→796 lines

### 2026-01-30: Simplified `app/` module
- **Removed `src/app/render_state.rs` (68 lines)** - Overengineered struct
- **Removed `src/app/app_tests.rs` (33 lines)** - Redundant tests
- **Added `get_current_word()` method** to `app.rs` - Simple extraction
- **Updated `terminal.rs`** to use `app.get_current_word()` directly
- **Removed `get_render_state()` method** from `app.rs`
- Updated tests in `app.rs` to use new method
- All 188 tests passing

### File Details

#### `src/main.rs` (52 lines)
**Purpose:** Application entry point
**Status:** ✅ KEEP - Essential

**What it does:**
1. Parses CLI args for `--force-kitty` flag
2. Detects terminal graphics capability
3. Loads font and prints metrics
4. Creates App and TuiManager instances
5. Runs TUI event loop

**Dependencies:** app, rendering::capability, rendering::font, ui
**Complexity:** Low - straightforward initialization
**Redundancy:** None

---

#### `src/engine/config.rs` (49 lines, reduced from 164→260→49)
**Purpose:** Timing configuration only
**Status:** ✅ REVIEWED & SIMPLIFIED

**What it is:**
- `TimingConfig` struct with WPM, punctuation multipliers, word length penalty
- All PRD Section 3.2 values correctly implemented
- Used by `reading/state.rs` to create ReadingState

**Removed (115 lines of dead code):**
- `ThemeConfig` - Future theming (not used)
- `GutterConfig` - Future progress bars (not used)
- `AudioConfig` - Future audio features (not used)
- `TactileConfig` - Future tactile controls (not used)
- `Config` master struct (combined unused configs)

**Remaining PRD Alignment:**
- ✅ Timing defaults match PRD exactly (WPM 300, multipliers, penalties)

**Dependencies:** std
**Complexity:** Low - well-structured
**Redundancy:** Fixed - removed 115 lines of unused future configs

---

#### `src/engine/error.rs` (27 lines, reduced from 87)
**Purpose:** Error handling enum
**Status:** ✅ REVIEWED & SIMPLIFIED

**What it is:**
- `SpeedyError` enum - IoError, EmptyFile
- Implements Display, Debug, Error traits
- Has `From<io::Error>` impl for IoError

**Removed (60 lines of dead code):**
- `load_file_safe()` function - unused utility function
- All 3 tests for load_file_safe
- `InvalidEncoding` variant - never constructed

**Used Code:**
- ✅ `SpeedyError` enum - kept for future error handling

**Dependencies:** std
**Complexity:** Very low - simple error enum
**Redundancy:** Fixed - removed 60 lines of test-only dead code

---

#### `src/input/mod.rs` (30 lines)
**Purpose:** Input module error types and document struct
**Status:** ✅ KEEP - Essential

**What it is:**
- `LoadError` enum - PdfParse, EpubParse, Clipboard, FileNotFound, UnsupportedFormat
- `LoadedDocument` struct - tokens Vec<String>, source String
- Module exports for pdf, epub, clipboard modules

**PRD Alignment:**
- ✅ Section 2.2 - PDF, EPUB, clipboard input formats
- ✅ Section 7.1 - @filename and @@ command support

**Dependencies:** std, thiserror
**Complexity:** Very low - standard error handling
**Redundancy:** None
**Used by:** app/app.rs (lines 4, 89-96, 110-113)

---

#### `src/input/pdf.rs` (95 lines)
**Purpose:** PDF file loading
**Status:** ✅ KEEP - Essential

**What it does:**
- Loads PDF files using `pdf-extract` crate
- Checks file existence
- Extracts text from PDF memory buffer
- Tokenizes text via `engine::tokenize_text()`
- Returns `LoadedDocument` with "pdf:" source prefix

**Note:** `tokenize_text()` is defined in `reading/timing.rs` and re-exported through `engine/mod.rs`

**PRD Alignment:**
- ✅ Section 2.2 - PDF support (@filename.pdf command)
- ✅ Section 7.1 - File loading command

**Dependencies:** std, pdf-extract, engine::tokenize_text
**Complexity:** Low - straightforward file I/O and parsing
**Redundancy:** None
**Tests:** 4 tests pass (file not found, source field, tokenization, error types)

---

#### `src/input/epub.rs` (155 lines)
**Purpose:** EPUB file loading
**Status:** ✅ KEEP - Essential with known limitation

**What it does:**
- Loads EPUB files using `epub` crate
- Checks file existence
- Iterates through all chapters
- Extracts plain text from HTML content (lines 60-82)
- Concatenates chapters with double newlines
- Tokenizes text via `engine::tokenize_text()`
- Returns `LoadedDocument` with "epub:" source prefix

**HTML Extraction Implementation:**
```rust
// Simple tag removal - no entity decoding
fn extract_plain_text(html: &str) -> String {
    // Removes <tag> markers
    // Cleans up whitespace
    // Does NOT decode HTML entities (&nbsp;, &amp;, etc.)
    // Does NOT remove <script>, <style>, <!-- comments -->
}
```

**Known Limitations:**
- ❌ HTML entities not decoded (e.g., `&nbsp;` → literal text, not space)
- ❌ Script/style tag content may appear in reading text
- ❌ HTML comments extracted as text
- **Impact:** Most EPUBs work; some may show weird characters or code snippets
- **Future Enhancement:** Add `html2text` crate or implement minimal entity decoding when issues arise

**PRD Alignment:**
- ✅ Section 2.2 - EPUB support (@filename.epub command)
- ✅ Section 7.1 - File loading command

**Dependencies:** std, epub, engine::tokenize_text
**Complexity:** Low - straightforward file parsing
**Redundancy:** None
**Tests:** 5 tests pass (file not found, source field, sentence boundaries, parse error, HTML extraction)

---

#### `src/input/clipboard.rs` (72 lines)
**Purpose:** System clipboard loading
**Status:** ✅ KEEP - Essential

**What it does:**
- Loads clipboard content using `arboard` crate
- Tokenizes text via `engine::tokenize_text()`
- Returns `LoadedDocument` with "clipboard" source
- Handles clipboard errors gracefully

**PRD Alignment:**
- ✅ Section 2.2 - Clipboard support (@@ command)
- ✅ Section 7.1 - REPL clipboard loading

**Dependencies:** arboard, engine::tokenize_text
**Complexity:** Low - single API call to clipboard
**Redundancy:** None
**Tests:** 3 tests pass (load success, source field, punctuation preservation)
**Used by:** app/app.rs `handle_load_clipboard()` (lines 110-113)

---


**What it is:**
- `TimingConfig` struct with WPM, punctuation multipliers, word length penalty
- All PRD Section 3.2 values correctly implemented
- Used by `reading/state.rs` to create ReadingState

**Removed (115 lines of dead code):**
- `ThemeConfig` - Future theming (not used)
- `GutterConfig` - Future progress bars (not used)
- `AudioConfig` - Future audio features (not used)
- `TactileConfig` - Future tactile controls (not used)
- `Config` master struct (combined unused configs)
- All commented out with Phase references

**Remaining PRD Alignment:**
- ✅ Timing defaults match PRD exactly (WPM 300, multipliers, penalties)
- ⏳ Other configs are for Phase 2+ - removed and documented in comments

**Dependencies:** std
**Complexity:** Low - well-structured
**Redundancy:** Fixed - removed 115 lines of unused future configs

---


**What it does:**
- Exports App, AppEvent, AppMode
- Removed RenderState export (no longer needed)

**Redundancy:** None

---

#### `src/app/app.rs` (~480 lines, reduced from 553)
**Purpose:** Main application logic
**Status:** ✅ KEEP - Essential, simplified

**What it does:**
- App struct with mode and reading_state
- File loading (PDF, EPUB, clipboard)
- Keypress handling (j/k navigation, WPM, pause)
- **NEW:** `get_current_word()` - simple extraction method
- **REMOVED:** `get_render_state()` - overengineered
- Tests: heavily tested (inline, ~200 lines)

**Dependencies:** app::event, app::mode, engine, input
**Complexity:** Medium - app logic is reasonable
**Redundancy:** Fixed - removed overengineered RenderState

---

#### `src/app/event.rs` (12 lines)
**Purpose:** Application event enum
**Status:** ✅ KEEP - Essential

**What it does:**
- Defines AppEvent enum (LoadFile, LoadClipboard, Quit, Help, etc.)
- Used by app.handle_event()

**Dependencies:** None
**Complexity:** Minimal
**Redundancy:** None

---

#### `src/engine/error.rs` (87 lines)
**Purpose:** Error handling + utility function
**Status:** ⚠️ REVIEW - Has dead code

**What it does:**
1. `SpeedyError` enum - IoError, EmptyFile, InvalidEncoding
2. Implements Display, Debug, Error traits
3. `load_file_safe()` function - reads file and validates not empty

**Dead Code Found:**
- ❌ `load_file_safe()` is ONLY used in its own tests (lines 51, 67, 81)
- ❌ Nowhere else in codebase calls `load_file_safe()`
- ❌ `InvalidEncoding` variant is never constructed

**Used Code:**
- ✅ `SpeedyError::IoError` - Has `From<io::Error>` impl (line 28-32)
- ✅ But no code actually returns `SpeedyError` - it's unused!

**Complexity:** Low - straightforward
**Redundancy:** ~70% of this file (60+ lines) is test-only dead code

**Question:** Should we:
1. Keep `SpeedyError` even though unused?
2. Remove `load_file_safe()` entirely (function + tests)?
3. Or wait until we need actual error handling?

---


**What it is:**
- Comment: "Audio module - stub for future epic"
- Empty module body
- Declared in lib.rs and main.rs

**PRD Reference:** Section 5 - AUDITORY & KINESTHETIC LAYERS
- Auditory Metronome (paragraph "thump", speed glide)
- Tactile Controls (tab-peek, tactical throttle)
- Audio profiles (Minimal/Subtle/Pronounced)

**Rationale:** Keep as placeholder for Phase 4+ implementation. Will add audio playback when implementing these features.

---

#### `src/storage/mod.rs` (2 lines)
**Purpose:** Stub for future storage epic (PRD Section 6.1)
**Status:** ✅ KEEP - Planned feature

**What it is:**
- Comment: "Storage module - stub for future epic"
- Empty module body
- Declared in lib.rs and main.rs

**PRD Reference:** Section 6.1 - Project Structure shows `storage/history.rs`
- Recent files history
- Reading position persistence

**Rationale:** Keep as placeholder for future persistence features. Will add history.rs when implementing file history and position saving.

---


**What it does:**
- Defines AppMode enum (Command, Reading, Paused, Peek, Quit)
- Used throughout app and ui modules
- Has Default impl (starts in Command mode)

**Dependencies:** None
**Complexity:** Minimal
**Redundancy:** None

---


**What it does:**
- Exports all modules as public
- Enables `use speedy::*` in integration tests

**Used by:** `tests/integration_test.rs` (7 tests, all passing)
**Complexity:** Minimal
**Redundancy:** None
**Note:** Even though Speedy is a binary, lib.rs is needed for integration tests

---



