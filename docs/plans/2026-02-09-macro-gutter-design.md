# Design Document: Macro Gutter Feature

**Date:** 2026-02-09  
**Feature:** Document Progress Bar (Macro Gutter)  
**Status:** Design Complete - Ready for Implementation

---

## 1. Overview

The macro gutter is a 4px vertical progress bar displayed on the right edge of the reader zone. It provides document-level spatial awareness by showing the reader's current position within the document.

### Purpose
- Visual indicator of reading progress through the document
- Helps maintain spatial awareness during speed reading
- Provides subtle feedback about document length and remaining content

---

## 2. Architecture

### 2.1 Rendering Approach

**Kitty Graphics Protocol - Method on KittyGraphicsRenderer**

- **Rationale:** Consistent with existing `render_bar()` pattern, keeps all Kitty rendering in one place
- **Implementation:** Add `render_macro_gutter()` method to `KittyGraphicsRenderer`
- **Positioning:** Absolute coordinates within reader zone only

### 2.2 Positioning

**Inside Reader Zone Only**

- Located at the **rightmost 4px** of the reader zone
- **Does NOT extend** into the 5-line command section at the bottom
- Height = reader zone height (terminal height minus command section)

```
┌──────────────────────────────────┬─┐
│                                  │ │
│      Reader Zone                 │█│
│      (Words display here)        │█│ 4px
│                                  │█│ Macro
│                                  │█│ Gutter
│                                  │█│
│                                  │█│
├──────────────────────────────────┴─┤
│ Command Section (5 lines)          │  <-- NO GUTTER HERE
└────────────────────────────────────┘
```

**The gutter occupies 4px within the reader zone width**, reducing the available space for word display by 4px. Words should not overlap with the gutter.

### 2.3 Visual Design

**Solid Fill from Top**

- **Style:** Simple solid color bar
- **Fill direction:** Top-to-bottom
- **Progress calculation:** `current_word_index / (total_words - 1)`
- **Colors:** Theme accent color (refer to PRD for specific hex)

**Alpha-Based States**

- **Reading Mode:** 30% opacity (dimmed - lets reader focus on words)
- **Paused Mode:** 100% opacity (bright - indicates pause state)
- **Alpha values:** 77/255 for dimmed, 255/255 for bright

---

## 3. Data Flow

### 3.1 Render Separation

The macro gutter is rendered as a **completely independent call** from both word rendering and micro bar rendering. This is a three-step render process in the main loop:

```rust
// In main render loop (e.g., src/ui/terminal.rs):

// 1. Render the current word (separate call)
self.kitty_renderer.render_word(word, anchor_pos)?;

// 2. Render the micro bar - sentence progress (separate call)
self.kitty_renderer.render_bar(word_y, word_height, progress, bar_image_id)?;

// 3. Render the macro gutter - document progress (separate call)
self.kitty_renderer.render_macro_gutter(
    current_word_index,
    total_words,
    reader_area,
    mode,
    image_id
)?;
```

**Key Principle:** Each visual element (word, micro bar, macro gutter) is rendered independently with its own Kitty image transmission. This ensures:
- Clear separation of concerns
- Independent positioning and lifecycle
- Easier testing and debugging
- No coupling between components

### 3.2 Data Flow Diagram

```
┌──────────────┐     ┌──────────────────────────┐     ┌──────────────────┐
│     App      │     │  KittyGraphicsRenderer   │     │  Kitty Protocol  │
├──────────────┤     ├──────────────────────────┤     ├──────────────────┤
│              │     │                          │     │                  │
│ current_word ├───► │  render_macro_gutter()   │     │                  │
│              │     │  - Calculate fill height │     │                  │
│ total_words  ├───► │  - Apply alpha by mode   │───► │  Transmit image  │
│              │     │  - Position at edge      │     │  with position   │
│     mode     ├───► │                          │     │                  │
├──────────────┤     ├──────────────────────────┤     ├──────────────────┤
│   Viewport   │     │                          │     │                  │
│ reader_area  ├───► │                          │     │                  │
│              │     │                          │     │                  │
└──────────────┘     └──────────────────────────┘     └──────────────────┘
```

---

## 4. API Specification

### 4.1 Method Interface

Add to `src/rendering/kitty/mod.rs` in `impl KittyGraphicsRenderer`:

```rust
impl KittyGraphicsRenderer {
    /// Render document progress macro gutter
    ///
    /// Displays a 4px vertical bar on the right edge of the reader zone
    /// showing overall document progress. Alpha varies by mode:
    /// - Reading: 30% opacity (dimmed)
    /// - Paused: 100% opacity (bright)
    ///
    /// # Arguments
    /// * `current_word` - Current word index (0-based)
    /// * `total_words` - Total number of words in document
    /// * `reader_area` - Pixel dimensions of reader zone (x, y, width, height)
    /// * `mode` - Current app mode (Reading or Paused)
    /// * `image_id` - Unique image ID for this gutter instance
    pub fn render_macro_gutter(
        &mut self,
        current_word: usize,
        total_words: usize,
        reader_area: Rect,
        mode: AppMode,
        image_id: u32,
    ) -> Result<(), RendererError>;
}
```

### 4.2 Key Calculations

**Progress Height:**
```rust
let progress_ratio = if total_words > 1 {
    current_word as f32 / (total_words - 1) as f32
} else {
    0.0
};
let fill_height = (reader_area.height as f32 * progress_ratio) as u32;
```

**Alpha Value:**
```rust
let alpha = match mode {
    AppMode::Paused => 255,  // 100% opacity
    _ => 77,                 // 30% opacity
};
```

**Position:**
```rust
// Gutter occupies rightmost 4px INSIDE the reader zone
let x_position = reader_area.x + reader_area.width - 4;  // Right edge minus 4px
let y_position = reader_area.y;                          // Top of reader zone
```

**Note:** The reader zone width available for word rendering should account for the 4px gutter. Either:
- Pass a reduced width to `render_word()` (reader_area.width - 4), OR
- Position words knowing the rightmost 4px is reserved for the gutter

---

## 5. Implementation Plan

### 5.1 Modified Files

1. **`src/rendering/kitty/mod.rs`**
   - Add `render_macro_gutter()` method to `impl KittyGraphicsRenderer`
   - Implement RGBA buffer generation with alpha blending
   - Use existing Kitty protocol utilities (transmit_graphics)
   - Position at right edge of reader zone

2. **`src/ui/terminal.rs`** (Main integration point)
   - Call `kitty_renderer.render_macro_gutter()` as **third independent call**
   - Call after `render_word()` and `render_bar()`
   - Pass required state (current_word, total_words, mode, reader_area)
   - Use unique image_id (increment after call)

3. **`src/rendering/viewport.rs`**
   - Ensure reader area dimensions available
   - Exclude command section from reader_area calculation

### 5.2 Implementation Steps

1. **Add method to KittyGraphicsRenderer** (`src/rendering/kitty/mod.rs`)
   - Define `render_macro_gutter()` method
   - Create RGBA buffer (4px wide × reader_zone_height tall)
   - Fill from top to progress position with accent color
   - Apply alpha based on mode (30% Reading, 100% Paused)
   - Position at right edge using viewport coordinates
   - Transmit via Kitty protocol with unique image_id

2. **Integration** (in `src/ui/terminal.rs`)
   - In render loop, add third render call:
     ```rust
     // After render_word and render_bar:
     let gutter_id = self.kitty_renderer.current_image_id;
     if let Err(e) = self.kitty_renderer.render_macro_gutter(
         reading_state.current_index,
         reading_state.tokens.len(),
         reader_area,
         app.mode,
         gutter_id,
     ) {
         app.set_error(format!("Gutter render error: {}", e));
     } else {
         self.kitty_renderer.current_image_id += 1;
     }
     ```
   - Access app state for progress info
   - Get reader_area from viewport (excluding command section)

3. **Testing**
   - Unit tests for progress calculation in KittyGraphicsRenderer
   - Manual verification of positioning
   - Check mode switching (alpha changes)

---

## 6. Error Handling

| Scenario | Handling |
|----------|----------|
| **Empty document** (total_words = 0) | Render empty gutter (0% fill) |
| **Division by zero** | Guard with `total_words > 1` check |
| **Kitty protocol failure** | Log error, continue reading (non-fatal) |
| **Bounds overflow** | Clamp fill_height to gutter height |
| **Invalid coordinates** | Validate before Kitty transmission |

---

## 7. Testing Strategy

### 7.1 Unit Tests

Add tests in `src/rendering/kitty/mod.rs`:

```rust
#[test]
fn test_macro_gutter_progress_calculation_0_percent() {
    let height = calculate_fill_height(0, 100, 100);
    assert_eq!(height, 0);
}

#[test]
fn test_macro_gutter_progress_calculation_50_percent() {
    let height = calculate_fill_height(50, 100, 100);
    assert_eq!(height, 50);
}

#[test]
fn test_macro_gutter_progress_calculation_100_percent() {
    let height = calculate_fill_height(99, 100, 100);
    assert_eq!(height, 100);
}

#[test]
fn test_macro_gutter_alpha_values() {
    assert_eq!(get_alpha(AppMode::Paused), 255);
    assert_eq!(get_alpha(AppMode::Reading), 77);
}
```

### 7.2 Manual Testing

1. **Positioning verification:**
   - Load document in terminal
   - Verify gutter is at right edge of reader zone
   - Confirm gutter does NOT extend into command section
   - Resize terminal, verify gutter stays in correct position

2. **Progress verification:**
   - Start at beginning, gutter should be empty
   - Navigate to middle, gutter should be ~50% filled
   - Navigate to end, gutter should be completely filled

3. **Mode switching:**
   - In Reading mode, gutter should be dimmed
   - Press Space to pause, gutter should brighten
   - Resume reading, gutter should dim again

---

## 8. Acceptance Criteria

- [ ] 4px vertical gutter visible at right edge of reader zone
- [ ] Gutter fill represents current document progress (0% to 100%)
- [ ] Gutter does NOT extend into command section (bottom 5 lines)
- [ ] Reading mode: gutter at 30% opacity
- [ ] Paused mode: gutter at 100% opacity
- [ ] Progress updates correctly when navigating words
- [ ] Works with documents of any size (handles edge cases)
- [ ] No visual glitches during terminal resize
- [ ] All tests passing

---

## 9. References

- Original Design Doc: `docs/plans/2026-01-28-TUI Design Doc v2.md` (Section 5.1)
- PRD: `@PRD.md` (Theme colors)
- Kitty Graphics Protocol: Existing implementation in `src/rendering/kitty/`

---

**Decision Log:**
- **Rendering:** Kitty Graphics via method on KittyGraphicsRenderer (consistent with render_bar)
- **Positioning:** Inside reader zone only, NOT extending to command section
- **Visual style:** Solid fill from top (not gradient or segments)
- **State indication:** Alpha-based (transparency), not color-based
- **Render Separation:** Independent call from word rendering and micro bar rendering (third render call in main loop)
- **Architecture:** Method on KittyGraphicsRenderer (not separate module)
