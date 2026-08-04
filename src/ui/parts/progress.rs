//! Progress part — the two reading-progress indicators.
//!
//! - `render_bar`: sentence-progress micro bar below the word. Capsule shape
//!   (rounded ends) matching the rounded popup borders; anchored to the
//!   constant `word_line_height()` so its Y position never jitters between
//!   words of different glyph heights.
//! - `render_macro_gutter`: document-progress macro gutter on the right edge.
//!
//! Both share the viewport (sizing + cell mapping) and the protocol helpers;
//! colors are alpha-scaled by pause state (dimmed 30% while reading so the
//! word stays the focus, full opacity when paused).

use crate::engine::config::{
    PROGRESS_BAR_HEIGHT, PROGRESS_BAR_MARGIN_PX, PROGRESS_BAR_WIDTH_PCT, PROGRESS_BRIGHT_ALPHA,
    PROGRESS_COLOR_B, PROGRESS_COLOR_G, PROGRESS_COLOR_R, PROGRESS_DIM_ALPHA,
};
use crate::rendering::kitty::protocol::{encode_image_base64, transmit_graphics};
use crate::rendering::renderer::RendererError;
use crate::ui::parts::word::{move_to_pixel, KittyGraphicsRenderer, BAR_IMAGE_ID, GUTTER_IMAGE_ID};
use imageproc::image::{ImageBuffer, Rgba};
use ratatui::layout::Rect;

/// Bright/dim progress colors, alpha-scaled by pause state.
///
/// Reading dims the indicators (30% alpha) to keep focus on the word; Paused
/// brings them to full opacity so the user can read position at a glance.
fn progress_colors(paused: bool) -> (Rgba<u8>, Rgba<u8>) {
    let alpha_mult: f32 = if paused { 1.0 } else { 0.3 };
    let bright_alpha = (PROGRESS_BRIGHT_ALPHA as f32 * alpha_mult) as u8;
    let dim_alpha = (PROGRESS_DIM_ALPHA as f32 * alpha_mult) as u8;
    (
        Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            bright_alpha,
        ]),
        Rgba([
            PROGRESS_COLOR_R,
            PROGRESS_COLOR_G,
            PROGRESS_COLOR_B,
            dim_alpha,
        ]),
    )
}

/// Build a capsule (pill) bar image: a rectangle with rounded ends, radius =
/// half the height. Anti-aliased via a signed-distance field — the alpha of
/// each pixel is its coverage of the capsule silhouette, so the curved ends
/// stay smooth instead of stair-stepped.
fn build_capsule_bar(
    bar_width: u32,
    bar_height: u32,
    fill_width: u32,
    fill_color: Rgba<u8>,
    bg_color: Rgba<u8>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(bar_width, bar_height);
    if bar_width == 0 || bar_height == 0 {
        return buffer;
    }

    let radius = bar_height as f32 / 2.0;
    let center_y = radius;
    let right_edge = bar_width as f32 - radius;

    for y in 0..bar_height {
        for x in 0..bar_width {
            let px = x as f32 + 0.5;
            let cx = px.clamp(radius, right_edge);
            let d = ((px - cx).powi(2) + (y as f32 + 0.5 - center_y).powi(2)).sqrt() - radius;
            let coverage = (0.5 - d).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue; // outside the capsule: fully transparent
            }
            let color = if x < fill_width { fill_color } else { bg_color };
            let a = (color[3] as f32 * coverage) as u8;
            buffer.put_pixel(x, y, Rgba([color[0], color[1], color[2], a]));
        }
    }
    buffer
}

impl KittyGraphicsRenderer {
    /// Render the sentence-progress bar + document-progress gutter in one pass,
    /// using the fixed slot ids so re-transmission replaces in place (see the
    /// slot-id docs in word.rs — no id churn, no stacking, no flicker).
    #[allow(clippy::too_many_arguments)] // +self = 8; cohesive one-pass renderer API
    pub fn render_progress(
        &mut self,
        word_y: u32,
        word_height: u32,
        progress: f64,
        paused: bool,
        current_index: usize,
        total_tokens: usize,
        reader_area: Rect,
    ) -> Result<(), RendererError> {
        self.render_bar(word_y, word_height, progress, paused, BAR_IMAGE_ID)?;
        self.render_macro_gutter(
            current_index,
            total_tokens,
            reader_area,
            paused,
            GUTTER_IMAGE_ID,
        )?;
        Ok(())
    }

    /// Render the sentence-progress micro bar below the word.
    ///
    /// The bar is a capsule (rounded ends) so it reads soft against the
    /// square terminal grid — matches the rounded popup borders.
    pub fn render_bar(
        &mut self,
        word_y: u32,
        word_height: u32,
        progress: f64,
        paused: bool,
        image_id: u32,
    ) -> Result<(), RendererError> {
        // Bar sits one margin below the word
        let bar_y = word_y + word_height + PROGRESS_BAR_MARGIN_PX;

        let container_width = self
            .viewport()
            .get_dimensions()
            .map(|d| d.pixel_size.0)
            .unwrap_or(800);

        let bar_width = (container_width as f64 * PROGRESS_BAR_WIDTH_PCT) as u32;

        let (fill_color, bg_color) = progress_colors(paused);
        let fill_width = (bar_width as f64 * progress.clamp(0.0, 1.0)) as u32;
        let buffer = build_capsule_bar(
            bar_width,
            PROGRESS_BAR_HEIGHT,
            fill_width,
            fill_color,
            bg_color,
        );

        // Center the bar horizontally in the viewport
        let bar_x = (container_width - bar_width) / 2;
        move_to_pixel(self.viewport(), bar_x, bar_y);
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(
            image_id,
            bar_width,
            PROGRESS_BAR_HEIGHT,
            &base64_data,
            0,
            0,
            1,
        )
        .map_err(|e| RendererError::RenderFailed(format!("Bar render failed: {}", e)))
    }

    /// Render the document-progress macro gutter on the right edge of the reader zone.
    pub fn render_macro_gutter(
        &mut self,
        current_word: usize,
        total_words: usize,
        reader_area: Rect,
        paused: bool,
        image_id: u32,
    ) -> Result<(), RendererError> {
        let progress_ratio = if total_words > 1 {
            (current_word as f32 / (total_words - 1) as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let reader_height = reader_area.height as u32;
        let fill_height = (reader_height as f32 * progress_ratio) as u32;
        let gutter_width: u32 = 4;

        let (read_color, unread_color) = progress_colors(paused);

        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(gutter_width, reader_height);
        for y in 0..reader_height {
            let color = if y < fill_height {
                read_color
            } else {
                unread_color
            };
            for x in 0..gutter_width {
                buffer.put_pixel(x, y, color);
            }
        }

        // Anchor the gutter to the right edge of the reader zone
        let x_position =
            reader_area.x as u32 + (reader_area.width as u32).saturating_sub(gutter_width);
        let y_position = reader_area.y as u32;
        move_to_pixel(self.viewport(), x_position, y_position);
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(image_id, gutter_width, reader_height, &base64_data, 0, 0, 1)
            .map_err(|e| RendererError::RenderFailed(format!("Gutter render failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
        Rgba([r, g, b, a])
    }

    /// Capsule shape: 6px tall → radius 3. Corners must be fully transparent
    /// (rounded), center row full alpha, fill/track split at fill_width.
    #[test]
    fn capsule_bar_rounds_corners_and_splits_fill() {
        let bar = build_capsule_bar(20, 6, 10, solid(255, 0, 0, 200), solid(0, 0, 255, 100));

        // Top-left corner cell is outside the radius-3 circle → transparent
        assert_eq!(bar.get_pixel(0, 0)[3], 0, "corner must be rounded off");
        assert_eq!(bar.get_pixel(0, 5)[3], 0, "bottom corner rounded off");

        // Center of the left end cap is opaque
        assert_eq!(bar.get_pixel(0, 3)[0], 255, "left cap center is fill color");

        // Middle rows are full-width solid (opaque at mid-height)
        assert_eq!(bar.get_pixel(5, 3)[3], 200, "fill center is opaque");

        // Fill/track split at x=10: left fill, right track
        assert_eq!(*bar.get_pixel(9, 3), solid(255, 0, 0, 200), "x=9 is fill");
        assert_eq!(
            *bar.get_pixel(10, 3),
            solid(0, 0, 255, 100),
            "x=10 is track"
        );

        // Fully transparent when empty / degenerate sizes don't panic
        assert_eq!(
            build_capsule_bar(0, 6, 0, solid(255, 0, 0, 255), solid(0, 0, 255, 255)).width(),
            0
        );
        assert_eq!(
            *build_capsule_bar(20, 6, 20, solid(255, 0, 0, 255), solid(0, 0, 255, 255))
                .get_pixel(10, 3),
            solid(255, 0, 0, 255),
            "100% fill"
        );
    }
}
