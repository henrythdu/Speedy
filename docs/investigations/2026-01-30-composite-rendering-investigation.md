# Investigation: Composite Rendering Implementation

**Date:** 2026-01-30  
**Investigator:** AI Assistant  
**Status:** Investigation Complete - Ready for Implementation Planning

---

## Executive Summary

Current Kitty rendering uses per-word images with a coordinate calculation bug. Moving to composite image approach (single RGBA buffer per frame) fixes the bug and provides an **extensible canvas** for future features. This epic focuses on the **minimum viable composite approach**: background + word centered correctly. Ghost words, anchor coloring, and progress bars are straightforward future enhancements once the canvas exists.

---

## 1. Current Implementation

### 1.1 Per-Word Image Rendering

**Current Architecture:**
```rust
// In KittyGraphicsRenderer::render_word()
1. Create image buffer: word_width × reading_zone_height
2. Fill background: #1A1B26 (theme color)
3. Draw word at vertical offset
4. Transmit via Kitty Graphics Protocol with (x, y) position
5. Clear previous image by ID before next render
```

**Problem:** Words render at wrong vertical position (too low on screen).

### 1.2 Coordinate Calculation Bug

**Current Code:**
```rust
// TuiManager::new() (terminal.rs:58)
let center_y = (dims.pixel_size.1 as f32 * 0.42) as u32;
// ^^ 42% of FULL SCREEN HEIGHT (WRONG)

// calculate_vertical_center() (kitty.rs:92)
let center = (zone_height as f32 * 0.42) as u32;
// ^^ 42% of READING ZONE (85% of screen) - returns relative position
```

**Impact:**
- Words appear near bottom of screen (in/near command area)
- Position calculation uses wrong reference frame (full screen vs reading zone)

---

## 2. Composite Image Approach (Recommended)

### 2.1 What Is Composite Rendering?

**Instead of:** Per-word small images positioned via coordinates
**We do:** Single large image of entire reading zone, word centered in it

**Frame Composition:**
```
┌─────────────────────────────────────────┐
│          READING ZONE (85%)          │
│                                     │
│                                     │
│             [ WORD ]                 │
│          (centered at 42%)            │
│                                     │
│                                     │
└─────────────────────────────────────────┘
│      COMMAND DECK (15%)               │
└─────────────────────────────────────────┘
```

### 2.2 How It Works

```rust
// 1. Create full-zone canvas
let canvas_width = terminal_width_px;
let canvas_height = reading_zone_height_px; // 85% of screen
let mut canvas = ImageBuffer::new(canvas_width, canvas_height);

// 2. Fill background
fill_background(&mut canvas, theme_bg_color);

// 3. Center word vertically at 42%
let word_y = (canvas_height as f32 * 0.42) as i32;

// 4. Calculate word X to center anchor
let word_x = (canvas_width / 2) - (word_width / 2);

// 5. Draw word
draw_text(&mut canvas, word, word_x, word_y);

// 6. Transmit
kitty_protocol.send_image(canvas);

// 7. Clear next frame: delete image ID 1, redraw
```

### 2.3 Benefits

✅ **Fixes Positioning Bug:** Canvas-relative coordinates, no math errors  
✅ **Simpler Clearing:** Single `delete_image(1)` operation  
✅ **Extensible Canvas:** Can add ghost words, anchor coloring, progress bars later  
✅ **Atomic Updates:** All elements update together, no flicker  
✅ **Aligns with Design Doc:** Section 6.2 CPU Compositing pattern  

---

## 3. Implementation Scope

### 3.1 IMMEDIATE GOAL (This Epic)

**Minimum Viable Composite Canvas**

1. **Create ReadingCanvas struct** - Full-zone RGBA buffer
2. **Implement composite_frame()** - Fill background + draw word centered
3. **Fix coordinate bug** - Center at 42% of reading zone height
4. **Simplify clearing** - Delete image ID 1, redraw entire canvas

**Acceptance Criteria:**
- Word appears in middle of reading zone (not bottom)
- Background filled with theme color (#1A1B26)
- Canvas spans full reading zone dimensions
- Clearing works reliably (no ghost images)

### 3.2 FUTURE ENHANCEMENTS (Later Epics)

These are **straightforward additions** once composite canvas exists:

| Feature | Implementation Effort | Description |
|---------|---------------------|-------------|
| **Ghost Words** | 1-2 days | Draw prev/next words at 15% opacity into canvas |
| **Anchor Coloring** | 1 day | Draw word segments with different colors into canvas |
| **Progress Bars** | 1-2 days | Draw macro-gutter + micro-bar into canvas |

**Why Easy:** Canvas is just an RGBA buffer - we can draw whatever we want into it. No architectural changes needed.

---

## 4. Implementation Requirements

### 4.1 New Data Structures

```rust
// src/rendering/kitty.rs

/// Full reading zone canvas
struct ReadingCanvas {
    buffer: RgbaImage,           // RGBA pixel data
    dimensions: (u32, u32),      // Width × Height
}

/// Simplified frame state (for now)
struct RenderFrame {
    current_word: String,
    anchor_position: usize,
}
```

### 4.2 KittyGraphicsRenderer Changes

**Modify Existing Methods:**
```rust
// Rename and repurpose
fn render_frame(&mut self, frame: &RenderFrame) -> Result<(), RendererError> {
    // 1. Clear previous image
    self.clear()?;

    // 2. Create or reuse canvas
    let canvas = self.create_canvas()?;

    // 3. Composite frame elements
    self.composite_word(&mut canvas, &frame)?;

    // 4. Encode and transmit
    self.transmit_canvas(&canvas)?;

    Ok(())
}
```

**Add New Methods:**
```rust
fn create_canvas(&self) -> Result<ReadingCanvas, RendererError> {
    // Get viewport dimensions
    let dims = self.viewport.get_dimensions()
        .ok_or_else(|| RendererError::RenderFailed("No viewport".to_string()))?;

    // Canvas = full width × reading zone height (85%)
    let canvas_width = dims.pixel_size.0;
    let canvas_height = (dims.pixel_size.1 as f32 * 0.85) as u32;

    // Create RGBA buffer
    let buffer = ImageBuffer::new(canvas_width, canvas_height);

    Ok(ReadingCanvas {
        buffer,
        dimensions: (canvas_width, canvas_height),
    })
}

fn composite_word(&self, canvas: &mut ReadingCanvas, frame: &RenderFrame) -> Result<()> {
    // 1. Fill background (#1A1B26)
    let bg_color = Rgba([26, 27, 38, 255]);
    for pixel in canvas.buffer.pixels_mut() {
        *pixel = bg_color;
    }

    // 2. Calculate vertical center (42% of reading zone)
    let word_y = (canvas.dimensions.1 as f32 * 0.42) as i32;

    // 3. Calculate word width
    let word_width = calculate_string_width(self.font.as_ref().unwrap(), &frame.current_word, self.font_size);

    // 4. Calculate word X (center horizontally)
    let word_x = (canvas.dimensions.0 / 2) as i32 - (word_width as i32 / 2);

    // 5. Draw word
    let scale = PxScale::from(self.font_size);
    let text_color = Rgba([247, 118, 142, 255]); // Coral red
    draw_text_mut(&mut canvas.buffer, text_color, word_x, word_y, scale, self.font.as_ref().unwrap(), &frame.current_word);

    Ok(())
}

fn transmit_canvas(&self, canvas: &ReadingCanvas) -> io::Result<()> {
    // Encode to base64
    let base64_data = self.encode_image_base64(&canvas.buffer);

    // Transmit via KGP with position (0, 0) - top-left corner
    self.transmit_graphics(
        1,  // Image ID 1 (fixed)
        canvas.dimensions.0,
        canvas.dimensions.1,
        &base64_data,
        0,   // x = 0 (canvas fills reading zone)
        0,   // y = 0 (top-left alignment)
    )
}
```

**Modify clear():**
```rust
fn clear(&mut self) -> Result<(), RendererError> {
    // Simply delete image ID 1
    self.delete_image(1).map_err(|e| RendererError::ClearFailed(e.to_string()))?;

    // Always use image ID 1 (fixed, not incrementing)
    self.current_image_id = 1;
    Ok(())
}
```

### 4.3 Integration with TuiManager

**Current Code (terminal.rs:220-228):**
```rust
if let Some(word) = &render_state.current_word {
    let anchor_pos = crate::reading::calculate_anchor_position(word);

    // Clear previous graphics
    let _ = RsvpRenderer::clear(&mut self.kitty_renderer);

    // Render word via Kitty Graphics Protocol
    if let Err(e) = RsvpRenderer::render_word(&mut self.kitty_renderer, word, anchor_pos) {
        eprintln!("Kitty render failed: {}", e);
    }
}
```

**New Code:**
```rust
if let Some(word) = &render_state.current_word.clone() {
    let anchor_pos = crate::reading::calculate_anchor_position(word);

    let frame = RenderFrame {
        current_word: word.clone(),
        anchor_position: anchor_pos,
    };

    if let Err(e) = RsvpRenderer::render_frame(&mut self.kitty_renderer, &frame) {
        eprintln!("Kitty render failed: {}", e);
    }
}
```

---

## 5. Performance Considerations

### 5.1 Buffer Size

For 1920×1080 terminal:
- Per-word image: ~200×920 = 184K pixels = ~250KB base64
- Composite image: ~1920×920 = 1.77M pixels = ~2.4MB base64
- **10x larger**, but still manageable

### 5.2 Target Performance (Per Design Doc)

- Per-frame budget: <10ms
- Rasterization: <3ms
- Encoding: <2ms
- Transmission: <5ms

**Confidence:** Composite approach should meet targets. Word cache (future) will help.

---

## 6. Testing Strategy (TDD)

### 6.1 Unit Tests

```rust
#[test]
fn test_create_canvas_dimensions() {
    // Given viewport 1920×1080
    // Canvas should be 1920×920 (85% height)
}

#[test]
fn test_word_centered_in_canvas() {
    // Canvas: 1000×1000
    // Word at: Y=420 (42%), X=centered
    // Should appear in middle, not bottom
}

#[test]
fn test_clear_resets_image_id() {
    // After clear, image ID should be 1
}

#[test]
fn test_transmit_uses_fixed_image_id() {
    // Always transmit with image ID 1
}
```

### 6.2 Integration Test

Run app, verify:
- Word appears in middle of reading zone (not bottom)
- No flickering between frames
- Background fills reading zone properly
- Clearing removes previous word

---

## 7. Migration Path

**Step 1: Add ReadingCanvas struct** (1 hour)
- Create struct definition
- Add test for dimensions

**Step 2: Implement create_canvas()** (2 hours)
- Fill background
- Test dimensions correct

**Step 3: Implement composite_word()** (2-3 hours)
- Calculate vertical center (42% of zone)
- Calculate horizontal center
- Draw word
- Test positioning correct

**Step 4: Modify render_frame() signature** (1 hour)
- Accept RenderFrame instead of word + anchor_pos
- Orchestrate: clear → create → composite → transmit

**Step 5: Update TuiManager** (1 hour)
- Pass RenderFrame to renderer
- Test integration

**Step 6: Fix coordinate bug** (verify)
- Ensure 42% calculation uses reading zone height, not screen height

**Total Estimate:** 6-8 hours (1 day)

---

## 8. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Large buffer transmission** | Performance | Monitor frame time, add word cache if needed |
| **Coordinate edge cases** | Positioning | Test with various terminal sizes (small/large) |
| **Clearing fails** | Ghost images | Fallback: delete_all_graphics() on error |

---

## 9. Conclusion

**Finding:** Current per-word rendering has coordinate bug. Composite image approach fixes bug and provides extensible canvas.

**Scope:** This epic implements minimum viable composite canvas (background + word centered correctly). Ghost words, anchor coloring, and progress bars are future enhancements.

**Recommendation:** Implement composite approach now. It's straightforward, fixes critical bug, and aligns with design document architecture.

**Next Step:** Follow AGENTS.md workflow → pal_consensus → pal_challenge → pal_thinkdeep to validate plan.

---

## Appendix: Relevant Code Locations

| File | Lines | Purpose |
|------|-------|---------|
| `src/rendering/kitty.rs` | 29-43 | KittyGraphicsRenderer struct |
| `src/rendering/kitty.rs` | 179-219 | rasterize_word() (to be replaced) |
| `src/rendering/kitty.rs` | 342-350 | clear() (to be simplified) |
| `src/rendering/kitty.rs` | 232-264 | transmit_graphics() (reuse) |
| `src/ui/terminal.rs` | 210-235 | TuiManager::render_frame() (to be updated) |
| `src/ui/terminal.rs` | 58-72 | TuiManager::new() coordinate calculation (bug location) |
