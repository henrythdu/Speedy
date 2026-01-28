# Epic 2 Series: Interactive REPL and Event Loop

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable interactive terminal usage with functional REPL and RSVP reading experience, transforming Speedy from library to usable application.

**Architecture:** Single-threaded event loop using `crossterm::event::poll(timeout)` with tick-based approach. Pure core state machine (app::App) separated from thin IO adapter layer (main.rs). Terminal setup/teardown managed via RAII pattern with panic hooks for robust cleanup.

**Tech Stack:** crossterm 0.29, ratatui 0.30, Rust 2021 edition, existing modules (app::, repl::, engine::, ui::)

---

## Overview

Break Epic 2 into 3 focused epics, each delivering a demonstrable milestone:

- **Epic 2A: Terminal Foundation & Architecture** - Establish stable REPL and core architecture
- **Epic 2B: Core Reading Experience** - Build primary user-facing reading feature
- **Epic 2C: Hardening & Platform Support** - Testing and platform compatibility

Each epic: 4-5 tasks, manageable within 1-2 focused work sessions.

---

## Epic 2A: Terminal Foundation & Architecture (4 tasks)

**Goal:** Establish stable interactive REPL with pure core architecture. Nothing else works without this foundation.

**Completion Criteria:**
- REPL prompt responds to user input
- File can be loaded (@filename)
- Pure core architecture established (AppEvent enum, handle_event(), get_render_state())
- Terminal cleanup works on normal exit and panic

**Tasks:**

### Task 2A-1: Terminal Setup & Event Loop Skeleton
- Create `src/terminal.rs` (RAII TerminalGuard)
- Implement basic event loop with crossterm::event::poll(timeout)
- Add panic hook for terminal restoration
- Test Ctrl+C handling

### Task 2A-2: Refactor app::App for Pure Core Architecture
- Add `AppEvent` enum (domain events, not crossterm::KeyEvent)
- Add `handle_event()` method
- Add `get_render_state()` method returning `RenderState` struct
- Add `RenderState` struct (no ratatui types)
- Export new types from `src/app/mod.rs`

### Task 2A-3: REPL Mode - Display & Input
- Display "speedy> " prompt
- Capture user input line-by-line
- Parse input using existing `repl::parse_repl_input()`
- Handle `:q` (quit) and `:h` (help) commands

### Task 2A-4: File Loading Integration
- Load file using `engine::load_file_safe()`
- Tokenize text using `engine::tokenize_text()` (hardcoded timing for now)
- Create `App` with reading state
- Transition to Reading mode

---

## Epic 2B: Core Reading Experience (5 main tasks + 1 polish sub-epic)

**Note:** After completing Task 2B-1, manual testing revealed critical UX issues. These are tracked in sub-epic **2B-1.1** to maintain clean separation between initial implementation and polish work.

**Sub-Epics:**
- **Epic 2B-1.1: TUI Polish & UX Fixes** - Address testing issues before proceeding to 2B-2

**Goal:** Build primary user-facing reading feature. Can read a document from start to finish with speed control.

**Completion Criteria:**
- Words display one by one with OVP anchoring
- Timing respects WPM (after Task 2B-3 fix)
- Pause/resume works with Space key
- WPM adjustment works with `[` / `]` keys
- Progress indication shows word count and WPM

**Tasks:**

### Task 2B-1: Word Rendering with OVP Anchoring
- Implement `calculate_anchor_position(word)` function
- Apply PRD Midnight theme colors (#1A1B26 bg, #A9B1D6 text, #F7768E anchor)
- Center word on screen
- Implement OVP positioning logic (1/2/3/4 char based on length)

### Task 2B-2: Reading Mode - Timing Loop
- Implement tick-based event loop with deadline calculation
- Handle Space key for pause/resume
- Handle 'q' key to return to REPL
- Calculate `next_word_deadline` based on current WPM
- Recalculate deadline on keypress (early wake-up handling)

### Task 2B-3: Fix Tech Debt - WPM Timing
- Modify `tokenize_text(text, wpm)` to accept WPM parameter
- Use `wpm_to_milliseconds(wpm)` for base delay
- Update all call sites in `main.rs`
- Allow runtime WPM changes without re-tokenizing

### Task 2B-4: WPM Controls (Reading Mode)
- Add MIN_WPM (50) and MAX_WPM (1000) constants
- Implement `recalculate_current_word_timing()` method
- Update `adjust_wpm()` to recalculate timing
- Clamp WPM values to min/max bounds

### Task 2B-5: Progress Indication
- Implement `progress()` method returning (current, total)
- Display word count: "current / total"
- Display percentage: " (XX%)"
- Display current WPM at bottom right
- Position progress at bottom of screen

---

## Epic 2C: Hardening & Platform Support (3 tasks)

**Goal:** Validate functionality and ensure platform compatibility. Robust terminal cleanup.

**Completion Criteria:**
- All features tested manually
- Terminal cleanup works on Ctrl+C and panic
- Platform-specific issues identified and documented
- Windows/Linux/macOS compatibility verified

**Tasks:**

### Task 2C-1: Integration Tests with Human Testing
- Create `tests/test_helpers.rs` (UserAction enum, mock terminal)
- Create `tests/integration_manual.md` (manual testing instructions)
- Add test mode flag to `main.rs` (SPEEDY_TEST env var)
- Implement `run_test_mode()` for simulated user actions
- Create TC1-TC5 test cases (REPL, pause/resume, WPM, quit, panic)

### Task 2C-2: Platform-Specific Terminal Handling
- Add terminal resize event handling
- Add Ctrl+C graceful exit
- Implement Windows line ending handling
- Update `TerminalGuard` for platform-specific cursor restoration
- Verify terminal restore on panic across platforms

### Task 2C-3: Issue Resolution & Reserve Slot
- Reserve slot for issues discovered during Epic 2B implementation
- Document any platform-specific bugs in `docs/platform_issues.md`
- Create follow-up beads as needed

---

## Human Testing Beads (5 beads)

After completing each epic, create these beads for manual verification:

### Bead 2A-HT1: REPL Basic Functionality
- Verify REPL prompt appears
- Load a sample file (@test.txt)
- Type `:q` to quit
- Verify clean terminal exit

### Bead 2B-HT1: Reading Experience Smoke Test
- Load a test document
- Verify words display with OVP anchoring
- Test pause/resume with Space key
- Test WPM adjustment with `[` / `]`
- Verify progress shows at bottom

### Bead 2B-HT2: Reading Completion
- Read a longer document
- Verify reading completes (end of file)
- Verify return to REPL after completion
- Test WPM throughout reading session

### Bead 2C-HT1: Platform Compatibility
- Run all 2A and 2B tests on Linux/macOS
- Note any platform-specific issues
- If Windows available, test on Windows
- Document results in `tests/integration_results.md`

### Bead 2C-HT2: Terminal Cleanup Validation
- Trigger intentional panic (insert code to crash)
- Verify terminal restores to normal state
- Test Ctrl+C during different modes
- Confirm no terminal corruption

---

## Overall Acceptance Criteria

Epic 2 series is complete when:

- [ ] Epic 2A: All 4 tasks completed, tested, committed
- [ ] Epic 2B: All 5 tasks completed, tested, committed
- [ ] Epic 2C: All 3 tasks completed, tested, committed
- [ ] All 5 human testing beads completed and documented
- [ ] All existing tests from Epic 1 still pass
- [ ] Binary compiles with `cargo build --release`
- [ ] `cargo test` shows all tests passing
- [ ] Integration results documented in `tests/integration_results.md`

---

## Architecture Validation

### Pure Core Separation
- [ ] `App` uses `AppEvent` enum (domain events), not `crossterm::KeyEvent`
- [ ] `App` exposes `handle_event()` for state transitions
- [ ] `App` exposes `get_render_state()` with `RenderState` struct (no ratatui types)
- [ ] `main.rs` acts as "shell" adapter only

### Event Loop Architecture
- [ ] Uses `crossterm::event::poll(timeout)` for tick-based approach
- [ ] Calculates `next_word_deadline` based on WPM
- [ ] Recalculates deadline on keypress (handles early wake-up)
- [ ] Single-threaded, no async complexity

### Test Strategy
- [ ] Core logic tested with unit tests (no I/O)
- [ ] Manual testing documented and results tracked
- [ ] Platform compatibility verified

---

## References

- PRD Section 6.1: Architecture (pure core + thin IO)
- PRD Section 6.3: Technical Implementation (double-buffering, accessibility)
- PRD Section 7.1: REPL Mode keybindings
- PRD Section 7.2: Reading Mode keybindings
- PRD Section 3.1: OVP Anchoring
- PRD Section 4.1: Midnight Theme colors

---

**Plan created:** 2026-01-22
**Total Scope:** 12 development tasks + 5 human testing beads
