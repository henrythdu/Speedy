# Terminal Resize Implementation Plan

**Date:** 2026-02-02
**Purpose:** Fix terminal resize handling and layout calculations

---

## Current State Analysis

### Bug: Horizontal Center Not Dynamic
- **Location:** `src/rendering/kitty.rs:125`
- **Issue:** `reading_zone_center.0` set once at startup, never updated
- **Impact:** Word appears off-center after terminal resize

### Layout Mismatch
- **Docs say:** 85% Reading / 15% Command (percentage-based)
- **Code does:** Dynamic Reading / Fixed 5 lines Command
- **Action:** Update docs to match code

---

## Implementation Tasks

### 1. Fix Horizontal Center Calculation (CRITICAL)
**File:** `src/rendering/kitty.rs`

Change `calculate_start_x()` to calculate X dynamically:
```rust
// OLD (buggy):
let center_x = self.reading_zone_center.0 as f32;

// NEW (correct):
let center_x = self.viewport
    .get_dimensions()
    .map(|dims| dims.pixel_size.0 / 2)
    .unwrap_or(0) as f32;
```

### 2. Add Event::Resize Handler
**File:** `src/ui/terminal.rs`

Add to event loop:
```rust
match event.read()? {
    Event::Resize(cols, rows) => {
        self.handle_resize(cols, rows, app)?;
    }
    // ... existing handlers
}
```

### 3. Implement Resize Handler Method
**File:** `src/ui/terminal.rs`

```rust
fn handle_resize(&mut self, cols: u16, rows: u16, app: &mut App) -> io::Result<()> {
    // Enforce minimum size
    if cols < 80 || rows < 24 {
        return Ok(()); // Ignore resize below minimum
    }
    
    // Auto-pause if reading
    let was_reading = app.mode() == AppMode::Reading;
    if was_reading {
        app.toggle_pause();
    }
    
    // Update viewport dimensions
    self.kitty_renderer.viewport().query_dimensions()?;
    
    // Clear and redraw
    self.render_frame(app)?;
    
    // Resume if was reading
    if was_reading {
        app.toggle_pause();
    }
    
    Ok(())
}
```

### 4. Update Documentation
**Files:** `docs/PRD.md`, `docs/ARCHITECTURE.md`

Change layout description from:
- "Reader Zone occupies **top 85%** of terminal"
- "Command Section occupies **bottom 15%** (~5 lines)"

To:
- "Reader Zone occupies **all space above command deck**"
- "Command Section occupies **fixed 5 lines** at bottom"

---

## Implementation Order

1. Fix horizontal center calculation (kitty.rs)
2. Add Event::Resize handler (terminal.rs)
3. Implement handle_resize method (terminal.rs)
4. Run tests to verify
5. Update PRD.md
6. Update ARCHITECTURE.md
7. Run code review
8. Final logic review

---

## Testing Checklist

- [ ] Word centers correctly at startup
- [ ] Word re-centers after terminal resize
- [ ] Font size stays constant (Option A)
- [ ] Auto-pause/resume works during resize
- [ ] Minimum size (80×24) enforced
- [ ] No visual artifacts during resize
- [ ] All 166 tests pass

---

## Design Decisions Confirmed

1. **Font Size:** Stays constant (calculated once at startup)
2. **Pause Behavior:** Auto-pause on resize, auto-resume after
3. **Minimum Size:** 80 columns × 24 rows
4. **Command Zone:** Fixed 5 lines (not percentage)
5. **Debouncing:** None (handle every resize event)

---

## Notes

- Font size remains constant even when terminal shrinks
- Text may overflow if terminal becomes very small (acceptable for MVP)
- Command deck always visible at bottom
- Reading zone expands/contracts based on terminal size
