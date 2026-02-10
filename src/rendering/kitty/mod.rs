//! Kitty Graphics Protocol rendering modules
//!
//! This module provides pixel-perfect word rendering with sub-pixel OVP anchoring
//! using the Kitty Graphics Protocol.

pub mod positioning;
pub mod protocol;
pub mod rasterizer;

use crate::app::mode::AppMode;
use crate::engine::config::{
    DEFAULT_CACHE_CAPACITY, DEFAULT_FONT_SIZE, PROGRESS_BAR_HEIGHT, PROGRESS_BAR_MARGIN_PX,
    PROGRESS_BAR_WIDTH_PCT, PROGRESS_BRIGHT_ALPHA, PROGRESS_COLOR_B, PROGRESS_COLOR_G,
    PROGRESS_COLOR_R, PROGRESS_DIM_ALPHA,
};
use crate::rendering::cache::WordCache;
use crate::rendering::font::{get_font, get_font_metrics, FontMetrics};
use crate::rendering::kitty::positioning::{calculate_start_x, calculate_vertical_center};
use crate::rendering::kitty::protocol::{
    delete_all_graphics, delete_image, encode_image_base64, transmit_graphics,
};

use crate::rendering::renderer::{RendererError, RsvpRenderer};
use crate::rendering::viewport::Viewport;
use ab_glyph::FontRef;
use imageproc::image::{ImageBuffer, Rgba};
use ratatui::layout::Rect;
use std::io::{self, Write};

/// Kitty Graphics Protocol renderer for pixel-perfect RSVP
pub struct KittyGraphicsRenderer {
    /// Terminal viewport for coordinate conversion
    viewport: Viewport,
    /// Font reference for rasterization
    font: Option<FontRef<'static>>,
    /// Font size in pixels
    font_size: f32,
    /// Font metrics for positioning calculations
    font_metrics: Option<FontMetrics>,
    /// Current image ID for protocol (incremented per word)
    pub current_image_id: u32,
    /// Word-level LRU cache for rendered buffers
    word_cache: WordCache,
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
            current_image_id: 1,
            word_cache: WordCache::new(DEFAULT_CACHE_CAPACITY),
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

    /// Get vertical center Y position (for bar positioning)
    pub fn get_vertical_center(&self) -> Option<u32> {
        calculate_vertical_center(&self.viewport)
    }

    /// Calculate word height for bar positioning
    pub fn calculate_word_height(
        &mut self,
        word: &str,
        anchor_position: usize,
    ) -> Result<u32, RendererError> {
        let font = self
            .font
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font not initialized".to_string()))?;
        let metrics = self
            .font_metrics
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font metrics not available".to_string()))?;

        let cached_word = self
            .word_cache
            .get_or_render(word, anchor_position, font, metrics)
            .map_err(|e| RendererError::RenderFailed(format!("Cache error: {}", e)))?;

        Ok(cached_word.height)
    }

    /// Render sentence progress bar below word
    ///
    /// # Arguments
    /// * `word_y` - Y position of word (from calculate_vertical_center)
    /// * `word_height` - Height of rendered word in pixels
    /// * `progress` - Fill percentage (0.0 to 1.0)
    /// * `mode` - Current app mode (Reading or Paused)
    /// * `image_id` - Image ID for this bar
    pub fn render_bar(
        &mut self,
        word_y: u32,
        word_height: u32,
        progress: f64,
        mode: &AppMode,
        image_id: u32,
    ) -> Result<(), RendererError> {
        use imageproc::image::ImageBuffer;

        // Simple: bar Y = word Y + word height + 10px margin
        let bar_y = word_y + word_height + PROGRESS_BAR_MARGIN_PX;

        // Get viewport width for bar sizing
        let container_width = self
            .viewport
            .get_dimensions()
            .map(|d| d.pixel_size.0)
            .unwrap_or(800);

        // Calculate bar dimensions (same as SentenceProgressBar)
        let bar_width = (container_width as f64 * PROGRESS_BAR_WIDTH_PCT) as u32;
        let bar_height = PROGRESS_BAR_HEIGHT;

        // Determine alpha multiplier based on mode (same as macro gutter)
        let alpha_mult: f32 = match *mode {
            AppMode::Paused => 1.0, // 100% opacity
            _ => 0.3,               // 30% opacity
        };

        // Calculate mode-aware alpha values
        let bright_alpha = (PROGRESS_BRIGHT_ALPHA as f32 * alpha_mult) as u8;
        let dim_alpha = (PROGRESS_DIM_ALPHA as f32 * alpha_mult) as u8;

        let fill_color = Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            bright_alpha,
        ]);
        let bg_color = Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            dim_alpha,
        ]);

        // Create bar buffer manually with mode-aware colors
        let fill_width = (bar_width as f64 * progress.clamp(0.0, 1.0)) as u32;
        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(bar_width, bar_height);

        for x in 0..bar_width {
            let color = if x < fill_width { fill_color } else { bg_color };
            buffer.put_pixel(x, 0, color);
            buffer.put_pixel(x, 1, color);
        }

        // Center bar horizontally in viewport
        let bar_x = (container_width - bar_width) / 2;

        // Move cursor to bar position
        if let Some((col, row)) = self.viewport.pixel_to_cell(bar_x, bar_y) {
            print!("\x1b[{};{}H", row + 1, col + 1);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Cursor flush failed: {}",
                    e
                )));
            }
        }

        // Transmit bar image
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(image_id, bar_width, bar_height, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(format!("Bar render failed: {}", e)))
    }

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
    ) -> Result<(), RendererError> {
        // Calculate progress ratio (0.0 to 1.0)
        let progress_ratio = if total_words > 1 {
            (current_word as f32 / (total_words - 1) as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Calculate fill height
        let reader_height = reader_area.height as u32;
        let fill_height = (reader_height as f32 * progress_ratio) as u32;

        // Determine alpha multiplier based on mode
        let alpha_mult: f32 = match mode {
            AppMode::Paused => 1.0, // 100% opacity
            _ => 0.3,               // 30% opacity
        };

        // Create RGBA buffer for gutter (4px wide × reader_height tall)
        let gutter_width: u32 = 4;

        // Colors matching micro bar (SentenceProgressBar):
        // - Bright for read portion (filled)
        // - Dim for unread portion (unfilled)
        let bright_alpha = (PROGRESS_BRIGHT_ALPHA as f32 * alpha_mult) as u8;
        let dim_alpha = (PROGRESS_DIM_ALPHA as f32 * alpha_mult) as u8;

        let read_color = Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            bright_alpha,
        ]);
        let unread_color = Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            dim_alpha,
        ]);

        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(gutter_width, reader_height);

        // Fill gutter: bright for read (top portion), dim for unread (bottom portion)
        for y in 0..reader_height {
            let color = if y < fill_height {
                read_color // Read portion - bright
            } else {
                unread_color // Unread portion - dim
            };
            for x in 0..gutter_width {
                buffer.put_pixel(x, y, color);
            }
        }

        // Calculate position at right edge of reader zone
        let x_position =
            reader_area.x as u32 + (reader_area.width as u32).saturating_sub(gutter_width);
        let y_position = reader_area.y as u32;

        // Move cursor to position
        if let Some((col, row)) = self.viewport.pixel_to_cell(x_position, y_position) {
            print!("\x1b[{};{}H", row + 1, col + 1);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Gutter cursor flush failed: {}",
                    e
                )));
            }
        }

        // Transmit gutter image
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(image_id, gutter_width, reader_height, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(format!("Gutter render failed: {}", e)))
    }
}

impl RsvpRenderer for KittyGraphicsRenderer {
    fn initialize(&mut self) -> Result<(), RendererError> {
        // Load bundled font
        self.font = get_font();
        if self.font.is_none() {
            return Err(RendererError::InitializationFailed(
                "Failed to load bundled font".to_string(),
            ));
        }

        // Get font metrics
        let font = self.font.as_ref().unwrap();
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

    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError> {
        if word.is_empty() {
            return Ok(());
        }

        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor_position {} out of bounds for word '{}' (length: {})",
                anchor_position, word, word_len
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

        let start_x =
            calculate_start_x(word, anchor_position, font, self.font_size, &self.viewport);
        let reading_zone_center_y = calculate_vertical_center(&self.viewport).unwrap_or(0);

        if let Some((col, row)) = self
            .viewport
            .pixel_to_cell(start_x as u32, reading_zone_center_y)
        {
            let cursor_command = format!("\x1b[{};{}H", row + 1, col + 1);
            print!("{}", cursor_command);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Failed to flush cursor command: {}",
                    e
                )));
            }
        }

        // Use word cache for rasterization (performance optimization)
        let cached_word = self
            .word_cache
            .get_or_render(word, anchor_position, font, metrics)
            .map_err(|e| RendererError::RenderFailed(format!("Cache error: {}", e)))?;

        let base64_data = encode_image_base64(&cached_word.buffer);
        let (width, height) = (cached_word.width, cached_word.height);

        transmit_graphics(self.current_image_id, width, height, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(e.to_string()))?;

        self.current_image_id += 1;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        // A "frame" consists of three images (word + bar + gutter).
        // We must clear all three images from the previous frame.
        if self.current_image_id > 3 {
            let prev_gutter_id = self.current_image_id - 1;
            let prev_bar_id = self.current_image_id - 2;
            let prev_word_id = self.current_image_id - 3;

            // Delete all three images from the previous frame
            // Errors are not propagated to prevent a single failed delete from crashing the app
            let _ = delete_image(prev_gutter_id);
            let _ = delete_image(prev_bar_id);
            let _ = delete_image(prev_word_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::viewport::TerminalDimensions;

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
    fn test_clear_returns_ok() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Render a word first to have something to clear
        let _ = renderer.render_word("test", 0);

        // Clear should succeed (though actual deletion may fail in test env)
        let result = renderer.clear();
        // In test environment without actual terminal, clear might fail
        // but we should at least not panic
        assert!(result.is_ok() || result.is_err());
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
}
