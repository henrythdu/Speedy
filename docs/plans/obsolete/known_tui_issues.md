# Known TUI Issues

This document tracks known issues and limitations in the TUI rendering system.

## Current Limitations

### 1. Terminal Size Assumptions
- **Issue:** The TUI layout uses hardcoded percentages (40/20/40 split) that may not work well on very narrow terminals
- **Workaround:** Ensure terminal is at least 80 columns wide
- **Status:** Minor - affects only extremely small terminals

### 2. No Cursor Positioning
- **Issue:** No visual cursor indicator in the word display area
- **Workaround:** The OVP anchor position (coral red) provides visual focus
- **Status:** Design decision - anchor position is sufficient focus indicator

### 3. Single Font Assumption
- **Issue:** Assumes monospace-like character width for OVP anchoring
- **Impact:** Non-monospace fonts may cause visual misalignment
- **Status:** Minor - most terminals use monospace by default

### 4. No Text Selection
- **Issue:** Cannot select or copy text from the TUI display
- **Workaround:** Use REPL mode for text selection
- **Status:** Low priority - TUI is for reading, not text extraction

## Fixed Issues

### 1. Timing Precision at High WPM (FIXED)
- **Original Issue:** Integer division `(60_000 / wpm)` caused cumulative timing drift at high WPM
- **Example:** 165 WPM calculated as 363ms instead of 364ms
- **Fix:** Changed to floating-point: `(60_000.0 / wpm as f64).round() as u64`
- **Affected Bead:** Speedy-6i6

### 2. TUI Integration Borrow Checker (FIXED)
- **Original Issue:** Closure capturing `TuiManager` conflicted with mutable borrow in `run_event_loop`
- **Fix:** Moved render logic directly into closure, avoiding double borrow
- **Affected Bead:** Speedy-nji

### 3. I/O Error Handling in Event Loop (FIXED during pre-commit)
- **Original Issue:** I/O errors from `event::poll()` were silently ignored, treated same as timeout
- **Example:** Terminal connection errors would cause tight loop instead of graceful exit
- **Fix:** Propagate I/O errors by returning `Err(e)` instead of `app.advance_reading()`
- **File:** src/ui/terminal.rs:62-65
- **Status:** FIXED - I/O errors now properly propagated to caller

### 4. Duplicate Rendering Logic (FIXED during pre-commit)
- **Original Issue:** Rendering logic duplicated between `main.rs` closure and `TuiManager::render_frame()`
- **Example:** Layout changes required editing both `main.rs` and `terminal.rs`
- **Fix:** Removed closure parameter from `run_event_loop`, now calls `self.render_frame(app)` internally
- **Files:** src/main.rs, src/ui/terminal.rs
- **Status:** FIXED - All rendering logic centralized in `TuiManager`

## Testing Checklist

### Manual Testing
- [ ] Load PDF/EPUB file and verify text displays correctly
- [ ] Verify OVP anchor moves correctly for words of different lengths
- [ ] Test WPM adjustment with `[` and `]` keys
- [ ] Verify pause/resume with space bar
- [ ] Test quit to REPL with `q` key
- [ ] Verify progress bar updates correctly
- [ ] Test at various WPM settings (100, 300, 600)
- [ ] Verify context words display correctly

### Automated Testing
- [ ] All 112 library tests pass
- [ ] All 7 integration tests pass
- [ ] Timing precision verified for 165, 300, 600 WPM

## Future Improvements

1. **Dynamic Layout:** Adapt layout based on terminal size
2. **Customizable Colors:** Allow user theme customization
3. **Multiple Font Support:** Detect and adapt to non-monospace fonts
4. **Text Selection:** Add mouse-based text selection
5. **Smooth Scrolling:** Animate word transitions (low priority)