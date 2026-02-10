# UI Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add margins, full-height accent bar, bottom-aligned label, and blinking cursor to match opencode's polished UI

**Architecture:** Use ratatui's nested Layout constraints for asymmetric margins (3-cell top/bottom, 1-cell sides), render accent bar as 1-cell-wide column across command section height, implement cursor blink state in TuiManager with 500ms wall-clock timing

**Tech Stack:** Rust, ratatui (TUI framework), crossterm (terminal I/O)

**Design Doc:** @docs/plans/2026-02-10-ui-polish-design.md

---

## Task 1: Add Asymmetric Margins to Layout

**Files:**
- Modify: `src/ui/terminal.rs:186-210` (render_frame layout)
- Test: Run app, visually verify margins appear

**Step 1: Add margin calculations**

Modify `render_frame()` in `src/ui/terminal.rs` to add nested layouts:

```rust
// Current code (simplified):
let main_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(1), Constraint::Length(5)])
    .split(area);

// New code with margins:
// First apply vertical margins (3 cells top/bottom)
let vertical_margins = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Top margin
        Constraint::Fill(1),    // Content
        Constraint::Length(3),  // Bottom margin
    ])
    .split(area);

let content_area = vertical_margins[1];

// Then apply horizontal margins (1 cell left/right)
let horizontal_margins = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(1),  // Left margin
        Constraint::Fill(1),    // Content
        Constraint::Length(1),  // Right margin
    ])
    .split(content_area);

let inner_area = horizontal_margins[1];

// Split inner area into reading and command sections
let main_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(1), Constraint::Length(5)])
    .split(inner_area);
```

**Step 2: Test margin rendering**

Run: `cargo run`
Expected: See dark margins around the UI edges (3 cells top/bottom, 1 cell sides)

**Step 3: Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat: add asymmetric margins (3-cell top/bottom, 1-cell sides)"
```

---

## Task 2: Add Terminal Size Guards for Margins

**Files:**
- Modify: `src/ui/terminal.rs` (add margin calculation helper)
- Test: Resize terminal to <15 rows, verify margins shrink

**Step 1: Create margin calculation helper**

Add helper function before `render_frame()`:

```rust
/// Calculate responsive margins based on terminal size
/// Returns (top_margin, bottom_margin, left_margin, right_margin)
fn calculate_margins(area: Rect) -> (u16, u16, u16, u16) {
    const MIN_HEIGHT_FOR_FULL_MARGINS: u16 = 15;
    const TARGET_TOP_BOTTOM: u16 = 3;
    const TARGET_LEFT_RIGHT: u16 = 1;
    
    let height = area.height;
    
    if height >= MIN_HEIGHT_FOR_FULL_MARGINS {
        // Full margins for large terminals
        (TARGET_TOP_BOTTOM, TARGET_TOP_BOTTOM, TARGET_LEFT_RIGHT, TARGET_LEFT_RIGHT)
    } else if height >= 10 {
        // Reduced margins for medium terminals
        (1, 1, TARGET_LEFT_RIGHT, TARGET_LEFT_RIGHT)
    } else {
        // Minimal margins for small terminals
        (0, 0, 0, 0)
    }
}
```

**Step 2: Update render_frame to use helper**

Replace hardcoded margins with:

```rust
let (top, bottom, left, right) = calculate_margins(area);

let vertical_margins = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(top),
        Constraint::Fill(1),
        Constraint::Length(bottom),
    ])
    .split(area);
// ... rest of layout code using left/right
```

**Step 3: Test responsive margins**

Run: `cargo run`
Steps:
1. Start with large terminal (see 3-cell margins)
2. Resize to 12 rows (see 1-cell margins)
3. Resize to 8 rows (see no margins)

**Step 4: Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat: add responsive margin sizing for small terminals"
```

---

## Task 3: Implement Full-Height Accent Bar

**Files:**
- Modify: `src/ui/reader/view.rs:12-60` (render_command_deck function)
- Test: Run app, verify red bar extends full 5-cell height

**Step 1: Update render_command_deck layout**

Current layout splits horizontally for accent bar. Modify to create full-height bar:

```rust
pub fn render_command_deck(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    command_buffer: &str,
    error_message: Option<&str>,
    _cursor_visible: bool,  // Will use in Task 5
) {
    // Clear the command area first
    frame.render_widget(Clear, area);

    // Split into accent bar (1 cell) and content area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let accent_area = layout[0];
    let content_area = layout[1];

    // Render full-height accent bar using block character
    // Create multi-line string with accent character for full height
    let accent_text = "▌\n▌\n▌\n▌\n▌";  // One per row
    let accent_bar = Paragraph::new(accent_text)
        .style(Style::default().fg(colors::anchor()));
    frame.render_widget(accent_bar, accent_area);

    // Render content in remaining area...
    // (keep existing content rendering logic)
}
```

**Step 2: Test accent bar height**

Run: `cargo run`
Expected: Coral red bar (▌) extends full 5-cell height of command section

**Step 3: Commit**

```bash
git add src/ui/reader/view.rs
git commit -m "feat: extend accent bar to full command section height"
```

---

## Task 4: Implement Bottom-Aligned Label

**Files:**
- Modify: `src/ui/reader/view.rs:30-60` (command content area rendering)
- Test: Run app, verify "COMMAND" appears at bottom of command section

**Step 1: Split content area for bottom alignment**

```rust
// Inside render_command_deck, after accent bar:

// Split content area to put label at bottom
let content_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Min(0),     // Input area (flexible)
        Constraint::Length(1),  // Label row (fixed at bottom)
    ])
    .split(content_area);

let input_area = content_layout[0];
let label_area = content_layout[1];

// Render label at bottom
let mode_label = match mode {
    AppMode::Command => "COMMAND",
    AppMode::Reading => "READING",
    AppMode::Paused => "PAUSED",
    AppMode::Quit => "QUIT",
};

let label_widget = Paragraph::new(mode_label)
    .style(Style::default().fg(colors::anchor()).bg(colors::surface()));
frame.render_widget(label_widget, label_area);

// Render input above label
let input_text = if let Some(error) = error_message {
    format!("ERROR: {}", error)
} else if command_buffer.is_empty() {
    "Type @file.pdf, @@, or :q".to_string()
} else {
    command_buffer.to_string()
};

let text_color = if error_message.is_some() {
    colors::anchor()
} else {
    colors::text()
};

// Input widget without borders (cleaner look)
let input_widget = Paragraph::new(input_text)
    .style(Style::default().fg(text_color).bg(colors::surface()));
frame.render_widget(input_widget, input_area);
```

**Step 2: Remove top border from command deck**

Remove the `.borders(Borders::TOP)` from input_widget to match opencode's cleaner look.

**Step 3: Test bottom alignment**

Run: `cargo run`
Expected: "COMMAND" label appears at bottom of command section, input area above it

**Step 4: Commit**

```bash
git add src/ui/reader/view.rs
git commit -m "feat: bottom-align mode label in command section"
```

---

## Task 5: Implement Cursor Blink State Management

**Files:**
- Modify: `src/ui/terminal.rs:25-35` (TuiManager struct)
- Modify: `src/ui/terminal.rs:70-150` (event loop)
- Test: Verify cursor toggles every 500ms

**Step 1: Add cursor state to TuiManager**

```rust
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    command_buffer: String,
    kitty_renderer: KittyGraphicsRenderer,
    cursor_visible: bool,        // NEW: Blink state
    last_cursor_toggle: Instant, // NEW: Last toggle time
    last_keypress: Instant,      // NEW: For pause-on-type
}
```

**Step 2: Initialize cursor state in constructor**

In `TuiManager::new()`, add:

```rust
Ok(Self {
    terminal,
    command_buffer: String::new(),
    kitty_renderer: renderer,
    cursor_visible: true,                    // Start visible
    last_cursor_toggle: Instant::now(),
    last_keypress: Instant::now(),
})
```

**Step 3: Add cursor toggle logic in event loop**

In `run_event_loop()`, add cursor management:

```rust
// Before the main loop:
const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);
const CURSOR_PAUSE_AFTER_TYPING: Duration = Duration::from_millis(500);

// In the main loop (Command mode section):
if app.mode() == AppMode::Command {
    // Check if we should toggle cursor (only if not recently typing)
    let time_since_keypress = self.last_keypress.elapsed();
    
    if time_since_keypress >= CURSOR_PAUSE_AFTER_TYPING {
        // Time to resume blinking
        if self.last_cursor_toggle.elapsed() >= CURSOR_BLINK_INTERVAL {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_toggle = Instant::now();
            // Force redraw to show/hide cursor
            needs_redraw = true;
        }
    } else {
        // Recently typed - keep cursor visible
        if !self.cursor_visible {
            self.cursor_visible = true;
            needs_redraw = true;
        }
    }
}
```

**Step 4: Update last_keypress on input**

When handling key input in Command mode:

```rust
KeyCode::Char(c) => {
    self.command_buffer.push(c);
    self.last_keypress = Instant::now();  // Reset blink pause
    self.cursor_visible = true;            // Show cursor immediately
}
```

**Step 5: Test cursor blink**

Run: `cargo run`
Steps:
1. Wait 500ms without typing - cursor should toggle on/off
2. Type a character - cursor should immediately appear solid
3. Wait 500ms after stopping - blinking should resume

**Step 6: Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat: implement cursor blink state management with 500ms timing"
```

---

## Task 6: Render Blinking Cursor in Command Deck

**Files:**
- Modify: `src/ui/reader/view.rs:12` (function signature)
- Modify: `src/ui/reader/view.rs:50-60` (input rendering)
- Modify: `src/ui/terminal.rs:200` (render_frame call)
- Test: Verify blinking block cursor appears at end of input

**Step 1: Update function signature**

```rust
pub fn render_command_deck(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    command_buffer: &str,
    error_message: Option<&str>,
    cursor_visible: bool,      // NEW parameter
) {
```

**Step 2: Render cursor in input text**

```rust
// Build input text with cursor
let cursor_char = if cursor_visible { "█" } else { " " };
let input_text = if let Some(error) = error_message {
    format!("ERROR: {}{}", error, cursor_char)
} else if command_buffer.is_empty() {
    format!("Type @file.pdf, @@, or :q{}", cursor_char)
} else {
    format!("{}{}", command_buffer, cursor_char)
};
```

**Step 3: Update render_frame call site**

In `src/ui/terminal.rs`:

```rust
render_command_deck(
    frame,
    command_area,
    app.mode(),
    &self.command_buffer,
    app.get_error(),
    self.cursor_visible,  // NEW: pass cursor state
);
```

**Step 4: Test blinking cursor**

Run: `cargo run`
Expected: 
- Solid block cursor (█) appears at end of input text
- Cursor blinks every 500ms when idle
- Cursor stays solid while typing

**Step 5: Commit**

```bash
git add src/ui/terminal.rs src/ui/reader/view.rs
git commit -m "feat: render blinking solid-block cursor in command deck"
```

---

## Task 7: Cross-Terminal Testing

**Files:**
- Test in: Multiple terminal emulators
- Document: Add test results to this plan

**Step 1: Test in Kitty**

Run: `cargo run` (in Kitty)
Check:
- [ ] Margins appear correctly
- [ ] Accent bar full height
- [ ] Label bottom-aligned
- [ ] Cursor blinks smoothly

**Step 2: Test in tmux**

Run: `cargo run` (in tmux session)
Check:
- [ ] All features work
- [ ] No rendering artifacts

**Step 3: Test in Alacritty**

Run: `cargo run` (in Alacritty)
Check:
- [ ] Block character renders correctly
- [ ] Colors look correct

**Step 4: Test terminal resize**

Steps:
1. Start with large terminal
2. Gradually shrink height to <10 rows
3. Verify margins disappear gracefully
4. Expand back up - margins should return

**Step 5: Document results**

Add test results section to this plan with any issues found.

**Step 6: Commit**

```bash
git commit -m "test: verify UI polish across terminal emulators"
```

---

## Task 8: Quality Gates

**Files:**
- All modified files
- Test suite

**Step 1: Run clippy**

```bash
cargo clippy --all-targets --all-features
```
Expected: 0 warnings

**Step 2: Run tests**

```bash
cargo test
```
Expected: All tests pass

**Step 3: Build check**

```bash
cargo build --release
```
Expected: Clean build

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete UI polish - margins, accent bar, cursor blink

- Add asymmetric margins (3-cell top/bottom, 1-cell sides)
- Implement responsive margin sizing for small terminals
- Extend accent bar to full command section height
- Bottom-align mode label in command section
- Implement blinking solid-block cursor with 500ms timing
- Add cursor blink pause while typing

Closes UI polish design doc implementation."
```

---

## Summary

**Total estimated time:** 2 hours
**Files modified:**
- `src/ui/terminal.rs` - Layout margins, cursor state management
- `src/ui/reader/view.rs` - Accent bar, label alignment, cursor rendering

**Key implementation details:**
- Use nested Layout splits for asymmetric margins
- Wall-clock timing (`Instant::now()`) for cursor blink
- Terminal size guards prevent margin collapse
- Event poll timeout ≤500ms for smooth cursor animation

**Testing requirements:**
- Visual verification in Kitty, tmux, Alacritty
- Terminal resize testing
- Cursor blink behavior verification
