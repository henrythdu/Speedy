//! Progress indicators rendered via the Kitty Graphics Protocol.
//!
//! Two separate progress displays live here, split out of the word renderer:
//!
//! - `render_bar`: the sentence-progress micro bar drawn below the current word.
//! - `render_macro_gutter`: the document-progress macro gutter on the right edge.
//!
//! Both only need the viewport (for sizing + cell mapping) and the protocol
//! transmission helpers — they share no state with word rasterization, so they
//! form a clean sub-concern of the KGP renderer.

use std::io::{self, Write};

use imageproc::image::{ImageBuffer, Rgba};
use ratatui::layout::Rect;

use crate::engine::config::{
    PROGRESS_BAR_HEIGHT, PROGRESS_BAR_MARGIN_PX, PROGRESS_BAR_WIDTH_PCT, PROGRESS_BRIGHT_ALPHA,
    PROGRESS_COLOR_B, PROGRESS_COLOR_G, PROGRESS_COLOR_R, PROGRESS_DIM_ALPHA,
};
use crate::rendering::kitty::protocol::{encode_image_base64, transmit_graphics};
use crate::rendering::kitty::KittyGraphicsRenderer;
use crate::rendering::renderer::RendererError;

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

impl KittyGraphicsRenderer {
    /// Render the sentence-progress micro bar below the word.
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
        let bar_height = PROGRESS_BAR_HEIGHT;

        let (fill_color, bg_color) = progress_colors(paused);

        let fill_width = (bar_width as f64 * progress.clamp(0.0, 1.0)) as u32;
        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(bar_width, bar_height);
        for x in 0..bar_width {
            let color = if x < fill_width { fill_color } else { bg_color };
            buffer.put_pixel(x, 0, color);
            buffer.put_pixel(x, 1, color);
        }

        // Center the bar horizontally in the viewport
        let bar_x = (container_width - bar_width) / 2;
        self.move_to_pixel(bar_x, bar_y)?;
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(image_id, bar_width, bar_height, &base64_data, 0, 0)
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
        self.move_to_pixel(x_position, y_position)?;
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(image_id, gutter_width, reader_height, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(format!("Gutter render failed: {}", e)))
    }

    /// Move the terminal cursor to a pixel position via the viewport's cell map.
    fn move_to_pixel(&mut self, x: u32, y: u32) -> Result<(), RendererError> {
        if let Some((col, row)) = self.viewport().pixel_to_cell(x, y) {
            print!("\x1b[{};{}H", row + 1, col + 1);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Cursor flush failed: {}",
                    e
                )));
            }
        }
        Ok(())
    }
}
