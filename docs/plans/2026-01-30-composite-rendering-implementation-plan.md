# Implementation Plan: Composite Rendering (Phase 1)

**Epic:** Fix Word Positioning via Composite Canvas  
**Timeline:** 1 day (6-8 hours)  
**Status:** Ready for bead creation  
**Validation:** ✅ Consensus → ✅ Challenge → ✅ Thinkdeep  

---

## 1. Executive Summary

**Objective:** Fix the coordinate calculation bug where words appear too low on screen by implementing composite canvas rendering (full-zone RGBA buffer with background fill + centered word).

**Alignment:**
- **PRD Section 4.2:** Implements "Pixel-Perfect Anchor System" with correct 42% positioning
- **Design Doc v2.0 Section 10:** Phase 1 "Foundation" - Basic word rendering (no ghosting)
- **Investigation:** Confirmed coordinate bug at `terminal.rs:58` using full screen height instead of reading zone height

**Scope:** Minimum viable composite canvas - background + word. Ghost words, anchor coloring, progress bars are **Phase 2 enhancements** (follow-up epic).

---

## 2. Current State Analysis

### 2.1 Bug Location
```rust
// src/ui/terminal.rs:58-63 (BUG)
let center_y = (dims.pixel_size.1 as f32 * 0.42) as u32;
// ^^ 42% of FULL SCREEN HEIGHT (wrong reference)
```

**Impact:** Words render at bottom of screen (near command deck) instead of middle of reading zone.

### 2.2 Current Rendering Flow
1. `TuiManager::render_frame()` calls `KittyGraphicsRenderer::render_word()`
2. `render_word()` creates per-word image (word_width × reading_zone_height)
3. Image positioned via pixel coordinates (x, y) - but y is wrong
4. Previous image cleared by ID before new render

### 2.3 Target State
1. `TuiManager::render_frame()` calls `KittyGraphicsRenderer::render_frame()`
2. Create full-zone canvas (terminal_width × reading_zone_height)
3. Fill background with theme color (#1A1B26)
4. Center word at 42% of canvas height (correct reference)
5. Transmit entire canvas as single image (image ID 1)
6. Clear: delete image ID 1, redraw

---

## 3. Implementation Tasks

### Task 1: Create ReadingCanvas Struct ⏱️ 1 hour
**Purpose:** Define full-zone RGBA buffer structure

**Acceptance Criteria:**
- [ ] `ReadingCanvas` struct defined in `src/rendering/kitty.rs`
- [ ] Contains `buffer: RgbaImage` field
- [ ] Contains `dimensions: (u32, u32)` field
- [ ] Unit test: `test_canvas_dimensions()` validates (width, height)

**Code Changes:**
```rust
// Add to src/rendering/kitty.rs after imports

/// Full reading zone canvas for composite rendering
/// 
/// Big Picture: Replaces per-word images with single full-zone buffer
/// PRD Reference: Section 4.2 - "Pixel-Perfect Anchor System"
/// Connections: Used by KittyGraphicsRenderer::render_frame()
pub struct ReadingCanvas {
    pub buffer: RgbaImage,
    pub dimensions: (u32, u32),
}

impl ReadingCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            buffer: ImageBuffer::new(width, height),
            dimensions: (width, height),
        }
    }
}
```

**Test:**
```rust
#[test]
fn test_canvas_dimensions() {
    let canvas = ReadingCanvas::new(1920, 920);
    assert_eq!(canvas.dimensions, (1920, 920));
    assert_eq!(canvas.buffer.width(), 1920);
    assert_eq!(canvas.buffer.height(), 920);
}
```

---

### Task 2: Implement create_canvas() Method ⏱️ 1.5 hours
**Purpose:** Create full-zone canvas with correct dimensions

**Acceptance Criteria:**
- [ ] `create_canvas()` method added to `KittyGraphicsRenderer`
- [ ] Calculates canvas dimensions: full width × 85% of screen height
- [ ] Returns `ReadingCanvas` with RGBA buffer
- [ ] Unit test: `test_create_canvas_dimensions()` validates 85% height calculation

**Code Changes:**
```rust
// Add to impl KittyGraphicsRenderer in src/rendering/kitty.rs

/// Create full-zone canvas for reading area
/// 
/// Purpose: Replaces per-word image creation with full-zone buffer
/// PRD Reference: Section 4.3 - "Reader Zone (Top 85%)"
/// Connections: Called by render_frame() each frame
fn create_canvas(&self) -> Result<ReadingCanvas, RendererError> {
    let dims = self.viewport
        .get_dimensions()
        .ok_or_else(|| RendererError::RenderFailed(
            "Viewport dimensions not available".to_string()
        ))?;

    // Canvas = full terminal width × reading zone height (85% of screen)
    let canvas_width = dims.pixel_size.0;
    let canvas_height = (dims.pixel_size.1 as f32 * 0.85) as u32;

    Ok(ReadingCanvas::new(canvas_width, canvas_height))
}
```

**Test:**
```rust
#[test]
fn test_create_canvas_dimensions() {
    let mut renderer = KittyGraphicsRenderer::new();
    renderer.initialize().unwrap();
    
    // Set viewport to 1920×1080
    let dims = TerminalDimensions::new(1920, 1080, 80, 30);
    renderer.viewport.set_dimensions(dims);
    
    let canvas = renderer.create_canvas().unwrap();
    
    // Should be 1920×918 (85% of 1080)
    assert_eq!(canvas.dimensions.0, 1920);
    assert_eq!(canvas.dimensions.1, (1080.0 * 0.85) as u32);
}
```

---

### Task 3: Implement composite_word() Method ⏱️ 2 hours
**Purpose:** Fill background and draw centered word

**Acceptance Criteria:**
- [ ] `composite_word()` method added to `KittyGraphicsRenderer`
- [ ] Fills background with theme color (#1A1B26)
- [ ] Centers word vertically at 42% of canvas height (FIXES BUG)
- [ ] Centers word horizontally based on word width
- [ ] Uses ab_glyph and imageproc for text rendering
- [ ] Unit test: `test_word_positioned_correctly()` validates 42% centering

**Code Changes:**
```rust
/// Composite word onto canvas with background fill
/// 
/// Purpose: Draw background + word into full-zone canvas
/// PRD Reference: 
///   - Section 4.2: "Reading line centered at 42% of Reader Zone height"
///   - Section 4.1: "Background #1A1B26 (Stormy Dark)"
///   - Section 4.1: "Text #A9B1D6 (Light Blue)"
///   - Section 4.1: "Anchor #F7768E (Coral Red)"
/// Connections: Called by render_frame() after create_canvas()
fn composite_word(
    &self, 
    canvas: &mut ReadingCanvas, 
    word: &str,
    _anchor_position: usize  // Phase 1: ignore anchor coloring
) -> Result<(), RendererError> {
    // Validate font is loaded
    let font = self.font
        .as_ref()
        .ok_or_else(|| RendererError::RenderFailed("Font not loaded".to_string()))?;

    // 1. Fill background with theme color (#1A1B26)
    let bg_color = Rgba([26, 27, 38, 255]);
    for pixel in canvas.buffer.pixels_mut() {
        *pixel = bg_color;
    }

    // 2. Calculate word dimensions
    let word_width = calculate_string_width(font, word, self.font_size);
    let word_height = self.font_metrics
        .as_ref()
        .map(|m| m.height)
        .unwrap_or(self.font_size);

    // 3. Calculate vertical position - CENTER AT 42% OF READING ZONE (BUG FIX)
    // PRD Section 4.2: "Reading line centered at 42% of Reader Zone height"
    let zone_center_y = (canvas.dimensions.1 as f32 * 0.42) as i32;
    let word_y = zone_center_y - (word_height / 2.0) as i32;

    // 4. Calculate horizontal position - center word in canvas
    let canvas_center_x = (canvas.dimensions.0 / 2) as i32;
    let word_x = canvas_center_x - (word_width / 2.0) as i32;

    // 5. Draw word (Phase 1: single color, Phase 2: anchor coloring)
    let scale = PxScale::from(self.font_size);
    let text_color = Rgba([247, 118, 142, 255]); // Coral red (#F7768E)
    
    draw_text_mut(
        &mut canvas.buffer,
        text_color,
        word_x.max(0),  // Ensure non-negative
        word_y.max(0),
        scale,
        font,
        word
    );

    Ok(())
}
```

**Test:**
```rust
#[test]
fn test_word_positioned_at_42_percent() {
    let mut renderer = KittyGraphicsRenderer::new();
    renderer.initialize().unwrap();
    
    let dims = TerminalDimensions::new(1000, 1000, 80, 30);
    renderer.viewport.set_dimensions(dims);
    renderer.set_reading_zone_center(500, 420); // 42% of 1000
    
    let mut canvas = renderer.create_canvas().unwrap();
    renderer.composite_word(&mut canvas, "HELLO", 2).unwrap();
    
    // Canvas is 1000×850 (85% of 1000)
    // Word should be centered at Y=357 (42% of 850)
    // Verify some pixels are drawn in the middle area
    let center_y = (850.0 * 0.42) as u32;
    let center_x = 500;
    
    // Check that text color exists near center
    let pixel = canvas.buffer.get_pixel(center_x, center_y);
    // Text is coral red, so R and G should be high, B lower
    assert!(pixel[0] > 200, "Text should be drawn at center");
}
```

---

### Task 4: Implement render_frame() Orchestrator ⏱️ 1 hour
**Purpose:** Orchestrate clear → create → composite → transmit pipeline

**Acceptance Criteria:**
- [ ] `render_frame()` method added to `RsvpRenderer` trait
- [ ] Implements full pipeline: clear → create → composite → transmit
- [ ] Replaces existing `render_word()` in TuiManager
- [ ] Uses fixed image ID 1 for all transmissions
- [ ] Integration test: validates word appears in center of reading zone

**Code Changes:**

**Update trait in src/rendering/renderer.rs:**
```rust
pub trait RsvpRenderer {
    // ... existing methods ...
    
    /// NEW: Render full frame via composite canvas
    /// 
    /// Purpose: Orchestrate composite rendering pipeline
    /// PRD Reference: Section 6.2 "The Rendering Pipeline"
    /// Connections: Called by TuiManager::render_frame()
    fn render_frame(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError>;
}
```

**Implement in src/rendering/kitty.rs:**
```rust
impl RsvpRenderer for KittyGraphicsRenderer {
    // ... existing methods ...

    /// Render complete frame via composite canvas
    /// 
    /// Purpose: Replace per-word rendering with full-zone composite
    /// Big Picture: Fixes coordinate bug by using canvas-relative positioning
    /// PRD Reference: Section 6.2 "The Rendering Pipeline"
    /// Design Doc: Phase 1 Foundation (Section 10)
    /// Connections: Called by TuiManager, uses create_canvas() and composite_word()
    fn render_frame(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError> {
        // 1. Clear previous frame
        self.clear()?;

        // 2. Create canvas for this frame
        let mut canvas = self.create_canvas()?;

        // 3. Composite word onto canvas
        self.composite_word(&mut canvas, word, anchor_position)?;

        // 4. Encode and transmit
        self.transmit_composite_canvas(&canvas)?;

        Ok(())
    }
}
```

**Add transmit method:**
```rust
/// Transmit composite canvas via Kitty Graphics Protocol
/// 
/// Purpose: Send full-zone image to terminal
/// Connections: Called by render_frame(), uses transmit_graphics()
fn transmit_composite_canvas(&self, canvas: &ReadingCanvas) -> io::Result<()> {
    let base64_data = self.encode_image_base64(&canvas.buffer);
    
    // Position at (0, 0) - canvas fills reading zone from top-left
    self.transmit_graphics(
        1,  // Fixed image ID 1 (always use same ID)
        canvas.dimensions.0,
        canvas.dimensions.1,
        &base64_data,
        0,  // x = 0
        0,  // y = 0
    )
}
```

**Update clear() to use fixed ID:**
```rust
fn clear(&mut self) -> Result<(), RendererError> {
    // Always delete image ID 1 (our composite canvas)
    self.delete_image(1)
        .map_err(|e| RendererError::ClearFailed(
            format!("Failed to clear composite canvas: {}", e)
        ))?;
    
    // Reset to fixed ID 1 (don't increment)
    self.current_image_id = 1;
    
    Ok(())
}
```

---

### Task 5: Update TuiManager Integration ⏱️ 1 hour
**Purpose:** Wire composite rendering into TuiManager event loop

**Acceptance Criteria:**
- [ ] Update `TuiManager::render_frame()` to use `render_frame()` instead of `render_word()`
- [ ] Remove coordinate calculation bug from `TuiManager::new()`
- [ ] Integration test: run app, verify word centered in reading zone

**Code Changes:**

**Update src/ui/terminal.rs:220-228:**
```rust
pub fn render_frame(&mut self, app: &App) -> io::Result<()> {
    let render_state = app.get_render_state();

    // Render word via Kitty Graphics Protocol (composite canvas)
    if let Some(word) = &render_state.current_word {
        let anchor_pos = crate::reading::calculate_anchor_position(word);

        // Use new render_frame() method (composite canvas approach)
        if let Err(e) = RsvpRenderer::render_frame(
            &mut self.kitty_renderer, 
            word, 
            anchor_pos
        ) {
            eprintln!("Kitty render failed: {}", e);
        }
    }

    // Always render via Ratatui for UI (commands, etc.)
    self.terminal.draw(|frame| {
        let area = frame.area();

        // Split screen: Reading zone (top 85%) + Command deck (bottom 15%)
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(area);

        let command_area = main_layout[1];

        // Reading zone is rendered via Kitty Graphics Protocol (composite canvas)
        // Command deck area
        render_command_deck(frame, command_area, app.mode(), &self.command_buffer);
    })?;

    Ok(())
}
```

**Update src/ui/terminal.rs:58-72 (REMOVE BUG):**
```rust
// BEFORE (BUG):
// let center_y = (dims.pixel_size.1 as f32 * 0.42) as u32;  // 42% of FULL SCREEN

// AFTER (FIXED):
// Don't calculate reading_zone_center.y here - it's now canvas-relative
// The 42% calculation happens in composite_word() using canvas height
if let Some(dims) = renderer.viewport().get_dimensions() {
    let center_x = dims.pixel_size.0 / 2;
    // Let reading_zone_center.y be 0 for now - composite_word() will calculate
    // actual Y position based on canvas height
    renderer.set_reading_zone_center(center_x, 0);
    
    // Calculate font size for 5-line height
    renderer.calculate_font_size_from_cell_height(dims.cell_size.1);
}
```

---

### Task 6: Verify Coordinate Fix ⏱️ 0.5 hours
**Purpose:** Confirm word appears in middle of reading zone

**Acceptance Criteria:**
- [ ] Run app with `cargo run -- --force-kitty`
- [ ] Verify word appears at 42% of reading zone height (not full screen)
- [ ] Verify background fills entire reading zone
- [ ] Verify no ghost images (clearing works)

**Manual Test:**
```bash
# 1. Run app
cargo run -- --force-kitty

# 2. Load test content
# Type in command deck: @@
# (Loads from clipboard - have some text copied)

# 3. Verify visual:
# - Word should appear in middle of screen (not near bottom)
# - Background should fill top 85% of terminal
# - Word should advance without flickering

# 4. Check debug output (if enabled):
# Should see word centered at correct Y position
```

**Expected Result:**
```
Terminal Height: 1000px
Reading Zone: 850px (85%)
42% of 850px = 357px from top

Word should appear at Y≈357, not Y≈420 (which would be 42% of 1000px)
```

---

## 4. Testing Strategy

### Unit Tests (TDD)
Each task includes unit tests following AGENTS.md TDD requirements:

| Test | Purpose | Validation |
|------|---------|------------|
| `test_canvas_dimensions` | Verify ReadingCanvas struct | Correct (width, height) |
| `test_create_canvas_dimensions` | Verify 85% height calculation | Full width × 85% height |
| `test_word_positioned_at_42_percent` | Verify bug fix | Word at 42% of zone |
| `test_clear_resets_image_id` | Verify clearing works | ID reset to 1 |
| `test_transmit_uses_fixed_image_id` | Verify fixed ID | Always ID 1 |

### Integration Test
```rust
#[test]
fn test_composite_rendering_pipeline() {
    // 1. Initialize renderer
    // 2. Render word via render_frame()
    // 3. Verify canvas dimensions correct
    // 4. Verify word positioned at 42%
    // 5. Verify background filled
    // 6. Verify image transmitted
}
```

### Manual Verification
Run app and visually confirm:
- [ ] Word in middle of reading zone
- [ ] Background fills zone properly  
- [ ] No flicker between frames
- [ ] Clearing removes previous word

---

## 5. Timeline & Dependencies

| Task | Duration | Dependencies | Deliverable |
|------|----------|--------------|-------------|
| 1. ReadingCanvas struct | 1 hour | None | Struct + tests |
| 2. create_canvas() | 1.5 hours | Task 1 | Method + tests |
| 3. composite_word() | 2 hours | Task 2 | Method + tests |
| 4. render_frame() | 1 hour | Task 3 | Orchestrator + tests |
| 5. TuiManager update | 1 hour | Task 4 | Integration |
| 6. Verify fix | 0.5 hours | Task 5 | Manual test |
| **Total** | **7 hours** | **Sequential** | **Phase 1 Complete** |

---

## 6. Acceptance Criteria

### Technical
- [ ] Word appears at 42% of reading zone height (not full screen)
- [ ] Background fills reading zone with #1A1B26
- [ ] Canvas dimensions: full width × 85% height
- [ ] Clearing uses fixed image ID 1
- [ ] All unit tests pass
- [ ] No compiler warnings

### Functional
- [ ] App runs without errors
- [ ] Word advances at WPM rate
- [ ] No ghost images between frames
- [ ] Background consistent across frames

### Documentation
- [ ] Architecture doc updated with new methods
- [ ] Code comments explain "why" not "what"
- [ ] Test coverage >80% for new code

---

## 7. Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Buffer too large** | Performance | Low | Profile frame time, reduce if >10ms |
| **Font metrics wrong** | Positioning | Medium | Add debug logging for Y position |
| **Clearing fails** | Ghost images | Low | Fallback to delete_all_graphics() |
| **Integration issues** | App crashes | Low | Extensive testing in TuiManager |

---

## 8. Future Work (Phase 2)

After Phase 1 is complete and validated:

| Feature | Effort | PRD Section |
|---------|--------|-------------|
| Ghost words (15% opacity) | 1-2 days | 4.2 Three-Container Model |
| Anchor character coloring | 1 day | 4.2 ORP Anchoring |
| Progress bars | 1-2 days | 4.4 Progress & Spatial Awareness |
| Word-level LRU cache | 2 days | 6.1 Performance |

These will be separate epics once composite canvas is stable.

---

## 9. Document References

- **Investigation:** `docs/investigations/2026-01-30-composite-rendering-investigation.md`
- **PRD:** `docs/PRD.md` (Sections 4.2, 4.3, 6.2)
- **Design Doc:** `docs/plans/2026-01-28-TUI Design Doc v2.md` (Section 10, Phase 1)
- **Architecture:** `docs/ARCHITECTURE.md` (to be updated)

---

**Ready for bead creation!** Break into individual tasks using `bd create` for each Task 1-6 above.
