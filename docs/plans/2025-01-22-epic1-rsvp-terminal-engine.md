# Epic 1: RSVP Terminal Engine - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a functional RSVP (Rapid Serial Visual Presentation) terminal reader with REPL interface, configurable WPM, pause/resume controls, and robust error handling, using a clean modular architecture with engine/UI separation.

**Architecture:** Separates "Brain" (pure timing logic in engine/) from "Eyes" (terminal rendering in ui/) connected through an AppMode state machine. Text flows from file/clipboard → TokenStream → PlaybackEngine → Renderer.

**PRD Alignment:**
- REPL interface (Section 2.2): Interactive prompt with `@filename`, `@@`, tab completion
- WPM controls (Section 7.2): `[ / ]` to decrease/increase speed
- Error handling: File I/O, terminal resize, graceful degradation

**Tech Stack:**
- Rust 1.90+
- ratatui 0.30+ - TUI framework
- crossterm 0.29+ - Terminal control (embedded by ratatui)
- `cargo` - Package manager and testing

**Task Count:** 14 tasks (expanded from original 11 to include REPL, WPM controls, and error handling per PRD alignment)

---

## Task 1: Initialize Rust Project Structure

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/engine/mod.rs`
- Create: `src/ui/mod.rs`
- Create: `src/audio/mod.rs` (stub)
- Create: `src/storage/mod.rs` (stub)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "speedy"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"

[dev-dependencies]
```

**Step 2: Create main.rs entry point**

```rust
fn main() {
    println!("Speedy RSVP Reader - MVP");
}
```

**Step 3: Create module stubs**

```rust
// src/engine/mod.rs
pub mod timing;

// src/ui/mod.rs
pub mod reader;

// src/audio/mod.rs
// TODO: Epic N - Audio metronome implementation

// src/storage/mod.rs
// TODO: Epic N - Persistence and history
```

**Step 4: Verify project compiles**

```bash
cargo check
```
Expected: No errors

**Step 5: Commit**

```bash
git add .
git commit -m "chore: initialize rust project structure"
```

---

## Task 2: Write Failing Tests for Timing Engine

**Files:**
- Create: `src/engine/timing.rs`
- Test: `src/engine/timing.rs` (tests module)

**Step 1: Write test for WPM calculation**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_to_milliseconds_at_300() {
        let wpm = 300;
        let expected_ms = 60_000 / wpm; // 200ms per word
        assert_eq!(expected_ms, 200);
    }

    #[test]
    fn test_wpm_to_milliseconds_at_600() {
        let wpm = 600;
        let expected_ms = 60_000 / wpm; // 100ms per word
        assert_eq!(expected_ms, 100);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test engine::timing::tests::test_wpm_to_milliseconds_at_300
```
Expected: FAIL with "undefined function wpm_to_milliseconds"

**Step 3: Write test for word splitting**

```rust
#[test]
fn test_word_splitting_single_word() {
    let text = "hello";
    let words: Vec<&str> = text.split_whitespace().collect();
    assert_eq!(words, vec!["hello"]);
}

#[test]
fn test_word_splitting_multiple_words() {
    let text = "hello world test";
    let words: Vec<&str> = text.split_whitespace().collect();
    assert_eq!(words, vec!["hello", "world", "test"]);
}
```

**Step 4: Run test to verify it fails**

```bash
cargo test engine::timing::tests::test_word_splitting_single_word
```
Expected: FAIL (test needs implementation to run)

**Step 5: Commit**

```bash
git add src/engine/timing.rs
git commit -m "test: add failing timing engine tests"
```

---

## Task 3: Implement Timing Engine

**Files:**
- Modify: `src/engine/timing.rs`

**Step 1: Implement WPM to milliseconds function**

```rust
pub fn wpm_to_milliseconds(wpm: u32) -> u32 {
    60_000 / wpm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_to_milliseconds_at_300() {
        let result = wpm_to_milliseconds(300);
        assert_eq!(result, 200);
    }

    #[test]
    fn test_wpm_to_milliseconds_at_600() {
        let result = wpm_to_milliseconds(600);
        assert_eq!(result, 100);
    }
}
```

**Step 2: Run test to verify it passes**

```bash
cargo test engine::timing::tests::test_wpm_to_milliseconds_at_300
cargo test engine::timing::tests::test_wpm_to_milliseconds_at_600
```
Expected: PASS

**Step 3: Implement word tokenization**

```rust
pub struct Token {
    pub text: String,
    pub duration_ms: u32,
}

pub fn tokenize_text(text: &str, wpm: u32) -> Vec<Token> {
    let base_duration = wpm_to_milliseconds(wpm);
    text.split_whitespace()
        .map(|word| Token {
            text: word.to_string(),
            duration_ms: base_duration,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_splitting_single_word() {
        let text = "hello";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_word_splitting_multiple_words() {
        let text = "hello world test";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[2].text, "test");
    }

    #[test]
    fn test_word_splitting_applies_duration() {
        let text = "hello";
        let tokens = tokenize_text(text, 300);
        assert_eq!(tokens[0].duration_ms, 200);
    }
}
```

**Step 4: Run all tests to verify they pass**

```bash
cargo test engine::timing
```
Expected: All PASS

**Step 5: Commit**

```bash
git add src/engine/timing.rs
git commit -m "feat: implement timing engine with word tokenization"
```

---

## Task 4: Write Failing Tests for AppMode State Machine

**Files:**
- Create: `src/app.rs`
- Test: `src/app.rs` (tests module)

**Step 1: Write test for AppMode state transitions**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appmode_reading_starts_reading() {
        let mode = AppMode::Reading;
        assert!(matches!(mode, AppMode::Reading));
    }

    #[test]
    fn test_appmode_paused_transitions_to_reading() {
        let paused = AppMode::Paused;
        let reading = AppMode::Reading;
        assert_ne!(paused, reading);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test app::tests::test_appmode_reading_starts_reading
```
Expected: FAIL with "AppMode not defined"

**Step 3: Write test for reading state structure**

```rust
pub struct ReadingState {
    pub tokens: Vec<engine::Token>,
    pub current_index: usize,
    pub wpm: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_state_initialization() {
        let tokens = vec![
            engine::Token { text: "hello".to_string(), duration_ms: 200 },
            engine::Token { text: "world".to_string(), duration_ms: 200 },
        ];
        let state = ReadingState {
            tokens: tokens.clone(),
            current_index: 0,
            wpm: 300,
        };
        assert_eq!(state.tokens, tokens);
        assert_eq!(state.current_index, 0);
        assert_eq!(state.wpm, 300);
    }
}
```

**Step 4: Run test to verify it fails**

```bash
cargo test app::tests::test_reading_state_initialization
```
Expected: FAIL with "undefined type ReadingState"

**Step 5: Commit**

```bash
git add src/app.rs
git commit -m "test: add failing app mode tests"
```

---

## Task 5: Implement AppMode State Machine

**Files:**
- Modify: `src/app.rs`

**Step 1: Implement AppMode enum**

```rust
use crate::engine;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Reading,
    Paused,
    Quit,
}

#[derive(Debug, Clone)]
pub struct ReadingState {
    pub tokens: Vec<engine::Token>,
    pub current_index: usize,
    pub wpm: u32,
}

impl ReadingState {
    pub fn new(tokens: Vec<engine::Token>, wpm: u32) -> Self {
        Self {
            tokens,
            current_index: 0,
            wpm,
        }
    }

    pub fn current_token(&self) -> Option<&engine::Token> {
        self.tokens.get(self.current_index)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.tokens.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }
}

pub struct App {
    pub mode: AppMode,
    pub reading_state: Option<ReadingState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Reading,
            reading_state: None,
        }
    }

    pub fn start_reading(&mut self, tokens: Vec<engine::Token>) {
        const DEFAULT_WPM: u32 = 300;
        self.reading_state = Some(ReadingState::new(tokens, DEFAULT_WPM));
        self.mode = AppMode::Reading;
    }

    pub fn toggle_pause(&mut self) {
        match self.mode {
            AppMode::Reading => self.mode = AppMode::Paused,
            AppMode::Paused => self.mode = AppMode::Reading,
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appmode_reading_starts_reading() {
        let mode = AppMode::Reading;
        assert!(matches!(mode, AppMode::Reading));
    }

    #[test]
    fn test_reading_state_initialization() {
        let tokens = vec![
            engine::Token { text: "hello".to_string(), duration_ms: 200 },
            engine::Token { text: "world".to_string(), duration_ms: 200 },
        ];
        let state = ReadingState::new(tokens, 300);
        assert_eq!(state.current_index, 0);
        assert_eq!(state.wpm, 300);
    }

    #[test]
    fn test_reading_state_current_token() {
        let tokens = vec![
            engine::Token { text: "hello".to_string(), duration_ms: 200 },
        ];
        let state = ReadingState::new(tokens, 300);
        let token = state.current_token().unwrap();
        assert_eq!(token.text, "hello");
    }

    #[test]
    fn test_reading_state_advance() {
        let tokens = vec![
            engine::Token { text: "hello".to_string(), duration_ms: 200 },
            engine::Token { text: "world".to_string(), duration_ms: 200 },
        ];
        let mut state = ReadingState::new(tokens, 300);
        assert!(state.advance());
        assert_eq!(state.current_index, 1);
        assert!(!state.advance()); // No more tokens
    }

    #[test]
    fn test_app_start_reading() {
        let mut app = App::new();
        let tokens = vec![
            engine::Token { text: "hello".to_string(), duration_ms: 200 },
        ];
        app.start_reading(tokens);
        assert_eq!(app.mode, AppMode::Reading);
        assert!(app.reading_state.is_some());
    }

    #[test]
    fn test_app_toggle_pause() {
        let mut app = App::new();
        app.toggle_pause();
        assert_eq!(app.mode, AppMode::Paused);
        app.toggle_pause();
        assert_eq!(app.mode, AppMode::Reading);
    }
}
```

**Step 2: Run all tests to verify they pass**

```bash
cargo test app
```
Expected: All PASS

**Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat: implement AppMode state machine with reading state"
```

---

## Task 6: Write Failing Tests for UI Reader Component

**Files:**
- Create: `src/ui/reader.rs`
- Test: `src/ui/reader.rs` (tests module)

**Step 1: Write test for rendering current word**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_word_centered() {
        // This will need a mock terminal or verify string output
        // For MVP, verify word is included in output
        let word = "hello";
        let output = render_word(word);
        assert!(output.contains(word));
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test ui::reader::tests::test_render_word_centered
```
Expected: FAIL with "undefined function render_word"

**Step 3: Commit**

```bash
git add src/ui/reader.rs
git commit -m "test: add failing UI reader tests"
```

---

## Task 7: Implement UI Reader with Ratatui

**Files:**
- Modify: `src/ui/reader.rs`

**Step 1: Implement basic word rendering**

```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::Paragraph,
    Frame,
    Terminal,
};

pub fn render_word(word: &str) -> String {
    word.to_string()
}

// Note: Full ratatui integration will be in main.rs event loop
// This module provides helper functions for rendering

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_word_centered() {
        let word = "hello";
        let output = render_word(word);
        assert_eq!(output, "hello");
    }
}
```

**Step 2: Run test to verify it passes**

```bash
cargo test ui::reader
```
Expected: PASS

**Step 3: Commit**

```bash
git add src/ui/reader.rs
git commit -m "feat: implement UI reader word rendering"
```

---

## Task 13: Implement REPL Interface

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`

**Step 1: Implement REPL command parsing**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    LoadFile(String),
    LoadClipboard,
    Quit,
}

pub fn parse_repl_input(input: &str) -> ReplCommand {
    let trimmed = input.trim();
    
    if trimmed.starts_with("@@") {
        return ReplCommand::LoadClipboard;
    }
    
    if trimmed.starts_with('@') && trimmed.len() > 1 {
        return ReplCommand::LoadFile(trimmed[1..].to_string());
    }
    
    if trimmed == ":q" || trimmed == ":quit" {
        return ReplCommand::Quit;
    }
    
    ReplCommand::LoadFile(trimmed.to_string()) // Fallback
}
```

**Step 2: Add REPL mode to AppMode enum**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Repl,           // NEW: Interactive prompt
    Reading,         // Displaying words
    Paused,          // Reading paused
    Quit,            // Application exit
}
```

**Step 3: Implement REPL loop in main.rs**

```rust
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable raw mode for REPL
    enable_raw_mode()?;
    
    let mut app = app::App::new();
    
    loop {
        match app.mode {
            AppMode::Repl => {
                print!("speedy> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                let command = app::parse_repl_input(&input);
                
                match command {
                    app::ReplCommand::LoadFile(filename) => {
                        let content = fs::read_to_string(&filename)?;
                        let tokens = engine::tokenize_text(&content, 300);
                        app.start_reading(tokens);
                    }
                    app::ReplCommand::LoadClipboard => {
                        // TODO: Add clipboard support (future epic)
                        println!("Clipboard loading not yet implemented");
                    }
                    app::ReplCommand::Quit => {
                        break;
                    }
                    _ => {}
                }
            }
            AppMode::Reading | AppMode::Paused => {
                // Existing event loop logic
                break; // Exit REPL mode
            }
            AppMode::Quit => break,
        }
    }
    
    Ok(())
}
```

**Step 4: Run REPL tests**

```bash
cargo test app::tests
```
Expected: PASS

**Step 5: Test REPL manually**

```bash
cargo run
# At prompt: @test.txt
# Should load file and start reading
```

**Step 6: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: implement REPL interface with file loading"
```

---

## Task 10: Write Failing Tests for WPM Controls

**Files:**
- Modify: `src/app.rs` (add WPM tests)

**Step 1: Write test for WPM adjustment**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_decrease() {
        let mut state = ReadingState::new(vec![], 300);
        state.adjust_wpm(-50);
        assert_eq!(state.wpm, 250);
    }

    #[test]
    fn test_wpm_increase() {
        let mut state = ReadingState::new(vec![], 300);
        state.adjust_wpm(50);
        assert_eq!(state.wpm, 350);
    }

    #[test]
    fn test_wpm_minimum_bound() {
        let mut state = ReadingState::new(vec![], 100);
        state.adjust_wpm(-50); // Should not go below minimum
        assert!(state.wpm >= 50); // Minimum reasonable WPM
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test app::tests::test_wpm_decrease
```
Expected: FAIL with "adjust_wpm not defined"

**Step 3: Commit**

```bash
git add src/app.rs
git commit -m "test: add failing WPM control tests"
```

---

## Task 11: Implement WPM Controls in Event Loop

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`

**Step 1: Add WPM adjustment to ReadingState**

```rust
impl ReadingState {
    const MIN_WPM: u32 = 50;
    const MAX_WPM: u32 = 1000;
    
    pub fn adjust_wpm(&mut self, delta: i32) {
        let new_wpm = self.wpm as i32 + delta;
        self.wpm = new_wpm.clamp(Self::MIN_WPM as i32, Self::MAX_WPM as i32) as u32;
    }
}
```

**Step 2: Handle WPM keys in event loop**

```rust
// In main.rs event loop:
if let Event::Key(key) = event::read()? {
    match key {
        KeyCode::Char('q') => break,
        KeyCode::Char(' ') => {
            app.toggle_pause();
        }
        KeyCode::Char('[') => {  // NEW: Decrease WPM
            if let Some(state) = &mut app.reading_state {
                state.adjust_wpm(-50);
            }
        }
        KeyCode::Char(']') => {  // NEW: Increase WPM
            if let Some(state) = &mut app.reading_state {
                state.adjust_wpm(50);
            }
        }
        _ => {}
    }
}
```

**Step 3: Update display to show WPM**

```rust
// In terminal draw:
let wpm_display = app.reading_state
    .as_ref()
    .map(|s| format!("{} WPM", s.wpm))
    .unwrap_or_else(|| "".to_string());
```

**Step 4: Run tests**

```bash
cargo test app::tests
```
Expected: PASS

**Step 5: Test WPM controls manually**

```bash
cargo run -- /tmp/test.txt
# Press [ to decrease speed, ] to increase speed
```

**Step 6: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: implement WPM controls in event loop"
```

---

## Task 12: Add Comprehensive Error Handling

**Files:**
- Modify: `src/main.rs`
- Create: `src/engine/error.rs`

**Step 1: Define error types**

```rust
#[derive(Debug)]
pub enum SpeedyError {
    IoError(std::io::Error),
    EmptyFile(String),
    InvalidEncoding(String),
}
```

**Step 2: Add error tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file_handling() {
        let tokens = tokenize_text("", 300);
        assert_eq!(tokens.len(), 0);
    }
}
```

**Step 3: Handle errors gracefully in main.rs**

```rust
fn load_file_safe(path: &str) -> Result<String, SpeedyError> {
    let content = fs::read_to_string(path)
        .map_err(SpeedyError::IoError)?;
    
    if content.trim().is_empty() {
        return Err(SpeedyError::EmptyFile(path.to_string()));
    }
    
    Ok(content)
}
```

**Step 4: Commit**

```bash
git add src/engine/error.rs src/main.rs
git commit -m "feat: add comprehensive error handling"
```

---

## Task 13: Write Integration Test

**Files:**
- Create: `tests/integration_test.rs`

**Step 1: Write end-to-end test**

```rust
use std::{fs, process::Command};

#[test]
fn test_end_to_end_reading() {
    // Create test file
    let test_file = "/tmp/speedy_test.txt";
    fs::write(test_file, "hello world").expect("Failed to write test file");

    // Run speedy
    let output = Command::new("cargo")
        .args(["run", "--", test_file])
        .output()
        .expect("Failed to run speedy");

    // Verify output contains expected text
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
}
```

**Step 2: Run integration test**

```bash
cargo test test_end_to_end_reading
```
Expected: PASS

**Step 3: Commit**

```bash
git add tests/
git commit -m "test: add integration test for end-to-end reading"
```

---

## Task 14: Final Verification

**Files:**
- Verify: All tests pass
- Verify: Build succeeds

**Step 1: Run all tests**

```bash
cargo test
```
Expected: All PASS

**Step 2: Run build check**

```bash
cargo build --release
```
Expected: Binary created at `target/release/speedy`

**Step 3: Test binary manually**

```bash
# Test REPL mode
cargo run
# At prompt: @test.txt
# Should load file and start reading

# Test WPM controls during reading
# Press [ to decrease, ] to increase speed

# Test pause/resume
# Press SPACE to pause, SPACE again to resume

# Test quit
# Press q to exit to REPL, then :q to quit
```
Expected: Words flash on screen, REPL loads files, WPM adjustable, pause/resume works

**Step 4: Commit final version**

```bash
git add .
git commit -m "chore: Epic 1 complete - RSVP terminal engine with REPL and WPM controls"
```

---

## Success Criteria

Epic 1 is complete when:
- ✅ User can run `speedy` and see REPL prompt (`speedy>`)
- ✅ REPL accepts `@filename` to load text files
- ✅ REPL accepts `@@` for clipboard (placeholder for future epic)
- ✅ Reading mode displays words flashing on screen
- ✅ SPACE pauses/resumes reading
- ✅ `[` / `]` keys adjust WPM (default 300, range 50-1000)
- ✅ `q` quits reading mode and returns to REPL
- ✅ `:q` quits application from REPL
- ✅ Modular architecture (engine/ui separation)
- ✅ All tests pass (unit + integration)
- ✅ Project compiles without warnings
- ✅ Error handling for file I/O, empty files, invalid input
- ✅ `audio/` and `storage/` stubs in place for future epics
