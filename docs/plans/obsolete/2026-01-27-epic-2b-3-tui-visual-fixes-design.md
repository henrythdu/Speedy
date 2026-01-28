# Epic 2B-1.2: TUI Fixes and Integrated REPL

## Overview

Fixes critical TUI rendering issues and restructure REPL to be integrated within TUI (like opencode), rather than exiting to separate terminal mode.

## Issues to Address

### Critical Bugs

1. **OVP Anchor Moving Around** (HIGH)
   - **Issue**: Anchor character moves horizontally instead of staying fixed at visual center
   - **Root Cause**: `render_word_display()` uses `Alignment::Center` which fights with calculated padding
   - **Fix**: Change to `Alignment::Left` so padding calculation works correctly

2. **Context Words Not Dimmed** (HIGH)
   - **Issue**: 3 words before and after current word are extremely visible
   - **Fix**: Change dimmed color from RGB(100, 110, 150) to RGB(50, 60, 90) for ~20% opacity

3. **Progress Bar Not Dimmed Correctly** (MEDIUM)
   - **Issue**: Progress bar doesn't appear dimmed enough
   - **Fix**: Filled = text color, Empty = dimmed color, placed directly under current word (1px tall)

4. **Gutter Not Showing Progress** (MEDIUM)
   - **Issue**: Gutter is just a placeholder, not showing document progress
   - **Fix**: Implement full-height gutter with top-down progress filling, bookmark line

### Architecture Change

5. **Integrated REPL** (HIGH - Major Change)
   - **Issue**: REPL is separate terminal mode that exits TUI, breaking immersion
   - **Root Cause**: Current architecture has TUI and REPL as separate modes that exit/enter each other
   - **User Requirement**: REPL should be a panel within TUI (like opencode's bottom bar), always visible but only interactive when not in reading mode

## Detailed Design

### Section 1: OVP Anchor Fix

**Current Implementation Problem:**
```rust
// src/ui/render.rs:41
Paragraph::new(Line::from(spans))
    .alignment(Alignment::Center)  // ← PROBLEM
    .style(Style::default().bg(colors::background()))
```

We calculate left padding to position anchor at visual center:
```rust
let left_padding = (word_len / 2).saturating_sub(anchor_pos);
for _ in 0..left_padding {
    spans.push(Span::styled(" ", ...));
}
```

But then `Alignment::Center` centers ENTIRE paragraph, undoing our positioning.

**Fix:**
Change to `Alignment::Left` so padding calculation determines position:
```rust
Paragraph::new(Line::from(spans))
    .alignment(Alignment::Left)  // Fixed
    .style(Style::default().bg(colors::background()))
```

This ensures anchor character appears at consistent visual coordinate regardless of word length.

---

### Section 2: Integrated REPL Architecture

**Current architecture**: When you press 'q' in reading mode, it exits TUI entirely and goes back to terminal REPL. This breaks immersion.

**New approach**: REPL is always part of TUI interface - a bottom panel that shows:
- When in **Command Mode**: Interactive input field where you type `@file`, `@@`, `:h`, `:q`
- When in **Reading Mode**: Dimmed text like "Press 'q' for commands" (not interactive)

**Key changes**:
1. TUI never exits until user quits app (`AppMode::Quit`)
2. AppMode::Repl now means "in command panel" not "exit TUI"
3. User types commands in bottom panel, presses Enter to execute
4. Commands (`@file`, `@@`) switch to Reading Mode
5. In Reading Mode, 'q' switches back to Command Mode (not exit TUI)

**Flow example**:
```
User starts app → TUI shows command panel with help text
User types "@ test.pdf" + Enter → TUI switches to reading mode
User reads, presses 'q' → TUI switches to command mode (not exit)
User types "@ other.pdf" + Enter → TUI switches to reading mode again
User types ":q" + Enter → App quits
```

**AppMode states**:
- `Command`: User typing commands (`@file`, `@@`, `:h`, `:q`) in bottom panel
- `Reading`: RSVP display active, word auto-advancing, command panel dimmed
- `Paused`: RSVP display frozen, no auto-advancement, command panel dimmed showing "Press 'q' for commands"
- `Quit`: App exit

**Mode Transitions**:
- Start → Check for saved document → If yes: Reading mode, if no: Command mode
- Command mode + Enter on `@file` or `@@` → Load document → Reading mode
- Reading/Paused + `q` → Command mode (reading area still visible but dimmed)
- Command mode + `:q` → Quit

---

### Section 3: Layout Structure

**Revised Layout:**

```
┌────────────────────────────────────────┬──────────┐
│                                    │ ██░░░░░░ │  Gutter (full height)
│            hello                  │ ██░░░░░░ │  - Vertical progress bar
│            ───────               │ ━━━━━━━━━ │  - Bookmark line
│                                    │ ██░░░░░░ │  - All dimmed in reading mode
│                                    │ ██░░░░░░ │
├────────────────────────────────────────┴──────────┤
│ Speedy RSVP Reader | @@ (copy clipboard) | :h help │
│ Last: test.pdf (150 words) at position 42/150    │
└───────────────────────────────────────────────────────┘
```

**Layout Details:**

**Main Display Area** (90% of screen height):
- **When in Reading Mode**:
  - Current word (OVP-centered) in text color
  - 3 words before/after in dimmed color (20% opacity)
  - 1px progress bar under current word (subtle presence)

- **When in Command Mode**:
  - Reading display still visible but dimmed
  - Line 1: Help text - "Speedy RSVP Reader | @@ (copy clipboard) | :h help | :q quit"
  - Line 2: Last loaded document info - "Last: test.pdf (150 words) at position 42/150"

**Gutter** (Right side, ~3-5 characters wide):
- **Full vertical rectangle**: Extends from top to bottom of reading area
- **Top-down progress fill**: `██` = current position, `░░░` = remaining
- **Bookmark line**: Horizontal marker (`━━━`) at bookmark position (functionality for future)
- **Reading mode**: Entire gutter dimmed significantly (subtle, non-distracting)
- **Command mode**: Normal visibility

**Command Panel** (Bottom 2 lines):
- **Line 1**:
  - Command Mode: Help text (NO `speedy>` prefix)
  - Reading/Paused Mode: Dimmed "Press 'q' for commands"
- **Line 2**: Document info (file name, word count, position)

---

### Section 4: Gutter Implementation

**4.1 Data Structure**

Add to `App`:
```rust
pub struct App {
    mode: AppMode,
    reading_state: Option<ReadingState>,
    command_input: String,
    bookmark_position: Option<usize>,  // NEW: Bookmark position
}
```

**4.2 Gutter Rendering Function** (`src/ui/render.rs`)

```rust
pub fn render_gutter(
    tokens_len: usize,
    current_index: usize,
    bookmark: Option<usize>,
    mode: AppMode,
) -> Paragraph<'static> {
    if tokens_len == 0 {
        return Paragraph::new("│")
            .alignment(Alignment::Right)
            .style(Style::default().fg(colors::dimmed()).bg(colors::background()));
    }

    let progress_height = (current_index as f64 / tokens_len as f64 * 100.0) as usize;

    let mut spans = Vec::new();
    let fill_style = if mode == AppMode::Reading || mode == AppMode::Paused {
        Style::default().fg(colors::dimmed())
    } else {
        Style::default().fg(colors::text())
    };

    // Progress fill (top-down)
    for i in 0..100 {
        if i < progress_height {
            spans.push(Span::styled("█", fill_style));
        } else {
            spans.push(Span::styled("░", Style::default().fg(colors::dimmed())));
        }

        // Insert bookmark line if at position
        if let Some(bookmark_pos) = bookmark {
            let bookmark_height = (bookmark_pos as f64 / tokens_len as f64 * 100.0) as usize;
            if i == bookmark_height {
                // Replace character with bookmark marker
                spans.pop();
                spans.push(Span::styled("━", Style::default().fg(colors::dimmed())));
            }
        }

        spans.push(Span::raw("\n"));
    }

    Paragraph::new(Line::from(spans)).alignment(Alignment::Left)
}
```

**4.3 Render State Extension**

Update `RenderState` to include bookmark:
```rust
pub struct RenderState {
    pub mode: AppMode,
    pub current_word: Option<String>,
    pub tokens: Vec<Token>,
    pub current_index: usize,
    pub bookmark_position: Option<usize>,
    pub progress: (usize, usize),
    // ... other fields
}
```

---

### Section 5: Command Input & Mode Transitions

**5.1 Command Input Handling**

Add to `App`:
```rust
pub struct App {
    mode: AppMode,
    reading_state: Option<ReadingState>,
    command_input: String,           // Current command being typed
    bookmark_position: Option<usize>,   // Bookmark position
}

impl App {
    pub fn handle_command_char(&mut self, c: char) -> bool {
        // Only accept input in Command mode
        if !matches!(self.mode, AppMode::Command) {
            return false;
        }

        match c {
            '\n' | '\r' => {
                // Enter: execute command
                self.execute_command();
                true
            }
            '\x08' | '\x7f' => {
                // Backspace: remove last character
                self.command_input.pop();
                true
            }
            '\x03' => {
                // Ctrl+C: quit app
                self.mode = AppMode::Quit;
                true
            }
            _ if c.is_ascii() && !c.is_control() => {
                // Regular character: append to input
                self.command_input.push(c);
                true
            }
            _ => false,
        }
    }

    pub fn execute_command(&mut self) {
        let input = self.command_input.trim();
        self.command_input.clear();

        if input.is_empty() {
            return;
        }

        // Parse and execute (reuse existing logic)
        let event = parse_repl_input(input);
        self.handle_event(event);

        // If document loaded, switch to reading mode
        if matches!(self.mode, AppMode::Reading) {
            // Mode already set by handle_event
        }
    }
}
```

**5.2 TUI Event Loop Update**

Modify `TuiManager::run_event_loop()`:
```rust
pub fn run_event_loop(&mut self, app: &mut App) -> io::Result<AppMode> {
    let mut last_tick = Instant::now();
    let render_tick = Duration::from_millis(1000 / 60);

    loop {
        let current_mode = app.mode();
        if current_mode == AppMode::Quit {
            return Ok(AppMode::Quit);
        }
        // Command, Reading, Paused all stay in TUI

        // Only poll for events in Command mode
        // Reading mode uses timing-based auto-advancement
        let poll_timeout = if matches!(current_mode, AppMode::Command) {
            Duration::from_millis(100)  // Fast response for typing
        } else {
            let wpm = app.get_wpm();
            let timeout_ms = wpm_to_milliseconds(wpm);
            Duration::from_millis(timeout_ms)
        };

        match event::poll(poll_timeout) {
            Ok(true) => {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.handle_command_char(c);
                        }
                        KeyCode::Ctrl(c) if c == 'c' => {
                            app.mode = AppMode::Quit;
                        }
                        _ => {}
                    }
                }
            }
            Ok(false) => {
                // Only auto-advance in Reading mode, not Paused
                if app.mode() == AppMode::Reading {
                    app.advance_reading();
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        if last_tick.elapsed() >= render_tick {
            self.render_frame(app)?;
            last_tick = Instant::now();
        }
    }
}
```

**5.3 Command Line Rendering**

Add `render_command_line()` (`src/ui/render.rs`):
```rust
pub fn render_command_line(
    input: &str,
    mode: AppMode,
    last_doc_info: &str,
) -> Paragraph<'static> {
    match mode {
        AppMode::Command => {
            // Line 1: Help text (no "speedy>" prefix)
            let help_text = "Speedy RSVP Reader | @@ (copy clipboard) | :h help | :q quit";

            // Line 2: Last document info
            Paragraph::new(vec![
                Line::from(help_text),
                Line::from(last_doc_info),
            ]).style(Style::default().fg(colors::text()).bg(colors::background()))
        }
        AppMode::Reading | AppMode::Paused => {
            // Line 1: Dimmed prompt
            let prompt = Line::from(Span::styled(
                "Press 'q' for commands",
                Style::default().fg(colors::dimmed()),
            ));

            // Line 2: Document info (dimmed)
            let info = Line::from(Span::styled(
                last_doc_info,
                Style::default().fg(colors::dimmed()),
            ));

            Paragraph::new(vec![prompt, info])
        }
        _ => Paragraph::new(""),
    }
}
```

---

### Section 6: txt File Support

**6.1 Text File Loader** (`src/input/text.rs`)

Create new module for plain text file loading:
```rust
use std::fs;
use std::path::Path;
use crate::engine::timing::{tokenize_text, Token};
use crate::input::LoadedDocument;

pub fn load(path: &str) -> Result<LoadedDocument, LoadError> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(LoadError::FileNotFound(path_obj.to_path_buf()));
    }

    let content = fs::read_to_string(path_obj)
        .map_err(|e| LoadError::ParseError(format!("Failed to read file: {}", e)))?;

    if content.trim().is_empty() {
        return Err(LoadError::ParseError("File is empty".to_string()));
    }

    let tokens = tokenize_text(&content);

    if tokens.is_empty() {
        return Err(LoadError::ParseError("No readable content found".to_string()));
    }

    Ok(LoadedDocument {
        tokens,
        source: path.to_string(),
    })
}
```

**6.2 Update File Handler** (`src/app/app.rs`)

```rust
fn handle_load_file(&mut self, path: &str) {
    let path_obj = Path::new(path);

    let ext = path_obj
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("pdf") => match pdf::load(path.to_str().unwrap_or("")) {
            Ok(doc) => self.apply_loaded_document(doc),
            Err(e) => self.handle_load_error(&e),
        },
        Some("epub") => match epub::load(path.to_str().unwrap_or("")) {
            Ok(doc) => self.apply_loaded_document(doc),
            Err(e) => self.handle_load_error(&e),
        },
        Some("txt") => match text::load(path.to_str().unwrap_or("")) {  // NEW
            Ok(doc) => self.apply_loaded_document(doc),
            Err(e) => self.handle_load_error(&e),
        },
        Some(_) | None => {
            let filename = path_obj.file_name().map_or_else(
                || "unknown".to_string(),
                |n| n.to_string_lossy().to_string(),
            );
            eprintln!("Unsupported format: {}", filename);
            eprintln!("Supported formats: .pdf, .epub, .txt");
            eprintln!("For clipboard, use @@ command");
        }
    }
}
```

**6.3 Update Cargo.toml**

No additional dependencies required - text loading uses std::fs only.

---

## Implementation Plan

### 7.1 Bead Breakdown

This epic breaks into 6 focused beads:

#### Bead 2B-1.2-1: Fix OVP Anchor Alignment
- **Acceptance**: Anchor character stays at fixed screen position across all word lengths
- **File**: `src/ui/render.rs`
- **Test**: Manual visual test with words of various lengths

#### Bead 2B-1.2-2: Darken Context Words to 20% Opacity
- **Acceptance**: Context words barely visible compared to main text
- **File**: `src/ui/theme.rs`
- **Test**: Visual verification in reading mode

#### Bead 2B-1.2-3: Add Command Mode to AppMode
- **Acceptance**: New `AppMode::Command` enum variant exists
- **Files**: `src/app/mode.rs`, `src/app/app.rs`
- **Test**: Mode transitions work correctly

#### Bead 2B-1.2-4: Implement Command Input Buffer
- **Acceptance**: `command_input` field exists, characters accumulate correctly
- **Files**: `src/app/app.rs`
- **Test**: Type characters, backspace works, Ctrl+C quits

#### Bead 2B-1.2-5: Render Command Line Panel and Gutter
- **Acceptance**: 2-line command panel and full-height gutter render correctly
- **Files**: `src/ui/render.rs`, `src/ui/terminal.rs`
- **Test**: Visual test of command mode vs reading/paused mode

#### Bead 2B-1.2-6: Add txt File Support
- **Acceptance**: `@file.txt` loads and displays correctly
- **Files**: `src/input/text.rs` (new), `src/app/app.rs`, `src/input/mod.rs`
- **Test**: Load .txt file, verify tokenization

### 7.2 Implementation Order

**Phase 1: Critical Fixes (Beads 2B-1.2-1, 2B-1.2-2)**
1. Fix OVP alignment
2. Darken dimmed color
3. Test visual improvements

**Phase 2: Command Mode Infrastructure (Beads 2B-1.2-3, 2B-1.2-4)**
1. Add Command mode to enum
2. Add command_input buffer to App
3. Implement character-by-character input handling
4. Add Ctrl+C quit handler

**Phase 3: Rendering (Bead 2B-1.2-5)**
1. Create render_command_line() function
2. Create render_gutter() with top-down progress
3. Update TUI layout for 2-line command panel
4. Integrate gutter into main layout
5. Update render_frame() to use new components

**Phase 4: File Loading (Bead 2B-1.2-6)**
1. Create text file loader
2. Update handle_load_file() for .txt case
3. Test with sample text files

**Phase 5: Integration & Testing**
1. Test all mode transitions
2. Test command execution (@file, @@, :h, :q)
3. Test dimming in all modes
4. Run all unit tests
5. Manual testing checklist

### 7.3 Testing Strategy

**Manual Testing:**
- [ ] Anchor stays fixed for words: "I", "hello", "extraordinary", "Antidisestablishmentarianism"
- [ ] Context words barely visible (20% opacity)
- [ ] Command mode shows help + last doc info
- [ ] Typing in command mode works, backspace deletes
- [ ] Enter on `@file.txt` loads and switches to reading
- [ ] Enter on `@@` loads clipboard
- [ ] Reading mode shows dimmed "Press 'q' for commands"
- [ ] Pressing 'q' switches to command mode (reading visible but dimmed)
- [ ] Gutter extends full height, progress fills top-down
- [ ] Gutter dimmed significantly in reading mode
- [ ] Ctrl+C quits app from any mode
- [ ] All existing keybindings still work (j/k/space/[/])

**Automated Testing:**
- Unit tests for command parsing
- Unit tests for mode transitions
- Integration tests for file loading
- All 119 existing tests must still pass

### 7.4 Dependencies & Risks

**Dependencies**: No new crates required

**Risks**:
1. **Bookmark functionality incomplete**: Adding `bookmark_position` field but bookmark toggle command (`b` key) not implemented - deferring to future epic
2. **Gutter rendering complexity**: 100-line vertical gutter with bookmark line position calculation may need iterative refinement
3. **Mode state bugs**: New Command mode could introduce edge cases in mode transitions - thorough testing required
4. **Terminal size assumptions**: Full-height gutter assumes sufficient terminal height - may need minimum height check

### 7.5 Backwards Compatibility

**Breaking Changes**:
- Main.rs must change: Remove REPL loop, always run TUI
- AppMode enum gains `Command` variant
- Render state gains `bookmark_position` field

**Preserved Behavior**:
- All existing keybindings in reading mode unchanged
- File loading logic identical, just extended for .txt
- OVP anchoring logic unchanged
- Timing algorithms unchanged

---

## Summary

This epic transforms Speedy from a REPL→TUI architecture to an integrated TUI with command panel, similar to opencode. The command panel is always present but only interactive in Command mode, while Reading/Paused modes dim the panel to reduce distraction.

Key improvements:
- OVP anchor stays fixed at screen position
- Context words barely visible (20% opacity)
- Full-height gutter with top-down progress filling
- Integrated REPL eliminates jarring mode switches
- txt file support expands input options
- Ctrl+C quit handling improves UX

**PRD Alignment:**
- PRD Section 3.1 (OVP anchoring): Fixed alignment issue
- PRD Section 4.1 (Midnight theme): Dimmed colors adjusted
- PRD Section 4.2 (Gutter): Full-height implementation with progress
- PRD Section 7.1 (REPL): Integrated design
