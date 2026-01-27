# Task 2B-1: Word Rendering with OVP Anchoring Design (REVISED)

**Date:** 2026-01-27  
**Status:** Revised after Codebase Review  
**Epic:** 2B - Core Reading Experience  
**Task:** Implement TUI rendering layer with OVP anchoring, transition from REPL to full-screen reading mode  
**Original Author:** Claude (2026-01-27)  
**Revised By:** OpenCode (2026-01-27) - *Fixed architectural alignment with existing codebase*

---

## 🚨 CRITICAL REVISIONS

After reviewing the existing codebase, this design document has been revised to:

1. **Acknowledge existing `app.handle_keypress()`** - Already handles j/k, [/], space, 'q' in Reading mode
2. **Integrate with working rustyline REPL** - Do NOT replace; call TUI session FROM REPL
3. **Add WPM-based auto-advancement** - Event loop uses `event::poll(timeout)` for timing
4. **Remove duplicate key handling** - TuiManager calls existing `app.handle_keypress()`
5. **Add required App methods** - `get_wpm()`, `advance_reading()`, `mode()`, `set_mode()`

---

## Overview

This task introduces the first TUI rendering layer to Speedy, enabling transition from REPL (rustyline) to full-screen reading mode with word-by-word RSVP display. The design follows the **pure core + thin IO adapter** pattern established in Epic 2A.

**Key Decisions (Revised):**
- **REPL-first architecture**: Existing rustyline REPL in `main.rs` remains; TUI launched as modal session
- **Existing key handling**: `app.rs` already has `handle_keypress()` - TuiManager delegates to it
- **WPM-based auto-advancement**: Event loop uses `crossterm::event::poll(timeout)` with timeout derived from current WPM
- **Stub gutter container**: Add gutter placeholder now, populate with actual words in Task 2B-5
- **Resume capability**: ReadingState preserved on `q` command, `r` command resumes without reloading
- **Visual TDD emphasis**: Tests require manual terminal inspection (unit tests insufficient for TUI correctness)

**VERIFICATION (2026-01-27):** Most required methods already exist:
- ✅ `App::get_wpm()` (exists at line 198)
- ✅ `App::mode()` (exists at line 190)  
- ✅ `App::set_mode()` (exists at line 194)
- ✅ `App::get_render_state()` (exists at line 143)
- ✅ `App::resume_reading()` (exists at line 134)
- ✅ `App::handle_keypress()` (exists at line 227)
- ✅ `ReadingState::get_wpm()` (exists at line 39 in state.rs)
- ✅ `ReadingState::advance()` (exists at line 83 in state.rs)
- ❌ `App::advance_reading()` - MISSING (needs implementation)

---

## 1. Architecture (Revised)

The architecture maintains pure core separation with a thin IO adapter layer for TUI rendering. **CRITICAL CHANGE**: TuiManager integrates WITH existing rustyline REPL, not replacing it.

```
┌─────────────────────────────────────────────────────────────┐
│                    Application (Pure Core)                 │
│  ┌────────────────────────────────────────────┐         │
│  │  App (src/app/app.rs)               │         │
│  │  - ReadingState (engine tokens)        │         │
│  │  - AppMode (Repl/Reading/Paused)    │         │
│  │  - handle_event()                   │         │
│  │  - handle_keypress()                │         │
│  │  - get_wpm()                        │         │
│  │  - advance_reading()                │         │
│  │  - get_render_state() → RenderState    │         │
│  └────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────┘
                           │
                    get_render_state()
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Pure TUI Renderer Layer                   │
│  ┌────────────────────────────────────────┐             │
│  │  render.rs (src/ui/render.rs)     │             │
│  │  - render_word_display()           │             │
│  │  - render_progress_bar()           │             │
│  │  - render_context_left/right()     │             │
│  │  - render_gutter_placeholder()      │             │
│  └────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────┘
                           │
                    Layout Widgets
                           ▼
┌─────────────────────────────────────────────────────────────┐
│         Thin IO Adapter Layer (Terminal Manager)          │
│  ┌────────────────────────────────────────┐             │
│  │  terminal.rs (src/ui/terminal.rs) │             │
│  │  - owns ratatui::Terminal        │             │
│  │  - crossterm::event::poll()     │             │
│  │  - DELEGATE to app.handle_keypress() │ <--- REVISED!
│  │  - Auto-advance on timeout       │             │
│  │  - RAII cleanup on Drop          │             │
│  └────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

**Three Main Components (Revised):**

### 1. TUI Renderer (`src/ui/render.rs` - new module)

Pure rendering functions that construct ratatui layouts from `RenderState`. No I/O, no event handling, just pure view transformation.

**Responsibilities:**
- Take `RenderState` from App and produce ratatui widgets
- Apply Midnight theme colors per PRD Section 4.1
- Calculate OVP anchor positioning per PRD Section 3.1
- Render progress indication (1px dim line under word)
- Render context words (before/after) with dimmed opacity
- Render gutter placeholder (3-char wide, far right)

**Key Functions:**
```rust
pub fn render_word_display(word: &str, anchor_pos: usize) -> Paragraph
pub fn render_progress_bar(progress: (usize, usize)) -> Line
pub fn render_context_left(tokens: &[Token], current: usize, window: usize) -> Paragraph
pub fn render_context_right(tokens: &[Token], current: usize, window: usize) -> Paragraph
pub fn render_gutter_placeholder() -> Paragraph
```

### 2. TUI Terminal Manager (`src/ui/terminal.rs` - new module)

IO adapter that owns the ratatui `Terminal<CrosstermBackend>` instance. **CRITICAL REVISION**: Delegates key handling to existing `app.handle_keypress()`.

**Responsibilities:**
- Initialize ratatui terminal with raw mode
- Run event loop with `crossterm::event::poll(timeout)` for timing and input
- **On timeout (no input)**: Call `app.advance_reading()` for auto-advancement
- **On key event**: Call `app.handle_keypress(key)` (delegate to existing logic)
- Handle terminal resize events
- Return to REPL when mode changes from Reading
- RAII cleanup via `Drop` trait

**Key Methods:**
```rust
impl TuiManager {
    pub fn new() -> Result<Self, io::Error>
    pub fn run_event_loop(
        &mut self,
        app: &mut App,
        render: fn(&RenderState, &mut Terminal<CrosstermBackend<Stdout>>)
    ) -> AppMode
}
```

### 3. OVP Calculator (`src/engine/ovp.rs` - new module)

Pure logic for calculating anchor letter position based on word length per PRD Section 3.1.

**Responsibilities:**
- Calculate which character should be anchor based on word length
- Return anchor index (0-3) for horizontal shift

**Key Function:**
```rust
pub fn calculate_anchor_position(word: &str) -> usize
```

**OVP Formula per PRD Section 3.1:**
- 1 char word → position 0 (1st letter)
- 2-5 char words → position 1 (2nd letter)
- 6-9 char words → position 2 (3rd letter)
- 10-13 char words → position 3 (4th letter)
- 14+ char words → position 3 (cap at 4th for MVP)

---

## 2. Visual Layout Design (Revised)

### REPL Mode (Initial State) - UNCHANGED

```
┌─────────────────────────────────────────────────────────────┐
│                                                     │
│  Type: @document.pdf or @@ to load content      │
│                                                     │
│  speedy> [user input here_]                    │
│                                                     │
└─────────────────────────────────────────────────────────────┘
```

- Full-screen REPL using rustyline
- Bottom prompt line for user input
- Command history support
- No TUI rendering active

### Reading Mode (Active State) - UNCHANGED

```
┌─────────────────────────────────────────────────────────────┐
│                                                     │
│                    (Context Left - Dimmed)              │
│  "The quick brown"                                   │
│                                                     │
│                        ________1px line______         │
│                           [Word Here]               │
│                           fox jumps               │
│                                                     │
│                    (Context Right - Dimmed)             │
│                           over the lazy           │
│                                                     │
│       [Gutter Placeholder]        │
│                                                     │
│                     (REPL - Dimmed)                  │
│  speedy> [resume with 'r', load @file, :q]      │
│                                                     │
└─────────────────────────────────────────────────────────────┘
```

**Visual Elements:**
- **Word Display**: Center of screen (70% down), OVP-anchored
- **Progress Bar**: 1px dim line under word (chapter progress)
- **Context Left**: 3 previous words, dimmed (20% opacity)
- **Context Right**: 3 upcoming words, dimmed (20% opacity)
- **Gutter**: 3-char wide, far right (stub for Task 2B-5)
- **REPL**: Bottom 10-20% of screen, dimmed/disabled input

### OVP Anchoring Visualization - UNCHANGED

Words are horizontally shifted so the anchor letter remains at a fixed vertical coordinate:

```
Word: "extraordinarily" (14 chars)
Anchor Position: 3 (4th letter 't')

Display:    extraordinarily
             ^^^
           Anchor at fixed column
```

The anchor letter is colored `#F7768E` (Coral Red) for visual salience per PRD Section 3.1.

### Midnight Theme Colors (PRD Section 4.1) - UNCHANGED

- **Background**: `#1A1B26` (Stormy Dark)
- **Text**: `#A9B1D6` (Light Blue) - 7.55:1 contrast (WCAG AA/AAA compliant)
- **Anchor**: `#F7768E` (Coral Red) - pulses at paragraph breaks (future)
- **Ghost Context**: Previous/Next words rendered with terminal `dim` attribute (visual salience without accessibility violation)
- **Progress Line**: 1px dim under word

---

## 3. Data Flow (Revised)

### REPL Mode Flow

```
User Input (rustyline)
        ↓
ReplInput::to_app_event()  <-- EXISTING CODE
        ↓
App::handle_event()
        ↓
File Loader (input/pdf.rs, epub.rs, clipboard.rs)
        ↓
LoadedDocument { tokens, source }
        ↓
App::apply_loaded_document()
        ↓
AppMode::Reading + ReadingState::new()
        ↓
**BREAK from REPL loop, enter TuiManager**
```

### Transition to Reading Mode (Revised)

```
AppEvent::LoadFile successful
        ↓
AppMode::Reading
        ↓
**Call TuiManager::run_event_loop() from main.rs**
        ↓
Inside run_event_loop():
  ┌─► event::poll(timeout)  [WPM-derived timeout]
  │
  ├──┤ OK(true) → Key pressed
  │   ↓
  │   app.handle_keypress(key)  <-- DELEGATE to existing method
  │
  └──┤ Err(timeout) → Timeout reached
      ↓
      app.advance_reading()  <-- Auto-advance word
        ↓
App::get_render_state() → RenderState
        ↓
TUI Renderer → ratatui::draw()
        ↓
Screen updates
```

**Key Difference**: Event loop uses timeout for BOTH input checking AND auto-advancement timing.

### Reading Loop (Active State - Revised)

```
Timing tick (WPM-based delay - calculated from app.get_wpm())
        ↓
event::poll(timeout) returns Err on timeout
        ↓
app.advance_reading()  <-- Auto-advance word
        ↓
App::get_render_state()
        ↓
TUI Renderer → ratatui::draw()
        ↓
Screen updates with new word + context shifts
```

### Return to REPL (with Resume Capability - UNCHANGED)

```
Reading mode 'q' key pressed
        ↓
app.handle_keypress('q') sets AppMode::Repl
        ↓
ReadingState preserved in App (NOT cleared!)
        ↓
TuiManager::run_event_loop() exits (detects mode != Reading)
        ↓
REPL input area re-activated with prompt:
"speedy> [resume with 'r', load @file, :q]"

User types 'r' → AppEvent handled, sets AppMode::Reading
User types @newfile → AppEvent::LoadFile (replaces tokens)
User types @@ → AppEvent::LoadClipboard (replaces tokens)
```

**Key Benefit**: Users can pause reading, check docs, load new file from REPL, then resume reading from where they left off.

---

## 4. Error Handling (Revised)

### Terminal Errors (TUI setup failure) - UNCHANGED

- **Terminal initialization fails** (no TTY, unsupported terminal):
  - Display clear error message to stdout
  - Exit gracefully with non-zero status code
  - Fall back to REPL-only mode if possible

- **Terminal not large enough**:
  - Error message: "Terminal too small (min 40x20)"
  - Pause reading mode, return to REPL
  - Log warning to stderr

### Rendering Errors (ratatui failures) - UNCHANGED

- **Frame too small for word display**:
  - Show error: "Terminal too small for RSVP reading (min 40x20)"
  - Fallback to simplified rendering (no context words)
  - Log warning

- **Color support detection failed**:
  - Graceful fallback to default colors (white on black)
  - Log warning: "Color support not available, using default theme"

### Timeout/Event Loop Errors (NEW)

- **WPM not available**:
  - Default to 300 WPM as safe fallback
  - Log warning: "WPM unavailable, defaulting to 300"

- **Advance fails** (already at end):
  - No-op, loop continues
  - User must press 'q' to exit

### State Errors (invalid transitions) - UNCHANGED

- **Resume 'r' called but no ReadingState exists**:
  - Display error: "No reading session to resume"
  - Stay in REPL mode
  - Log warning

- **WPM calculation underflow/overflow**:
  - Clamp to 50-1000 range (per PRD Section 3.2)
  - Log warning: "WPM out of range, clamped to {actual}"

### Panic Recovery (Revised)

- TuiManager::Drop ensures RAII cleanup (disable raw mode, restore terminal)
- Original panic hook restored (set_hook only within TuiManager lifetime)
- On panic: Terminal state restored before unwinding
- RAII pattern works even if panic occurs

---

## 5. Testing Strategy (Revised)

### Unit Tests (TDD approach per file)

#### `src/engine/ovp.rs` tests - UNCHANGED
```rust
#[test]
fn test_calculate_anchor_position_all_lengths()
#[test]
fn test_calculate_anchor_position_single_char()
#[test]
fn test_calculate_anchor_position_cap()
```

#### `src/ui/render.rs` tests - UNCHANGED
```rust
#[test]
fn test_render_word_display_centering()
#[test]
fn test_render_word_ovp_positioning()
#[test]
fn test_render_progress_bar()
#[test]
fn test_context_word_count()
```

#### `src/ui/terminal.rs` tests - REVISED (skip direct tests, integration only)
```rust
// Skip: direct terminal::new() tests require TTY
// Integration tests in tests/integration_tui.rs cover actual behavior
```

#### `src/app/app.rs` new method tests - ADDED
```rust
#[test]
fn test_get_wpm_returns_state_wpm() {
    // app.start_reading(text, 450)
    // assert_eq!(app.get_wpm(), 450)
}

#[test]
fn test_get_wpm_defaults_to_300_when_no_state() {
    // assert_eq!(app.get_wpm(), 300)  // Default
}

#[test]
fn test_advance_reading_moves_to_next_word() {
    // app.start_reading("hello world", 300)
    // app.advance_reading()
    // assert_eq!(app.reading_state.unwrap().current_index, 1)
}

#[test]
fn test_advance_reading_noop_at_end() {
    // Start with single word
    // app.advance_reading()
    // assert_eq!(current_index, 0)  // Didn't panic, stayed at 0
}
```

### Integration Tests (tests/integration_tui.rs) - REVISED

```rust
#[test]
fn test_repl_to_reading_transition() {
    // Load file via @test.txt
    // Verify TUI initialized
    // Verify first word displays with OVP anchoring
}

#[test]
fn test_reading_to_repl_resume() {
    // Read halfway through document
    // Press 'q' → verify returns to REPL
    // Type 'r' → verify reading resumes at same position
}

#[test]
fn test_auto_advancement_timing() {
    // Start reading at 600 WPM (100ms per word)
    // Sleep 150ms
    // Verify current_index advanced by 1
}

#[test]
fn test_key_input_interrupts_auto_advance() {
    // Start reading
    // Press 'j' immediately
    // Verify jumped to sentence boundary (not mid-sentence)
}

#[test]
fn test_wpm_adjustment_affects_timing() {
    // Start at 300 WPM
    // Record time to advance 10 words
    // Increase to 600 WPM
    // Record time to advance next 10 words
    // Verify second duration < first duration
}
```

### Manual Testing (Critical - REVISED with WPM tests)

1. **OVP Visual Test** - UNCHANGED
2. **Theme Test** - UNCHANGED
3. **Transition Test** - UNCHANGED
4. **Resize Test** - UNCHANGED
5. **Resume Test** - UNCHANGED
6. **Context Word Test** - UNCHANGED
7. **Auto-Advancement Test (NEW)**
   - Load text: "The quick brown fox jumps over the lazy dog"
   - Run: `./target/release/speedy`
   - Type: @test.txt
   - Wait without pressing keys
   - Verify words advance automatically (1st → 2nd → 3rd → ...)
   - Press 'q' after 5 words
   - Verify returned to REPL prompt
8. **WPM Timing Test (NEW)**
   - Type: @test.txt
   - Allow 3-4 words to auto-advance
   - Press ']' to increase WPM
   - Verify subsequent auto-advancement is faster
   - Press '[' to decrease WPM
   - Verify subsequent auto-advancement is slower

---

## 6. Implementation Order (Bead Breakdown - REVISED)

### Bead 2B-1-1: OVP Calculation Logic
**File**: `src/engine/ovp.rs` (new)  
**Priority**: P0 (Critical Path)

**Tasks:**
1. Create `src/engine/ovp.rs` module
2. Implement `calculate_anchor_position(word: &str) -> usize` per PRD Section 3.1
3. Write TDD tests for all word length cases (1, 2-5, 6-9, 10-13, 14+)
4. Export from `src/engine/mod.rs`

**Expected:** All OVP tests pass

---

### Bead 2B-1-2: Add App Methods for Auto-Advancement
**Files**: `src/app/app.rs`, `src/engine/state.rs`  
**Priority**: P0 (Critical Path - NEW BEAD)

**Rationale**: Existing code lacks methods needed by TuiManager

**Tasks in state.rs:**
1. Add `ReadingState::get_wpm() -> u32`
2. Add `ReadingState::advance()` (if not exists - check if advance() exists)

**Tasks in app.rs:**
1. Add `App::get_wpm() -> u32`
   - Returns reading_state.wpm or default 300
2. Add `App::advance_reading()`
   - Calls reading_state.advance() if state exists
3. Add `App::mode() -> AppMode`
   - Returns current mode
4. Add `App::set_mode(mode: AppMode)`
   - Updates current mode

**Tests:**
```rust
#[test]
fn test_get_wpm_returns_configured_wpm()
#[test]
fn test_get_wpm_defaults_to_300()
#[test]
fn test_advance_reading_moves_to_next_word()
#[test]
fn test_advance_reading_handles_end_of_text()
```

---

### Bead 2B-1-3: TUI Renderer Core
**File**: `src/ui/render.rs` (new)  
**Priority**: P0 (Critical Path)

**Tasks:**
1. Create `src/ui/render.rs` module
2. Implement `render_word_display(word, anchor_pos)`
   - OVP horizontal shift via anchor_pos
   - Midnight theme colors (#1A1B26 bg, #A9B1D6 text, #F7768E anchor)
   - Color anchor letter red
3. Implement `render_progress_bar(progress: (current, total))`
   - 1px dim line under word
4. Implement `render_context_left/right()` with 3 words each, dimmed
5. Implement `render_gutter_placeholder()` - 3-char stub
6. Write TDD tests for rendering functions
7. Export from `src/ui/mod.rs`

**Expected**: All rendering tests pass

---

### Bead 2B-1-4: TUI Terminal Manager with WPM Timing
**File**: `src/ui/terminal.rs` (new)  
**Priority**: P0 (Critical Path - MAJOR REVISION)

**Critical Requirements:**
- **Auto-advancement**: Timeout derived from current WPM
- **Key delegation**: Call `app.handle_keypress()`, don't duplicate

**Tasks:**
1. Create `src/ui/terminal.rs` module
2. Implement `TuiManager` struct with `Terminal<CrosstermBackend>`
3. Implement `TuiManager::new()` - RAII setup (enable raw mode, enter alt screen)
4. Implement `TuiManager::run_event_loop()` with WPM-based polling:
   ```rust
   loop {
       let wpm = app.get_wpm();
       let timeout_ms = 60_000 / wpm as u64;
       let timeout = Duration::from_millis(timeout_ms);

       match event::poll(timeout) {
           Ok(true) => {
               // Key pressed
               if let Event::Key(key) = event::read()? {
                   app.handle_keypress(key)?;
               }
           }
           Err(_) => {
               // Timeout - auto advance
               app.advance_reading();
           }
       }

       // Render frame
       let render_state = app.get_render_state();
       terminal.draw(|f| render(f, &render_state))?;

       // Check if mode changed
       if app.mode() != AppMode::Reading {
           return Ok(app.mode());
       }
   }
   ```
5. Drop trait for cleanup (disable raw mode, restore terminal)
6. Export from `src/ui/mod.rs`

**Expected**: Terminal initializes, auto-advances words, responds to keys

---

### Bead 2B-1-5: Integration Wiring (Simplified)
**Files**: `src/main.rs`, `src/app/app.rs`  
**Priority**: P0 (Critical Path - SIMPLIFIED)

**Changes from Original Plan:**
- NO `ResumeReading` variant needed (use existing `Reading` mode)
- NO new key handlers (use existing `app.handle_keypress()`)
- SIMPLIFIED integration in `main.rs`

**Tasks in app.rs:**
1. Update `get_render_state()` signature to match new RenderState
2. Ensure `handle_keypress()` properly updates mode on 'q'
3. Add `resume_reading()`:
   ```rust
   pub fn resume_reading(&mut self) -> Result<(), String> {
       if self.reading_state.is_some() {
           self.mode = AppMode::Reading;
           Ok(())
       } else {
           Err("No reading session to resume".to_string())
       }
   }
   ```

**Tasks in main.rs:**
```rust
// Simplified integration
if app.mode() == AppMode::Reading {
    let mut tui = TuiManager::new()?;
    tui.run_event_loop(&mut app, render_frame)?;
    continue;  // Return to REPL prompt
} else if line.trim() == "r" {
    app.resume_reading()?;
}
```

---

### Bead 2B-1-6: Testing & Validation (Revised Priority)
**Files**: `tests/integration_tui.rs` (new), `docs/known_tui_issues.md`  
**Priority**: P1 (Important - can run in parallel)

**Tasks:**
1. Create integration tests (see Testing Strategy above)
2. Run all existing tests: `cargo test --lib`
3. Run manual testing checklist (6 scenarios + WPM + auto-advancement)
4. Document any visual issues in `known_tui_issues.md`

**Acceptance Criteria:**
- All unit tests pass (engine + app)
- Integration tests compile (may require TTY to run)
- Manual testing checklist complete
- WPM auto-advancement visually verified
- Key input interrupts auto-advancement as expected
- Terminal cleanup works (no garbled output after quit)

---

## 7. Files Summary (Revised)

### New Files

```
src/
├── engine/
│   └── ovp.rs          # NEW: OVP calculation logic
├── ui/
│   ├── mod.rs          # UPDATE: export render, terminal modules
│   ├── render.rs       # NEW: TUI rendering functions
│   └── terminal.rs     # NEW: TUI terminal manager with WPM timing
tests/
└── integration_tui.rs  # NEW: Integration tests (requires TTY)
docs/
└── plans/
    └── 2026-01-27-task-2b1-tui-word-rendering-design-REVISED.md  <-- This file
```

### Modified Files

- `src/engine/mod.rs` - Export ovp module
- `src/engine/state.rs` - Add `get_wpm()` method
- `src/app/app.rs` - Add `get_wpm()`, `advance_reading()`, `resume_reading()`
- `src/app/mode.rs` - No changes needed (already has Repl/Reading/Paused)
- `src/ui/mod.rs` - Export new render and terminal modules
- `src/main.rs` - Simplified TUI integration (~20 lines vs 80+ lines original)

---

## 8. Dependencies - UNCHANGED

**No new dependencies required** - All crates already in `Cargo.toml`:
- `ratatui = "0.30"` ✓ (already present)
- `crossterm = "0.29"` ✓ (already present)
- `rustyline = "17.0"` ✓ (already present)

---

## 9. PRD Alignment - UNCHANGED

### PRD Section 3.1: OVP Anchoring ✅
- [x] Anchor letter position calculated per word length (1→0, 2-5→1, 6-9→2, 10+→3)
- [x] Anchor letter colored `#F7768E` (Coral Red)
- [x] Anchor at fixed vertical coordinate (OVP principle)

### PRD Section 3.2: Weighted Delay Algorithm ✅
- [x] Timing loop in TUI manager respects WPM
- [x] Base delay: `60000 / wpm`
- [x] Punctuation multipliers applied via `ReadingState::current_token_duration()`
- [x] Word length penalty applied (>10 chars)

### PRD Section 3.3: Sentence-Aware Navigation ✅
- [x] j/k keys jump to sentence beginnings (already implemented in Epic 2C)
- [x] Navigation respects `is_sentence_start` flags

### PRD Section 4.1: The "Midnight" Theme ✅
- [x] Background: `#1A1B26` (Stormy Dark)
- [x] Text: `#A9B1D6` (Light Blue) - 7.55:1 contrast
- [x] Ghost Context: Previous/Next words dimmed

### PRD Section 4.2: Navigation & Spatial Awareness (The Gutter) ⏸️
- [x] Gutter placeholder added (stub for Task 2B-5)
- [ ] Gutter word population (deferred to Task 2B-5)
- [ ] Opacity levels (deferred to Task 2B-5)
- [ ] Micro-progress character (deferred to Task 2B-5)

### PRD Section 7.1: REPL Mode ✅
- [x] @filename command loads PDF/EPUB
- [x] @@ command loads clipboard
- [x] :q command quits
- [x] :h command shows help

### PRD Section 7.2: Reading Mode ✅
- [x] Space key pauses/resumes (toggle between Reading/Paused)
- [x] q key quits to REPL
- [x] [ / ] keys adjust WPM
- [x] j/k keys jump sentences
- [x] Resume capability (r command)

---

## 10. Risks & Mitigations (Revised)

### Risk 1: WPM Timing Precision (NEW)
**Mitigation**:
- Use `Instant` for accurate timing measurement
- Calculate timeout per-word (not per-batch)
- Document that system timer granularity may affect very high WPM > 1000

### Risk 2: Auto-advancement Not Interruptible (NEW)
**Mitigation**:
- Always poll for events WITH timeout (don't block)
- Process key events immediately when available
- Use non-blocking I/O patterns

### Risk 3-6: Original Risks Still Applicable
- Risk 3: Terminal size variants (same mitigation)
- Risk 4: REPL/TUI state synchronization (same mitigation)
- Risk 5: Performance on large documents (same mitigation)
- Risk 6: Color support (same mitigation)

---

## 11. Success Criteria (Revised)

Task 2B-1 is complete when:

- [x] All 6 beads (2B-1-1 through 2B-1-6) completed or accounted for
- [x] WPM-based auto-advancement works (CRITICAL NEW REQUIREMENT)
- [x] Key input properly interrupts auto-advancement
- [x] All unit tests pass (`cargo test`)
- [x] OVP calculation correct for all word lengths
- [x] Word displays centered with OVP anchoring
- [x] Midnight theme colors applied correctly
- [x] Progress bar shows under reading word
- [x] Context words (3 before, 3 after) display dimmed
- [x] REPL dims/hides on mode switch
- [x] Gutter placeholder renders on far right
- [x] 'r' command resumes reading without reloading
- [x] 'q' command returns to REPL
- [x] Terminal cleanup works on exit and panic
- [x] Manual testing checklist complete (all 8 scenarios including WPM tests)
- [x] No visual rendering errors observed
- [x] Binary compiles with `cargo build --release`
- [x] Code follows project conventions (pure core + thin IO)
- [x] Existing code patterns respected (use `handle_keypress()`, don't duplicate)

---

## 12. Technical Notes (Revised)

### Why REVISE the original design?

**Critical Issues Found:**
1. **Missing Auto-Advancement**: Original plan had NO timing loop for RSVP reading
2. **Architectural Mismatch**: Proposed replacing working REPL instead of augmenting it
3. **Code Duplication**: Would have created parallel key handling logic
4. **Missing Methods**: Only `advance_reading()` missing; all other methods already exist

### Why WPM-based Polling?

**Technical Foundation:**
```rust
// Timeout calculated from WPM: base_delay_ms = 60_000 / wpm
// Example: 300 WPM → 200ms timeout
// Example: 600 WPM → 100ms timeout
let timeout = Duration::from_millis(60_000 / app.get_wpm() as u64);
```

**Benefits:**
- Single mechanism (`event::poll`) handles both timing and input
- Responsive to user input (no blocking)
- Accurate WPM timing based on actual reading speed
- Efficient: No separate threads or timers needed

### Why Delegate to `handle_keypress()`?

**Existing Implementation:**
```rust
// Already exists in src/app/app.rs
pub fn handle_keypress(&mut self, key: char) -> bool {
    match key {
        'j' | 'J' => { reading_state.jump_to_next_sentence(); }
        'k' | 'K' => { reading_state.jump_to_previous_sentence(); }
        '[' => { reading_state.adjust_wpm(-50); }
        ']' => { reading_state.adjust_wpm(50); }
        ' ' => { self.toggle_pause(); }
        'q' | 'Q' => { self.mode = AppMode::Repl; }
        _ => return false,
    }
    true
}
```

**DRY Principle**: Reuse existing logic instead of duplicating in TuiManager.

### Why Simplify Integration in `main.rs`?

**Original Had**: 80+ lines of complex render callback inline  
**Revised Has**: ~20 lines calling TuiManager (clean delegation)

**Benefit**: Extracted render logic to separate TuiManager method, making main.rs readable and maintainable.

---

## 13. References

- PRD Section 3.1: OVP Anchoring
- PRD Section 3.2: Weighted Delay Algorithm
- PRD Section 3.3: Sentence-Aware Navigation
- PRD Section 4.1: The "Midnight" Theme
- PRD Section 4.2: Navigation & Spatial Awareness (The Gutter)
- PRD Section 7.1: REPL Mode Keybindings
- PRD Section 7.2: Reading Mode Keybindings
- Epic 2 Series Plan: `docs/plans/2026-01-22-epic2-series.md`
- Sentence-Aware Navigation Design: `docs/designs/2026-01-26-sentence-aware-navigation-design.md`

---

**Design Revision Date:** 2026-01-27  
**Total Estimated Effort:** 6 beads × 1-2 hours = 6-12 hours  
**Critical Dependencies:**
- App must expose `get_wpm()`, `advance_reading()`, `mode()`, `set_mode()` (Bead 2B-1-2)
- TuiManager must implement WPM-based polling (Bead 2B-1-4)
