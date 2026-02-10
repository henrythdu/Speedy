# UI Polish: Margin, Command Deck, and Cursor Design

**Date:** 2026-02-10  
**Status:** Design Complete - Ready for Implementation

---

## 1. Overview

Polish the Speedy TUI to match opencode's clean, modern aesthetic with proper margins, full-height accent bars, and a blinking cursor indicator.

### Goals
- Add visual margins around the UI (opencode-style layout)
- Extend accent bar to full command section height
- Reposition "COMMAND" label to bottom (like opencode's "Build")
- Add blinking solid-block cursor

---

## 2. Visual Design

### 2.1 Overall Layout with Asymmetric Margins

```
     [3 cells - background #1A1B26]
┌────────────────────────────────────────────┐
│[1]│                                  │[1]│  ← 1-cell side margins
│   │     READING ZONE                 │   │
│   │     (Maximum width available)    │   │
│   │                                  │   │
│   ├──────────────────────────────────┤   │
│   │▌ COMMAND | Type @file.pdf... █   │   │  ← Command section
│   │                                  │   │
└────────────────────────────────────────────┘
     [3 cells - background #1A1B26]
```

**Margin Specifications:**
- **Top/Bottom:** 3 cells (more breathing room)
- **Left/Right:** 1 cell (maximize reading width)
- **Background:** Theme background color (#1A1B26)

### 2.2 Command Deck Layout

**Structure (bottom-aligned like opencode):**
```
▌ COMMAND | Type @file.pdf, @@, or :q █
```

**Elements:**
1. **Accent Bar (▌)** - Full height of command section, coral red (#F7768E)
2. **Mode Label** - "COMMAND" positioned at bottom of section
3. **Separator** - "|" character with spacing
4. **Input Text** - User's typed command or placeholder
5. **Cursor (█)** - Solid block, white (#FFFFFF), blinking

**Height:** 5 cells (unchanged)
**Accent Bar:** Extends full 5-cell height

### 2.3 Cursor Behavior

**Visual:**
- Character: Full block (█) or similar solid cell-filling character
- Color: White (#FFFFFF) or theme text color
- Width: 1 cell
- Position: Immediately after last typed character

**Behavior:**
- **Blinking:** 500ms on/off cycle when idle
- **Solid:** Stops blinking while typing (immediate feedback)
- **Visibility:** Only shown in Command mode when command deck is focused

---

## 3. Implementation Plan

### 3.1 Modified Files

1. **`src/ui/terminal.rs`**
   - Add margin calculations to layout
   - Update `render_frame()` to apply margins
   - Pass cursor state to command deck renderer

2. **`src/ui/reader/view.rs`**
   - Update `render_command_deck()` signature to accept cursor position
   - Implement full-height accent bar
   - Bottom-align "COMMAND" label
   - Add blinking cursor rendering

3. **`src/ui/terminal.rs`** (TuiManager)
   - Add cursor blink state tracking
   - Toggle cursor visibility on timer

### 3.2 Layout Calculations

**Current Layout (no margins):**
```rust
let main_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(1), Constraint::Length(5)])
    .split(area);
```

**New Layout (with margins):**
```rust
// Outer margin wrapper
let margin_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Top margin
        Constraint::Fill(1),    // Content area
        Constraint::Length(3),  // Bottom margin
    ])
    .split(area);

// Horizontal margins for content area
let content_area = margin_layout[1];
let horizontal_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(1),  // Left margin
        Constraint::Fill(1),    // Actual content
        Constraint::Length(1),  // Right margin
    ])
    .split(content_area);

let inner_area = horizontal_layout[1];

// Split inner area into reading and command sections
let main_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(1), Constraint::Length(5)])
    .split(inner_area);
```

### 3.3 Command Deck Rendering

**Updated signature:**
```rust
pub fn render_command_deck(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    command_buffer: &str,
    error_message: Option<&str>,
    cursor_visible: bool,  // NEW: Blink state
    cursor_position: usize, // NEW: Character position
)
```

**Accent bar implementation:**
- Use a vertical layout with 5 single-cell rows
- Render "▌" in each row with coral red foreground
- Or use a single Paragraph with multi-line content

**Cursor rendering:**
```rust
let cursor_char = if cursor_visible { "█" } else { " " };
let cursor_style = Style::default().fg(Color::White);
```

**Bottom alignment:**
- Calculate text to fill bottom row of command section
- Use `format!("{:<width$}", text)` to left-align at bottom

### 3.4 Cursor Blink Implementation

**In TuiManager:**
```rust
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    command_buffer: String,
    kitty_renderer: KittyGraphicsRenderer,
    cursor_visible: bool,        // NEW
    last_cursor_toggle: Instant, // NEW
}

// In run_event_loop, during Command mode:
if app.mode() == AppMode::Command {
    if self.last_cursor_toggle.elapsed() >= Duration::from_millis(500) {
        self.cursor_visible = !self.cursor_visible;
        self.last_cursor_toggle = Instant::now();
        // Force re-render to show/hide cursor
    }
}
```

**Reset blink on typing:**
```rust
// When key pressed in command mode:
self.cursor_visible = true;
self.last_cursor_toggle = Instant::now();
```

---

## 4. Testing Strategy

### 4.1 Visual Verification

1. **Margin check:**
   - Verify 3-cell top/bottom margins visible
   - Verify 1-cell side margins visible
   - Check background color is consistent

2. **Accent bar check:**
   - Bar extends full 5-cell height
   - Color is coral red (#F7768E)
   - Positioned at left edge of command section

3. **Label alignment:**
   - "COMMAND" appears at bottom of command section
   - Consistent positioning across mode changes

4. **Cursor behavior:**
   - Blinks every 500ms when idle
   - Solid when typing
   - White color, 1 cell width
   - Positioned correctly after input text

### 4.2 Edge Cases

- **Empty command buffer:** Cursor at position 0, placeholder text visible
- **Long commands:** Cursor stays visible, text may scroll or truncate
- **Error state:** Cursor still blinks, error message shown in red
- **Mode transitions:** Cursor only visible in Command mode

---

## 5. Acceptance Criteria

### Core Features
- [ ] 3-cell top/bottom margins visible around UI (with responsive clamping)
- [ ] 1-cell left/right margins visible
- [ ] Accent bar extends full height of command section (5 cells)
- [ ] "COMMAND" label positioned at bottom of command section
- [ ] Blinking solid-block cursor (█) visible in command input
- [ ] Cursor stops blinking while typing (immediate feedback)
- [ ] Cursor resumes blinking 500ms after last keystroke
- [ ] All existing functionality preserved (WPM display, progress bars, etc.)

### Consensus-Driven Safeguards
- [ ] Terminal size guards: Margins clamp to minimum values when terminal height < 15 rows
- [ ] Wall-clock timing for cursor blink (use `Instant::now()`, not tick counting)
- [ ] Event loop poll timeout ≤500ms to support cursor blink timing
- [ ] Cross-terminal testing: Verify in tmux, Windows Terminal, Alacritty, Kitty
- [ ] No clippy warnings
- [ ] Clean build and test

### Future Considerations (Post-MVP)
- [ ] Config option for cursor blink: `cursor_blink = on|off|steady`
- [ ] Alternative label placement (top-aligned) if bottom proves problematic

---

## 6. Implementation Notes

**Why asymmetric margins?**
- Reading needs maximum width for word display
- Vertical breathing room improves reading comfort
- Matches cinematic/widescreen aesthetic

**Why full-height accent bar?**
- Creates stronger visual separation
- Matches opencode's polished look
- Consistent with "Build" section styling

**Why bottom-aligned label?**
- Matches opencode's design pattern
- Creates visual "grounding" at bottom
- More modern than centered or top-aligned

**Why solid block cursor?**
- More visible than thin cursor
- Matches modern terminal emulators
- Clear indication of insertion point

### Consensus Review Findings (Multi-Model)

**Models Consulted:**
- UI/UX Expert (FOR): 9/10 confidence - praises visual hierarchy improvements
- Technical Architect (NEUTRAL): 9/10 confidence - validates ratatui implementation
- Skeptical Reviewer (AGAINST): 8/10 confidence - raises terminal compatibility concerns

**Key Agreements:**
- All changes technically feasible with ratatui/crossterm
- Implementation effort: ~2 hours total
- Full-height accent bar has highest ROI
- Cursor blink requires careful event loop integration

**Critical Safeguards Added:**
1. Terminal size guards (prevent margin collapse on small terminals)
2. Wall-clock timing (prevent cursor drift)
3. Cross-terminal testing requirement
4. Future config option for cursor blink toggle

**Resolution on Disagreements:**
- Asymmetric margins: Proceed with responsive clamping
- Bottom-aligned label: Proceed but consider top-aligned alternative if issues arise
- Blinking cursor: Make toggleable in future, default to blinking for now

---

## References

- opencode UI screenshot for visual reference
- Current Speedy UI screenshot for baseline
- PRD Section 4.1 (Midnight Theme colors)
- `src/ui/terminal.rs` - Main layout and event loop
- `src/ui/reader/view.rs` - Command deck rendering
