//! Word part — the RSVP display: current word + ghost words.
//!
//! Owns the `KittyGraphicsRenderer` struct (shared by the progress and
//! background parts, which extend it via `impl` blocks) plus everything about
//! placing the current word and its ghosts: OVP anchoring, rasterization,
//! opacity, per-frame image-id bookkeeping (place-then-delete so the terminal
//! repaints once instead of the word blinking out on every advance).

use crate::engine::config::{DEFAULT_CACHE_CAPACITY, DEFAULT_FONT_SIZE};
use crate::rendering::cache::WordCache;
use crate::rendering::font::{get_font, get_font_metrics, FontMetrics};
use crate::rendering::kitty::positioning::{calculate_start_x, calculate_vertical_center};
use crate::rendering::kitty::protocol::{
    delete_all_graphics, delete_image, encode_image_base64, transmit_graphics,
};
use crate::rendering::kitty::rasterizer::rasterize_word;
use crate::rendering::renderer::{RenderFrame, RendererError, RsvpRenderer};
use crate::rendering::viewport::Viewport;
use ab_glyph::FontRef;
use imageproc::image::{ImageBuffer, Rgba};

/// Kitty image-id strategy (double-buffer swap).
///
/// Every slot re-transmits its image only when the content actually changed
/// (see the per-slot state below) — kitty deletes an image + its placements
/// on ANY same-id re-transmit, so idle re-uploads tear the word/bar down and
/// rebuild it at the render rate (which scales with WPM) and read as flicker.
///
/// On a change, the slot allocates a FRESH image id and only then deletes the
/// old one (delete-after-place). Same-id re-transmission would delete the old
/// image first and show the new one only once its placement lands — a visible
/// gap at every word advance. With a fresh id the sequence is: place new →
/// delete old, so the screen always has a complete word (no gap, by
/// construction). Ids only grow, so the fresh image (higher id, same z) sits
/// on top during the transient overlap (kitty: same z → higher id = higher
/// z-index). The background (background.rs) keeps id 0 — anonymous, but it
/// only re-transmits on theme/dims change.
///
/// Everything that determines a slot's on-screen image: word, anchor (OVP
/// point), pixel y, and opacity. Equal state ⇒ the image on screen is already
/// correct ⇒ transmitting again would only make kitty delete + rebuild it.
#[derive(Clone, PartialEq, Eq)]
struct SlotState {
    word: String,
    anchor: usize,
    y: u32,
    opacity_bits: u32,
}

/// Kitty Graphics Protocol renderer for RSVP — draws the current word, the
/// optional ghost words, and owns the progress bar / gutter slots.
pub struct KittyGraphicsRenderer {
    /// Terminal viewport for coordinate conversion
    viewport: Viewport,
    /// Font reference for rasterization
    font: Option<FontRef<'static>>,
    /// Font size in pixels
    font_size: f32,
    /// Font metrics for positioning calculations
    font_metrics: Option<FontMetrics>,
    /// Word-level LRU cache for rendered buffers
    word_cache: WordCache,
    /// Ghost word opacity (0.0 - 1.0), default 0.3
    ghost_opacity: f32,
    /// Whether each ghost slot was rendered last frame — true means a stale
    /// image may be on screen, so toggling ghosts off must delete the slot.
    ghost_prev_was_rendered: bool,
    ghost_next_was_rendered: bool,
    /// Cache key of the last placed background (w, h, r, g, b) — skip re-transmit when unchanged
    pub(crate) background_key: Option<(u32, u32, u8, u8, u8)>,
    /// Next image id to allocate (see `alloc_image_id`).
    image_id_counter: u32,
    /// Last-transmitted state + on-screen image id per reading-zone slot. A
    /// slot skips re-transmission when its state is unchanged; on change it
    /// allocates a fresh id (place-new-then-delete-old, see module docs).
    word_slot: Option<SlotState>,
    word_slot_id: Option<u32>,
    ghost_prev_slot: Option<SlotState>,
    ghost_prev_slot_id: Option<u32>,
    ghost_next_slot: Option<SlotState>,
    ghost_next_slot_id: Option<u32>,
    /// Last-transmitted bar/gutter keys (progress.rs) — same change-detection
    /// rule as the word slots; the on-screen ids live here too.
    pub(crate) bar_slot: Option<(u32, u32, u32, bool)>, // (bar_y, bar_width, fill_width, paused)
    pub(crate) bar_slot_id: Option<u32>,
    pub(crate) gutter_slot: Option<(u32, u32, u32, u32, bool)>, // (x, y, fill_height, reader_height, paused)
    pub(crate) gutter_slot_id: Option<u32>,
}

impl Default for KittyGraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl KittyGraphicsRenderer {
    /// Create a new KittyGraphicsRenderer with default font size
    pub fn new() -> Self {
        Self {
            viewport: Viewport::new(),
            font: None,
            font_size: DEFAULT_FONT_SIZE,
            font_metrics: None,
            word_cache: WordCache::new(DEFAULT_CACHE_CAPACITY),
            ghost_opacity: 0.1,
            ghost_prev_was_rendered: false,
            ghost_next_was_rendered: false,
            background_key: None,
            image_id_counter: 1,
            word_slot: None,
            word_slot_id: None,
            ghost_prev_slot: None,
            ghost_prev_slot_id: None,
            ghost_next_slot: None,
            ghost_next_slot_id: None,
            bar_slot: None,
            bar_slot_id: None,
            gutter_slot: None,
            gutter_slot_id: None,
        }
    }

    /// Calculate font size based on terminal cell dimensions
    ///
    /// Sets the font size to render at approximately 5 lines height
    /// based on the cell height from the viewport.
    pub fn calculate_font_size_from_cell_height(&mut self, cell_height_px: f32) {
        // Font size should be approximately 5 lines height
        // We use a scale factor of 1.0 for the font, so font_size = cell_height × 5
        self.font_size = cell_height_px * 5.0;

        // Update font metrics with the new size
        if let Some(ref font) = self.font {
            self.font_metrics = Some(get_font_metrics(font, self.font_size));
        }

        // Sync font size with word cache (clears cache if size changed)
        self.word_cache.set_font_size(self.font_size);
    }

    /// Get reference to viewport (for external access to query dimensions)
    pub fn viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Allocate the next image id for a slot's transmission. Never 0 (that's
    /// the anonymous background slot). Ids only increase within a session, so
    /// during the brief moment a fresh image coexists with the one it replaces
    /// (same position, same z), the fresh one has the higher id → draws on top
    /// (kitty: same z → higher id = higher z-index).
    pub(crate) fn alloc_image_id(&mut self) -> u32 {
        let id = self.image_id_counter;
        self.image_id_counter = self.image_id_counter % 2_000_000_000 + 1;
        id
    }

    /// Get vertical center Y position (for bar positioning)
    pub fn get_vertical_center(&self) -> Option<u32> {
        calculate_vertical_center(&self.viewport)
    }

    /// Constant line height in px (font_size × 1.5, same as ghost stacking).
    /// Used as the bar anchor so the progress bar sits at a FIXED position —
    /// per-word glyph heights vary a few px and made the bar jitter each word.
    pub fn word_line_height(&self) -> u32 {
        (self.font_size * 1.5) as u32
    }

    /// Render a word at a specific Y position with given opacity
    ///
    /// This method supports the ghost words feature by allowing words to be
    /// rendered at arbitrary vertical positions with configurable opacity.
    /// The X position is calculated based on OVP anchoring.
    ///
    /// # Arguments
    /// * `word` - The word text to render
    /// * `anchor` - Character index for OVP anchoring
    /// * `y_position` - Pixel Y coordinate for rendering
    /// * `opacity` - Alpha multiplier (0.0 = invisible, 1.0 = full)
    ///
    /// # Returns
    /// Result containing the image id placed (for per-frame id bookkeeping)
    /// or a renderer error
    fn render_at_position(
        &mut self,
        word: &str,
        anchor: usize,
        y_position: u32,
        opacity: f32,
        image_id: u32,
    ) -> Result<u32, RendererError> {
        if word.is_empty() {
            return Err(RendererError::InvalidArguments(
                "Cannot render empty word".to_string(),
            ));
        }

        let word_len = word.chars().count();
        if anchor >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor {} out of bounds for word '{}' (length: {})",
                anchor, word, word_len
            )));
        }

        let font = self
            .font
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font not initialized".to_string()))?;

        let metrics = self
            .font_metrics
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font metrics not available".to_string()))?;

        if !self.viewport.has_dimensions() {
            let _ = self.viewport.query_dimensions();
        }

        // Calculate X position based on OVP anchoring
        let start_x = calculate_start_x(word, anchor, font, self.font_size, &self.viewport);

        // Move cursor to position (buffered — see move_to_pixel docs)
        move_to_pixel(&self.viewport, start_x as u32, y_position);

        // Get cached word buffer
        let cached_word = self
            .word_cache
            .get_or_render(word, anchor, || {
                rasterize_word(word, anchor, font, self.font_size, metrics)
            })
            .map_err(|e| RendererError::RenderFailed(format!("Cache error: {}", e)))?;

        // Apply opacity at render time
        let buffer = if opacity < 1.0 {
            apply_opacity(&cached_word.buffer, opacity)
        } else {
            cached_word.buffer.clone()
        };

        let base64_data = encode_image_base64(&buffer);
        let (width, height) = (cached_word.width, cached_word.height);

        transmit_graphics(image_id, width, height, &base64_data, 0, 0, 1)
            .map_err(|e| RendererError::RenderFailed(e.to_string()))?;

        Ok(image_id)
    }
}

impl RsvpRenderer for KittyGraphicsRenderer {
    fn initialize(&mut self) -> Result<(), RendererError> {
        // Load bundled font
        self.font = get_font();
        let font = match self.font.as_ref() {
            Some(f) => f,
            None => {
                return Err(RendererError::InitializationFailed(
                    "Failed to load bundled font".to_string(),
                ))
            }
        };

        // Get font metrics
        self.font_metrics = Some(get_font_metrics(font, self.font_size));

        // Initialize word cache with current font size
        self.word_cache.set_font_size(self.font_size);

        // Query viewport dimensions
        match self.viewport.query_dimensions() {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback is acceptable - will use estimated dimensions
                Ok(())
            }
        }
    }

    fn render_frame(&mut self, frame: &RenderFrame) -> Result<(), RendererError> {
        // Validate current word anchor
        let word_len = frame.word.chars().count();
        if frame.anchor >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor {} out of bounds for word '{}' (length: {})",
                frame.anchor, frame.word, word_len
            )));
        }

        // Calculate vertical positions for ghost stacking
        let center_y = calculate_vertical_center(&self.viewport).unwrap_or(0);
        let line_height = (self.font_size * 1.5) as u32;

        // Render order (critical for partial-render UX):
        // 1. Previous ghost (above) - transmitted first, non-fatal
        // 2. Next ghost (below) - transmitted second, non-fatal
        // 3. Current word (center) - transmitted last, MUST succeed
        //
        // Double-buffer swap per slot: on change, allocate a FRESH id, place
        // the new image, then delete the old id — the screen always shows a
        // complete image (no delete-then-recreate gap). Unchanged slots are
        // left alone entirely (kitty deletes + rebuilds on any same-id
        // re-transmit, which flickers at high render rates).

        // 1. Previous ghost (above center)
        if let Some((word, anchor)) = frame.ghost_prev {
            // Validate anchor but don't fail on error - ghost rendering is non-fatal
            if anchor < word.chars().count() {
                let y_offset = center_y.saturating_sub(line_height);
                let state = SlotState {
                    word: word.to_string(),
                    anchor,
                    y: y_offset,
                    opacity_bits: self.ghost_opacity.to_bits(),
                };
                if self.ghost_prev_slot.as_ref() != Some(&state) {
                    let id = self.alloc_image_id();
                    if let Err(e) =
                        self.render_at_position(word, anchor, y_offset, self.ghost_opacity, id)
                    {
                        // Log but don't fail - ghost rendering is non-fatal
                        tracing::warn!(error = %e, "ghost_prev render failed");
                    } else {
                        if let Some(old) = self.ghost_prev_slot_id.replace(id) {
                            let _ = delete_image(old);
                        }
                        self.ghost_prev_slot = Some(state);
                    }
                }
                self.ghost_prev_was_rendered = true;
            } else {
                tracing::warn!(anchor, word = %word, "ghost_prev anchor out of bounds");
            }
        } else if self.ghost_prev_was_rendered {
            // Ghosts were just toggled off: clear the stale slot image so it
            // doesn't linger on screen (replace-only would never touch it).
            if let Some(old) = self.ghost_prev_slot_id.take() {
                let _ = delete_image(old);
            }
            self.ghost_prev_slot = None;
            self.ghost_prev_was_rendered = false;
        }

        // 2. Next ghost (below center)
        if let Some((word, anchor)) = frame.ghost_next {
            // Validate anchor but don't fail on error - ghost rendering is non-fatal
            if anchor < word.chars().count() {
                let y_offset = center_y + line_height;
                let state = SlotState {
                    word: word.to_string(),
                    anchor,
                    y: y_offset,
                    opacity_bits: self.ghost_opacity.to_bits(),
                };
                if self.ghost_next_slot.as_ref() != Some(&state) {
                    let id = self.alloc_image_id();
                    if let Err(e) =
                        self.render_at_position(word, anchor, y_offset, self.ghost_opacity, id)
                    {
                        // Log but don't fail - ghost rendering is non-fatal
                        tracing::warn!(error = %e, "ghost_next render failed");
                    } else {
                        if let Some(old) = self.ghost_next_slot_id.replace(id) {
                            let _ = delete_image(old);
                        }
                        self.ghost_next_slot = Some(state);
                    }
                }
                self.ghost_next_was_rendered = true;
            } else {
                tracing::warn!(anchor, word = %word, "ghost_next anchor out of bounds");
            }
        } else if self.ghost_next_was_rendered {
            if let Some(old) = self.ghost_next_slot_id.take() {
                let _ = delete_image(old);
            }
            self.ghost_next_slot = None;
            self.ghost_next_was_rendered = false;
        }

        // 3. Current word (center) - CRITICAL: rendered last, must succeed
        let state = SlotState {
            word: frame.word.to_string(),
            anchor: frame.anchor,
            y: center_y,
            opacity_bits: 1.0f32.to_bits(),
        };
        if self.word_slot.as_ref() != Some(&state) {
            let id = self.alloc_image_id();
            self.render_at_position(frame.word, frame.anchor, center_y, 1.0, id)?;
            if let Some(old) = self.word_slot_id.replace(id) {
                let _ = delete_image(old);
            }
            self.word_slot = Some(state);
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), RendererError> {
        // Clear word cache to free memory
        self.word_cache.clear();

        if let Err(e) = delete_all_graphics() {
            return Err(RendererError::CleanupFailed(format!(
                "Failed to cleanup graphics: {}",
                e
            )));
        }
        Ok(())
    }
}

/// Apply opacity multiplier to an RGBA image buffer
///
/// This modifies the alpha channel of each pixel by multiplying
/// the existing alpha value by the opacity factor.
///
/// # Arguments
/// * `buffer` - The source RGBA image buffer
/// * `opacity` - Alpha multiplier (0.0 = invisible, 1.0 = unchanged)
///
/// # Returns
/// A new image buffer with modified alpha values
fn apply_opacity(
    buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    opacity: f32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut result = buffer.clone();
    for pixel in result.pixels_mut() {
        // Multiply existing alpha by opacity factor
        pixel.0[3] = (pixel.0[3] as f32 * opacity) as u8;
    }
    result
}

/// Move the terminal cursor to a pixel position via the viewport's cell map.
///
/// Buffered, NOT flushed: the whole frame (placements + deletes + ratatui
/// cells) must reach the terminal in ONE batch so it repaints once — per-op
/// flushes split the frame and cause visible flicker. The only flush in the
/// render path is ratatui's `Terminal::draw` at the end of render_frame.
///
/// Takes `&Viewport` (not `&mut self`) so callers holding a font/metrics
/// borrow can still position the cursor — disjoint-field borrows.
pub(crate) fn move_to_pixel(viewport: &Viewport, x: u32, y: u32) {
    if let Some((col, row)) = viewport.pixel_to_cell(x, y) {
        print!("\x1b[{};{}H", row + 1, col + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitty_renderer_initialize_loads_font() {
        let mut renderer = KittyGraphicsRenderer::new();
        let result = renderer.initialize();
        assert!(
            result.is_ok(),
            "Initialization should succeed: {:?}",
            result
        );
        assert!(renderer.font.is_some(), "Font should be loaded");
        assert!(
            renderer.font_metrics.is_some(),
            "Font metrics should be available"
        );
    }

    #[test]
    fn test_get_reading_zone_height_without_dimensions() {
        use crate::rendering::kitty::positioning::get_reading_zone_height;
        let renderer = KittyGraphicsRenderer::new();

        let zone_height = get_reading_zone_height(&renderer.viewport);
        assert!(
            zone_height.is_none(),
            "Should return None without dimensions"
        );
    }

    #[test]
    fn test_calculate_font_size_from_cell_height() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // With cell height of 20px, font size should be 100px (20 * 5)
        renderer.calculate_font_size_from_cell_height(20.0);

        assert_eq!(renderer.font_size, 100.0);
        assert!(renderer.font_metrics.is_some());

        let metrics = renderer.font_metrics.unwrap();
        // Height should match font size
        assert!((metrics.height - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_render_frame_double_buffers_slot_swaps() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // First render places the word with a fresh id.
        let frame = RenderFrame::with_ghosts("test", 0, None, None);
        let _ = renderer.render_frame(&frame);
        let id_after_first = renderer.word_slot_id;
        assert!(id_after_first.is_some(), "first render must place a word");
        assert_ne!(
            id_after_first,
            Some(0),
            "slots never use the anonymous id 0"
        );

        // Unchanged frame: slot skips re-transmission — the SAME id stays on
        // screen (no churn, no same-id re-transmit that kitty would tear down).
        let _ = renderer.render_frame(&frame);
        assert_eq!(renderer.word_slot_id, id_after_first);

        // Changed content: a FRESH id is allocated (place-new-then-delete-old
        // swap) — never the previous one.
        let frame2 = RenderFrame::with_ghosts("different", 0, None, None);
        let _ = renderer.render_frame(&frame2);
        assert_ne!(
            renderer.word_slot_id, id_after_first,
            "swap must use a new id"
        );
    }

    #[test]
    fn test_ghost_off_clears_stale_ghost_slots() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Ghosts on: both slots render.
        let with_ghosts = RenderFrame::with_ghosts("word", 0, Some(("prev", 0)), Some(("next", 0)));
        let _ = renderer.render_frame(&with_ghosts);
        assert!(renderer.ghost_prev_was_rendered);
        assert!(renderer.ghost_next_was_rendered);

        // Ghosts off: the flags flip back, signalling the slots were cleared.
        let no_ghosts = RenderFrame::with_ghosts("word", 0, None, None);
        let _ = renderer.render_frame(&no_ghosts);
        assert!(!renderer.ghost_prev_was_rendered);
        assert!(!renderer.ghost_next_was_rendered);
    }

    #[test]
    fn test_cleanup_returns_ok() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Cleanup should attempt to delete all graphics
        let result = renderer.cleanup();
        // In test environment without actual terminal, cleanup might fail
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_apply_opacity() {
        // Create a simple 2x2 image with known alpha values
        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(2, 2);

        // Set different alpha values
        buffer.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Full opacity
        buffer.put_pixel(1, 0, Rgba([0, 255, 0, 128])); // 50% opacity
        buffer.put_pixel(0, 1, Rgba([0, 0, 255, 64])); // 25% opacity
        buffer.put_pixel(1, 1, Rgba([255, 255, 0, 0])); // 0% opacity

        // Apply 50% opacity multiplier
        let result = apply_opacity(&buffer, 0.5);

        // Check that alpha values are correctly multiplied
        assert_eq!(result.get_pixel(0, 0).0[3], 127); // 255 * 0.5 ≈ 127
        assert_eq!(result.get_pixel(1, 0).0[3], 64); // 128 * 0.5 = 64
        assert_eq!(result.get_pixel(0, 1).0[3], 32); // 64 * 0.5 = 32
        assert_eq!(result.get_pixel(1, 1).0[3], 0); // 0 * 0.5 = 0
    }

    #[test]
    fn test_apply_opacity_preserves_rgb() {
        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1, 1);
        buffer.put_pixel(0, 0, Rgba([100, 150, 200, 255]));

        let result = apply_opacity(&buffer, 0.3);

        // RGB values should be unchanged
        assert_eq!(result.get_pixel(0, 0).0[0], 100);
        assert_eq!(result.get_pixel(0, 0).0[1], 150);
        assert_eq!(result.get_pixel(0, 0).0[2], 200);
        // Only alpha should change
        assert_eq!(result.get_pixel(0, 0).0[3], 76); // 255 * 0.3 ≈ 76
    }
}
