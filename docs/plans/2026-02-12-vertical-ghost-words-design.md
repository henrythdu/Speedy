# Design Doc: Vertical Ghost Words

**Date:** 2026-02-12
**Status:** Approved for Implementation
**Last Validated:** 2026-02-12 (consensus + challenge + thinkdeep)

---

## 1. Overview

Add vertical ghost words (previous word above, next word below) with anchor-aligned positioning to improve eye tracking continuity and comprehension preview during speed reading.

---

## 2. Visual Layout

```
┌─────────────────────────────────┐
│         (padding)               │
│      pre[v]ious  ← ghost (30%)  │  above current
│        w[o]rd    ← current      │  centered
│        ne[x]t    ← ghost (30%)  │  below current
│         (padding)               │
└─────────────────────────────────┘
```

**Key principle:** All three words' anchor letters share the same X coordinate (ORP center). Eye stays fixed horizontally.

---

## 3. Technical Approach

**Option B: Three Separate Kitty Images**

- Render 3 images per frame at different Y positions
- Reuse existing word cache directly
- Apply opacity at render time (cache stores 100%)

**Rationale:** Pragmatic, reuses existing infrastructure. User will test for flicker at 600+ WPM.

---

## 4. Interface Changes

### 4.1 RenderFrame Struct (Validated Pattern)

Use a `RenderFrame` struct instead of setter methods. This provides:
- Explicit, testable data at call site
- Clear snapshot of what's being rendered
- No hidden state mutations

```rust
/// A single frame to render with optional ghost context
pub struct RenderFrame<'a> {
    /// Current word being displayed
    pub word: &'a str,
    /// Anchor index in current word
    pub anchor: usize,
    /// Previous word ghost (word, anchor)
    pub ghost_prev: Option<(&'a str, usize)>,
    /// Next word ghost (word, anchor)
    pub ghost_next: Option<(&'a str, usize)>,
}
```

### 4.2 RsvpRenderer Trait

```rust
pub trait RsvpRenderer {
    // ... existing methods ...
    
    /// Render a complete frame with optional ghost words
    fn render_frame(&mut self, frame: &RenderFrame) -> Result<(), RendererError>;
}
```

### 4.3 KittyGraphicsRenderer

```rust
pub struct KittyGraphicsRenderer {
    // ... existing fields ...
    ghost_opacity: f32,  // Configurable, default 0.3
}

impl KittyGraphicsRenderer {
    /// Configure ghost word opacity (0.0 - 1.0)
    pub fn set_ghost_opacity(&mut self, opacity: f32) {
        self.ghost_opacity = opacity.clamp(0.0, 1.0);
    }
    
    fn render_at_position(
        &mut self,
        word: &str,
        anchor: usize,
        y: u32,
        opacity: f32,
    ) -> Result<(), RendererError>;
}
```

---

## 5. Render Ordering (Critical for Partial-Render UX)

**Order matters for visual perception and partial-render UX:**

1. **Previous ghost (above)** - transmit first
2. **Next ghost (below)** - transmit second
3. **Current word (center)** - transmit last (most important)

```rust
fn render_frame(&mut self, frame: &RenderFrame) -> Result<()> {
    let center_y = calculate_vertical_center(&self.viewport);
    let line_height = (self.font_size as f32 * 1.5) as u32;
    
    // 1. Previous ghost (above) - non-fatal, transmitted first
    if let Some((gw, ga)) = &frame.ghost_prev {
        let _ = self.render_at_position(gw, *ga, center_y - line_height, self.ghost_opacity);
    }
    
    // 2. Next ghost (below) - non-fatal, transmitted second
    if let Some((gw, ga)) = &frame.ghost_next {
        let _ = self.render_at_position(gw, *ga, center_y + line_height, self.ghost_opacity);
    }
    
    // 3. Current word (center) - CRITICAL: transmitted last for partial-render UX
    // If transmission is interrupted, the most important word is visible
    self.render_at_position(frame.word, frame.anchor, center_y, 1.0)?;
    
    Ok(())
}
```

**Rationale:** If transmission is incomplete (e.g., high WPM, slow terminal), the current word is prioritized. Users see the most important content first.

---

## 6. Renderer Abstraction (Fallback Strategy)

### 6.1 Renderer Trait

Design an abstraction to allow fallback if the multi-transmit approach causes flicker:

```rust
/// Trait for different rendering strategies
pub trait GhostRendererStrategy {
    /// Render a frame with ghost words
    fn render_ghost_frame(
        &mut self,
        base: &mut dyn RsvpRenderer,
        frame: &RenderFrame,
    ) -> Result<(), RendererError>;
}
```

### 6.2 MultiTransmitRenderer (Initial Strategy)

Transmits 3 separate Kitty graphics per frame:

```rust
pub struct MultiTransmitRenderer;

impl GhostRendererStrategy for MultiTransmitRenderer {
    fn render_ghost_frame(
        &mut self,
        base: &mut dyn RsvpRenderer,
        frame: &RenderFrame,
    ) -> Result<(), RendererError> {
        // 3 separate transmit calls (current implementation)
        // ...as shown in Section 5...
    }
}
```

### 6.3 SingleBufferRenderer (Fallback if Flicker)

Composite all 3 words into a single image buffer before transmission:

```rust
pub struct SingleBufferRenderer {
    composite_buffer: Option<RgbaImage>,
}

impl GhostRendererStrategy for SingleBufferRenderer {
    fn render_ghost_frame(
        &mut self,
        base: &mut dyn RsvpRenderer,
        frame: &RenderFrame,
    ) -> Result<(), RendererError> {
        // Composite prev + current + next into single image
        // Apply opacity during composition
        // Single transmit call
    }
}
```

**Decision:** Start with `MultiTransmitRenderer`. If user reports flicker at 600+ WPM, switch to `SingleBufferRenderer`.

---

## 7. Opacity Implementation

**Opacity is configurable**, not hardcoded:

```rust
pub struct GhostConfig {
    /// Ghost word opacity (0.0 = invisible, 1.0 = full)
    /// Default: 0.3
    pub opacity: f32,
    /// Enable/disable ghost words entirely
    /// Default: true
    pub enabled: bool,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            opacity: 0.3,
            enabled: true,
        }
    }
}
```

Apply opacity at render time, not in cache:

```rust
fn render_at_position(&mut self, word: &str, anchor: usize, y: u32, opacity: f32) -> Result<()> {
    let cached = self.word_cache.get_or_render(word, anchor, font, metrics)?;
    
    let buffer = if opacity < 1.0 {
        apply_opacity(&cached.buffer, opacity)
    } else {
        cached.buffer.clone()
    };
    
    let base64 = encode_image_base64(&buffer);
    transmit_graphics(image_id, width, height, &base64, x, y)?;
    // ...
}

fn apply_opacity(buffer: &RgbaImage, opacity: f32) -> RgbaImage {
    let mut result = buffer.clone();
    for pixel in result.pixels_mut() {
        pixel.0[3] = (pixel.0[3] as f32 * opacity) as u8;
    }
    result
}
```

---

## 8. ReadingState Integration

```rust
impl ReadingState {
    /// Create a RenderFrame for the current position
    pub fn create_render_frame(&self) -> RenderFrame {
        let current = self.current_word();
        RenderFrame {
            word: &current.text,
            anchor: current.anchor,
            ghost_prev: self.prev_word.as_ref()
                .map(|w| (w.text.as_str(), w.anchor)),
            ghost_next: self.peek_next()
                .map(|w| (w.text.as_str(), w.anchor)),
        }
    }
}
```

**Caller usage:**

```rust
let frame = reading_state.create_render_frame();
renderer.render_frame(&frame)?;
```

---

## 9. Edge Cases

| Scenario | Behavior |
|----------|----------|
| First word (no prev) | Only render current + next ghost |
| Last word (no next) | Only render prev ghost + current |
| Single word text | Just render current, no ghosts |
| Ghost render fails | Continue without ghost (non-fatal), log warning |
| Small terminal | If viewport < 3 line heights, disable ghosts gracefully |
| Pause/Resume | Ghosts remain visible during pause; clear on resume to first word |
| Ghost failure recovery | If ghost consistently fails >3 times, auto-disable ghosts with user notification |

### 9.1 Small Terminal Handling

```rust
fn should_render_ghosts(&self) -> bool {
    let line_height = (self.font_size as f32 * 1.5) as u32;
    let min_height = line_height * 3; // prev + current + next
    
    self.viewport.height >= min_height && self.ghost_config.enabled
}
```

### 9.2 Pause/Resume Behavior

```rust
impl ReadingState {
    pub fn pause(&mut self) {
        // Ghosts remain visible - user can see context
    }
    
    pub fn resume(&mut self) {
        // Continue from current position
        // Ghosts update naturally on next frame
    }
    
    pub fn reset(&mut self) {
        // Clear ghost context - starting fresh
        self.prev_word = None;
    }
}
```

---

## 10. Files to Modify

| File | Change |
|------|--------|
| `src/rendering/renderer.rs` | Add `RenderFrame` struct + `render_frame()` to trait |
| `src/rendering/kitty/mod.rs` | Implement `render_frame()` + `render_at_position()` + `apply_opacity()` |
| `src/rendering/ghost_config.rs` | (NEW) `GhostConfig` struct with configurable opacity |
| `src/reading_state.rs` | Add `create_render_frame()` method + store prev_word |
| `src/ui/app.rs` (or caller) | Wire up ghost frame rendering |

---

## 11. Testing Plan

1. **Unit tests:** `apply_opacity()` function, `RenderFrame` construction
2. **Manual testing:** Visual verification of ghost positioning
3. **Performance testing:** Check for flicker at 600+ WPM
4. **Edge case testing:** First word, last word, single word, small terminal
5. **Pause/resume testing:** Verify ghosts behave correctly
6. **Fallback testing:** If flicker detected, test `SingleBufferRenderer`

---

## 12. Future Considerations

- [ ] Consider number of ghost words (configurable: 1 above/below vs 2 above/below)
- [ ] Consider different opacity for prev vs next ghost
- [ ] Consider ghost word color tinting (not just opacity)
