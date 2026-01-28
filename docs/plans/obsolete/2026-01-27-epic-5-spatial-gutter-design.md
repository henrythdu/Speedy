# Epic 5: Full-Height Gutter & Spatial Progress

## Overview

Implements full-height gutter with top-down progress filling and bookmark visualization for enhanced spatial awareness and document navigation.

## Problem Statement

**Current State:**
- Gutter is just a simple vertical line `│` (placeholder)
- No progress indication in gutter
- Users can't see document position at a glance
- No bookmark functionality visible

**User Need:**
- See document progress without moving eyes from reading position
- Visual orientation within long documents
- Bookmark positions for quick navigation reference

**Solution:**
- Full-height gutter (extends entire reading area)
- Progress fills from top down (current position)
- Bookmark line shows saved positions
- Subtle, non-distracting presentation

## Detailed Design

### Epic Scope

**3 Beads (Focused Implementation)**

### Bead 5-1: Extend Gutter to Full Height

**Current Implementation:**
```rust
// src/ui/render.rs
pub fn render_gutter_placeholder() -> Paragraph<'static> {
    Paragraph::new("│")
        .alignment(Alignment::Right)
}
```

**New Implementation:**
```rust
pub fn render_gutter(
    tokens_len: usize,
    current_index: usize,
    mode: AppMode,
) -> Paragraph<'static> {
    let gutter_height = calculate_gutter_height(); // Based on terminal size
    
    let progress_fill = (current_index as f64 / tokens_len as f64 * gutter_height as f64) as usize;
    
    let mut spans = Vec::new();
    for i in 0..gutter_height {
        if i < progress_fill {
            // Filled portion (current progress)
            spans.push(Span::styled("█", Style::default().fg(
                if mode == AppMode::Reading { colors::dimmed() } else { colors::text() }
            )));
        } else {
            // Empty portion (remaining)
            spans.push(Span::styled("░", Style::default().fg(colors::dimmed())));
        }
        spans.push(Span::raw("\n"));
    }
    
    Paragraph::new(Line::from(spans))
}
```

**Layout Integration:**
```rust
// src/ui/terminal.rs
fn render_frame(&mut self, app: &App) {
    let gutter_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)])
        .split(main_area)[1];
    
    let render_state = app.get_render_state();
    let gutter = render_gutter(
        render_state.tokens.len(),
        render_state.current_index,
        render_state.mode,
    );
    frame.render_widget(gutter, gutter_area);
}
```

**Key Requirements:**
- Gutter height matches reading area height (not fixed at 100 lines)
- Dynamically adjusts to terminal size
- Progress fill recalculates on terminal resize
- Renders efficiently (not 60 FPS updates - only on progress change)

**Acceptance Criteria:**
- Gutter extends from top to bottom of reading area
- Resizes correctly when terminal resizes
- Renders without performance impact (caches calculation)
- Shows vertical progress bar with █ (filled) and ░ (empty)

### Bead 5-2: Top-Down Progress Fill Algorithm

**Mathematical Implementation:**
```rust
fn calculate_progress_fill(
    current_index: usize,
    total_tokens: usize,
    gutter_height: usize,
) -> usize {
    if total_tokens == 0 {
        return 0;
    }
    
    // Current position as percentage
    let progress_percent = current_index as f64 / total_tokens as f64;
    
    // Convert to gutter height
    (progress_percent * gutter_height as f64).floor() as usize
}

fn render_gutter_with_progress(
    tokens: &[Token],
    current_index: usize,
    gutter_height: usize,
) -> Vec<Span<'static>> {
    let filled_lines = calculate_progress_fill(current_index, tokens.len(), gutter_height);
    
    (0..gutter_height)
        .map(|i| {
            if i < filled_lines {
                Span::styled("█", Style::default().fg(colors::dimmed()))
            } else {
                Span::styled("░", Style::default().fg(colors::dimmed()))
            }
        })
        .collect()
}
```

**Edge Cases to Handle:**
- Empty document (0 tokens) → show all ░
- Single word document → show 1 █, rest ░
- Current position at end → all █
- Mid-document → appropriate proportion

**Optimization:**
- Only recalculate when:
  - Current_index changes
  - Terminal is resized
  - Not on every render frame (60 FPS)

**Acceptance Criteria:**
- Progress fills from top (position 0) down
- Accurate representation of current position
- Handles all edge cases (empty, single word, end of doc)
- Performance: Doesn't recalculate unnecessarily

### Bead 5-3: Bookmark Visualization

**Bookmark Data Model:**
```rust
// In App or ReadingState
pub struct Bookmark {
    pub position: usize,        // Token index
    pub label: Option<String>,  // Optional label (future)
}

// App has Option<Bookmark> for now (single bookmark MVP)
pub bookmark: Option<Bookmark>
```

**Rendering Bookmark in Gutter:**
```rust
pub fn render_gutter_with_bookmark(
    tokens_len: usize,
    current_index: usize,
    bookmark: Option<&Bookmark>,
    mode: AppMode,
) -> Paragraph<'static> {
    let gutter_height = calculate_gutter_height();
    let filled_lines = calculate_progress_fill(current_index, tokens_len, gutter_height);
    
    let bookmark_line = bookmark.map(|b| {
        calculate_progress_fill(b.position, tokens_len, gutter_height)
    });
    
    let mut spans = Vec::new();
    for i in 0..gutter_height {
        let span = match (i < filled_lines, Some(i) == bookmark_line) {
            (true, true) => Span::styled("━", Style::default().fg(colors::anchor())),
            (true, false) => Span::styled("█", Style::default().fg(colors::dimmed())),
            (false, true) => Span::styled("━", Style::default().fg(colors::anchor())),
            (false, false) => Span::styled("░", Style::default().fg(colors::dimmed())),
        };
        spans.push(span);
        spans.push(Span::raw("\n"));
    }
    
    let style = if mode == AppMode::Reading {
        Style::default().fg(colors::dimmed())
    } else {
        Style::default().fg(colors::text())
    };
    
    Paragraph::new(Line::from(spans)).style(style)
}
```

**Visual Design:**
```
Bookmarks shown as: ━ (horizontal line)
Progress shown as:  █ (filled)
Empty shown as:     ░ (empty)

Example with bookmark at 30%:
┌────────────────┬─────┐
│                │  █░░│
│                │  █░░│    Progress: 20% (current)
│                │  █░░│    Bookmark: 30% (future position)
│    hello       │  █░░│
│    ───────     │  █░░│
│                │  █░░│
│                │ ━░░│  ← Bookmark line (horizontal)
│                │  ░░│
│                │  ░░│
│                │  ░░│
└────────────────┴─────┘
```

**Key Decisions:**
- Single bookmark for MVP (Epic 4)
- Bookmark line distinct from progress fill (horizontal marker)
- Bookmark color: anchor color (#F7768E) for consistency
- Only visible when bookmark set

**Future Extensions (Not in Epic 5):**
- Multiple bookmarks (Vec<Bookmark>)
- Bookmark labels/names
- Jump to bookmark command
- Bookmark persistence to file

**Acceptance Criteria:**
- Bookmark line appears at correct position in gutter
- Horizontal marker distinct from vertical progress fill
- Only shows when bookmark is Some(not None)
- Uses anchor color (coral red) per PRD specification

## Integration with Layout

**Terminal Layout Update:**
```rust
fn render_frame(&mut self, app: &App) {
    self.terminal.draw(|frame| {
        let area = frame.area();
        
        // Split: main area (left) + gutter area (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(95),  // Main reading area
                Constraint::Percentage(5),   // Gutter (was 3%, now wider)
            ])
            .split(area);
        
        let main_area = chunks[0];
        let gutter_area = chunks[1];
        
        // Render main reading area (same as before)
        self.render_main_content(frame, app, main_area);
        
        // Render gutter with progress
        let render_state = app.get_render_state();
        let gutter = render_gutter(
            render_state.tokens.len(),
            render_state.current_index,
            app.bookmark.as_ref(),
            render_state.mode,
        );
        frame.render_widget(gutter, gutter_area);
    })
}
```

**Key Changes from Epic 3:**
- Gutter width increased from 3% to 5% (wider for better visibility)
- Gutter renders full-height, not placeholder
- Progress calculation integrated
- Bookmark visualization added

## Implementation Order

**Depends on:** Epic 4 (Command Mode)
- Need command mode to add bookmark commands later
- Layout changes build on Epic 4's rendering work

**Within Epic 5:**
1. Extend gutter to full height (simple layout change)
2. Add progress fill algorithm (medium complexity)
3. Add bookmark visualization (requires bookmark data from Epic 4)

## Testing Checklist

- [ ] Gutter extends full height of reading area
- [ ] Progress fills from top down accurately
- [ ] Gutter resizes when terminal resizes
- [ ] Performance: No lag when scrolling (1000+ token documents)
- [ ] Bookmark line appears at correct position
- [ ] Bookmark line distinct from progress fill visually
- [ ] Multiple scroll/resize cycles don't cause rendering artifacts

## Dependencies

**Builds on:** Epic 4 (Integrated Command Mode)
- Uses command mode infrastructure
- Bookmarks set via future Epic 4 commands

**No external crates:** Pure ratatui rendering

## Testing Strategy

**Performance Testing:**
- Test with 10,000+ token document
- Verify no frame rate drops during rapid scrolling
- Check memory usage doesn't grow (caches calculation)

**Visual Testing:**
- Verify at multiple terminal sizes (20x10 to 200x50)
- Check terminal resize behavior
- Ensure no visual artifacts during mode transitions

## Risks

**Medium Risk** - Complex rendering logic

**Mitigations:**
- Calculate gutter height once per resize (not per frame)
- Cache progress calculations
- Profile with large documents
- Handle edge cases (empty doc, single word)

## Backwards Compatibility

**No Breaking Changes** - Purely additive visual enhancement

**Visual Changes:**
- Gutter now shows content (was placeholder)
- Users will notice new visual element (positive change)

## Future Extensions (Not in Epic 5)

- Multiple bookmarks (vertical list in gutter)
- Clickable gutter for navigation (if terminal supports mouse)
- Chapter/section markers in gutter
- Reading statistics (WPM chart) in gutter
- Customizable gutter width and colors

## Summary

Epic 5 adds sophisticated spatial awareness features:

1. **Full-Height Gutter** - Continuous progress visualization
2. **Top-Down Progress Fill** - Accurate document position representation
3. **Bookmark Visualization** - Visual anchor points for navigation

This transforms Speedy from a simple RSVP reader to a document navigator with spatial awareness, following the PRD vision of solving "Spatial Blindness."