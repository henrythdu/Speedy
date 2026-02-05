# Refactoring Plan: Rendering & UI Cleanup

**Date:** 2026-02-03  
**Status:** ✅ Consensus Approved - Ready for Implementation  
**Priority:** High  
**Consensus Confidence:** High (9/10, 8/10 across models)  
**Workflow:** Refactor → `cargo check` → `pal codereview` → Update if needed

---

## Overview

This refactoring addresses technical debt accumulated during rapid bug fixes for word skipping and punctuation rendering. The goal is to clean up production code, remove debug infrastructure, and improve code quality while maintaining existing functionality.

## Scope

### In Scope ✅
1. Remove debug logging infrastructure from production code
2. Centralize magic numbers into existing config system
3. Fix token skipping at the tokenizer level (remove workaround)
4. Eliminate code duplication in token-to-string conversion
5. Extract helper methods from monolithic event loop

### Out of Scope ❌
- Word cache implementation (future feature)
- Ghost word UI functions (future feature)
- Renderer abstraction layer (future architectural change)
- Error handling standardization (separate refactoring)

---

## Consensus Review Summary

**Overall Confidence:** High  
**Models Consulted:** 3 (gemini-pro: 9/10, gpt-5.1-codex, claude-opus-4.5: 8/10)

### Key Agreements
- ✅ Scope is appropriate and boundaries are well-defined
- ✅ Implementation order is sound (config → tokenizer → dedup → cleanup)
- ✅ Tokenizer fix is architecturally correct (move from consumer to producer)
- ✅ Technical feasibility is high (standard Rust patterns)

### Key Recommendations Applied
1. **Use Display trait** (not custom to_string()) for Token - more idiomatic Rust
2. **Add tokenizer tests BEFORE removing workaround** - critical regression safety
3. **Run tests after tokenizer fix** - not just final review
4. **Consider swap order** - Debug removal before deduplication for cleaner diffs
5. **Watch borrow checker** - Event loop extraction may need field-level parameters
6. **Document magic number origins** - Add comments explaining values

---

## Detailed Changes

### 1. Remove Debug Infrastructure

**File:** `src/ui/terminal.rs`

**Current Issues:**
- `log_file: std::fs::File` field in TuiManager struct (line 27-30)
- File I/O operations in hot render loop (lines 288-330)
- Hard-coded path `/tmp/speedy_debug.log` (line 59)
- Debug counter fields: `last_render_idx`, `advance_counter`

**Changes:**
```rust
// REMOVE from TuiManager struct:
// - log_file: std::fs::File
// - last_render_idx: usize  
// - advance_counter: u32

// REMOVE from new():
// - Log file creation logic

// REMOVE from render_frame():
// - All writeln!() calls to log_file
// - Index tracking logic

// REMOVE from event loop:
// - ADVANCE logging in auto-advance section
```

**Impact:** Cleaner code, no file I/O in render loop, no hard-coded paths

**Consensus Note:** Should be done before deduplication for cleaner diffs

---

### 2. Centralize Magic Numbers in Config

**File:** `src/engine/config.rs` (extend existing)

**Current Issues:**
- Magic numbers scattered across 10+ locations
- No single source of truth

**Changes:**
```rust
// ADD to config.rs with documentation:
pub const COMMAND_DECK_LINES: u16 = 5;  // Fixed 5-line command deck per PRD Section 4.3
pub const READING_ZONE_CENTER_PCT: f32 = 0.42;  // 42% of reading zone height for OVP per PRD
pub const KITTY_CHUNK_SIZE: usize = 4096;  // Kitty protocol max chunk size
pub const DEFAULT_WPM: u32 = 300;  // Default reading speed per PRD Section 3.2
pub const MIN_TERMINAL_COLS: u16 = 80;  // Minimum supported terminal width
pub const MIN_TERMINAL_ROWS: u16 = 24;  // Minimum supported terminal height
pub const FONT_SIZE_MULTIPLIER: f32 = 5.0;  // Font size = cell_height * 5 per design
pub const RENDER_FPS: u32 = 60;  // Target render rate for smooth UI
pub const FONT_SIZE_FALLBACK: f32 = 24.0;  // Fallback font size if viewport query fails

// ADD new struct:
pub struct UiConfig {
    pub command_deck_lines: u16,
    pub reading_zone_center_pct: f32,
    pub render_fps: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            command_deck_lines: COMMAND_DECK_LINES,
            reading_zone_center_pct: READING_ZONE_CENTER_PCT,
            render_fps: RENDER_FPS,
        }
    }
}
```

**Files to Update:**
- `src/ui/terminal.rs`: Use `COMMAND_DECK_LINES`, `DEFAULT_WPM`, `RENDER_FPS`
- `src/rendering/kitty.rs`: Use `FONT_SIZE_MULTIPLIER`, `KITTY_CHUNK_SIZE`
- `src/rendering/viewport.rs`: Use `MIN_TERMINAL_COLS`, `MIN_TERMINAL_ROWS`

---

### 3. Fix Token Skipping at Tokenizer Level ⭐ CRITICAL

**Files:** 
- `src/engine/tokenizer.rs` (primary fix)
- `src/ui/terminal.rs` (remove workaround)

**Current Issue:**
- Empty/newline tokens cause blank screens
- Workaround in terminal.rs event loop auto-skips them

**Root Cause:**
Tokenizer produces empty tokens or whitespace-only tokens that shouldn't be in the token stream.

**Changes in tokenizer.rs:**
```rust
// In tokenization logic, FILTER OUT:
// - Empty strings ("")
// - Whitespace-only strings (" ", "\n", "\t")
// BEFORE adding to tokens vector

// Example fix location (find exact function):
// After splitting text, before pushing to tokens:
if !word.trim().is_empty() {
    tokens.push(Token { ... });
}
```

**Changes in terminal.rs:**
```rust
// REMOVE the auto-skip logic (lines ~213-233):
// while advanced {
//     if let Some(word) = app.get_current_word() {
//         if word.trim().is_empty() { ... }
//     }
// }

// REPLACE with simple:
if !app.advance_reading() {
    app.set_mode(AppMode::Paused);
}
```

**⚠️ CRITICAL:** Write unit tests for tokenizer BEFORE removing workaround (see Testing Strategy)

---

### 4. Eliminate Token-to-String Duplication

**Files:**
- `src/engine/token.rs` (add method)
- `src/ui/terminal.rs` (use method)
- `src/app/app.rs` (use method)

**Current Issue:**
Same logic appears in 3 places:
1. `terminal.rs` lines 127-138 (LoadFile)
2. `terminal.rs` lines 149-160 (LoadClipboard)
3. `app.rs` lines 135-145 (get_current_word)

**Changes in token.rs:**
```rust
impl Token {
    /// Convert token to string with punctuation attached
    pub fn to_string(&self) -> String {
        let mut s = self.text.clone();
        for p in &self.punctuation {
            s.push(*p);
        }
        s
    }
}

// Also implement Display trait for convenience (consensus recommendation):
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
```

**Changes in terminal.rs:**
```rust
// REPLACE in LoadFile and LoadClipboard:
// let text: String = doc.tokens.iter().map(|t| { ... }).collect();
// app.start_reading(&text, 300);

// WITH:
app.apply_loaded_document(doc);
```

**Changes in app.rs:**
```rust
// REPLACE in get_current_word():
// .map(|t| {
//     let mut word = t.text.clone();
//     for p in &t.punctuation { word.push(*p); }
//     word
// })

// WITH:
.map(|t| t.to_string())
```

---

### 5. Extract Helper Methods ⚠️ BORROW CHECKER WARNING

**File:** `src/ui/terminal.rs`

**Current Issue:**
Monolithic `run_event_loop()` method (~185 lines) mixes concerns.

**Changes:**
```rust
impl TuiManager {
    // EXTRACT method for auto-advance logic:
    // ⚠️ WARNING: May need to pass fields instead of &mut self to avoid borrow issues
    fn handle_auto_advance(&self, app: &mut App) {
        if !app.advance_reading() {
            app.set_mode(AppMode::Paused);
        }
    }
    
    // EXTRACT method for command execution:
    fn execute_command(&mut self, command: &str, app: &mut App) -> io::Result<()> {
        use crate::ui::command::{parse_command, Command};
        match parse_command(command) {
            Command::LoadFile(path) => { /* ... */ }
            Command::LoadClipboard => { /* ... */ }
            Command::Quit => { /* ... */ }
            _ => Ok(())
        }
    }
}
```

**Consensus Note:** Event loop extraction can be tricky due to borrow checker. May need to pass specific fields as arguments rather than `&mut self`.

---

## Testing Strategy

### Required Tests (Per Consensus)

1. **Tokenizer Unit Tests** ⭐ MANDATORY BEFORE step 3
   - Test that tokenizer filters empty strings
   - Test that tokenizer filters whitespace-only strings
   - Test punctuation attachment works correctly
   - Test edge cases: multiple spaces, newlines, tabs

2. **Unit Tests:** Ensure existing tests still pass after each phase

3. **Integration Test:** Load a PDF and verify:
   - No words skipped
   - Punctuation appears correctly
   - WPM timing works
   - Command deck renders properly

4. **Edge Cases:**
   - Empty documents
   - Documents with many newlines
   - Very long words
   - Unicode characters

---

## Implementation Order (Updated Per Consensus)

1. **Phase 1: Config & Constants** (safest)
   - Extend `src/engine/config.rs`
   - Replace magic numbers in all files
   - Run `cargo check`

2. **Phase 2: Debug Removal** (consensus: swap order for cleaner diffs)
   - Remove debug fields and logic from TuiManager
   - Run `cargo check`

3. **Phase 3: Tokenizer Fix** ⭐ CRITICAL (foundational)
   - Fix tokenizer to filter empty tokens
   - **Write unit tests FIRST**
   - Run `cargo test` to verify tokenizer
   - Remove auto-skip from terminal.rs
   - Run `cargo check`

4. **Phase 4: Code Deduplication** (cleanup)
   - Add `Token::to_string()` and `Display` trait
   - Update terminal.rs and app.rs
   - Run `cargo check`

5. **Phase 5: Helper Methods** (cleanup)
   - Extract methods from event loop
   - ⚠️ Watch borrow checker issues
   - Run `cargo check`

6. **Phase 6: Review**
   - Run full test suite
   - `pal codereview` on changes
   - Address feedback

---

## Risk Assessment (Updated Per Consensus)

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking RSVP timing | Low | High | Thorough testing of WPM timing after each phase |
| Tokenizer regression | Medium | Medium | **Write tokenizer tests BEFORE removing workaround** |
| Config import errors | Low | Low | Compile-time checks |
| Borrow checker issues in event loop | Medium | Low | Pass fields instead of &mut self if needed |
| Ghost words break | Low | Low | Visual verification |

---

## Success Criteria

- ✅ All existing tests pass
- ✅ New tokenizer tests pass
- ✅ No debug logging in production code
- ✅ No magic numbers in source (all in config)
- ✅ Tokenizer produces no empty tokens
- ✅ No code duplication in token conversion
- ✅ `cargo check` produces no warnings
- ✅ Manual test: 1-minute reading session works smoothly

---

## Notes

- Word cache and ghost word features are explicitly out of scope
- Keep changes focused on cleanup, not feature additions
- Preserve all existing behavior while improving code quality
- **Consensus strongly supports this plan** - proceed with confidence
