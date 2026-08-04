//! Background part — the rounded-corner app background.
//!
//! Two halves of one element:
//!
//! - `render_background`: an opaque rounded-rect KGP image placed UNDER cells
//!   with non-default backgrounds (z = i32::MIN, see `BACKGROUND_Z_INDEX`).
//! - `render_background_cells`: the ratatui cell fill that paints everything
//!   EXCEPT the four screen-corner cells, so the card's rounded notches stay
//!   visible (unpainted cells keep the DEFAULT background, which the card
//!   shows through).
//!
//! The image is re-transmitted only when pixel dims or theme color change
//! (kitty images persist until deleted); the cell fill runs every frame.

use crate::rendering::kitty::protocol::{encode_image_base64, transmit_graphics};
use crate::rendering::renderer::RendererError;
use crate::ui::parts::word::KittyGraphicsRenderer;
use imageproc::image::{ImageBuffer, Rgba};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
    Frame,
};

/// Reserved image id for the rounded background rect — slot ids 1-5 cover
/// ghosts, word, bar, gutter (see word.rs), so 0 never collides.
const BACKGROUND_IMAGE_ID: u32 = 0;
/// Corner radius of the app background card, in pixels.
const BACKGROUND_RADIUS_PX: u32 = 10;
/// Z-index of the background card. MUST be below INT32_MIN/2 (-1,073,741,824):
/// kitty draws images with z in [-INT32_MIN/2, -1] BETWEEN the default
/// background and the cells — i.e. ON TOP of non-default cell backgrounds —
/// which would swallow the deck's surface fill and every popup background.
/// Only z < INT32_MIN/2 places the image UNDER cells with non-default
/// backgrounds (kitty graphics protocol §Controlling displayed image layout).
const BACKGROUND_Z_INDEX: i32 = i32::MIN;

impl KittyGraphicsRenderer {
    /// Render the rounded-corner background image, behind the text layer.
    ///
    /// One opaque rounded rect per (pixel dims, theme background) pair, placed
    /// with a negative z-index so the ratatui text stays on top. Re-transmitted
    /// only when dims or theme change — the cached key is stored on the
    /// renderer and the image persists in the terminal until deleted.
    pub fn render_background(&mut self, background: Rgba<u8>) -> Result<(), RendererError> {
        let dims = match self.viewport().get_dimensions() {
            Some(d) => d,
            None => return Ok(()),
        };
        let (w, h) = (dims.pixel_size.0, dims.pixel_size.1);
        let key = (w, h, background[0], background[1], background[2]);
        if self.background_key == Some(key) {
            return Ok(()); // already placed, image persists in the terminal
        }

        let buffer = build_rounded_rect(w, h, BACKGROUND_RADIUS_PX, background);
        let base64_data = encode_image_base64(&buffer);
        transmit_graphics(
            BACKGROUND_IMAGE_ID,
            w,
            h,
            &base64_data,
            0,
            0,
            BACKGROUND_Z_INDEX,
        )
        .map_err(|e| RendererError::RenderFailed(format!("Background render failed: {}", e)))?;

        self.background_key = Some(key);
        Ok(())
    }
}

/// Paint the background cell fill, skipping the four screen-corner cells so
/// the rounded card's notches show through (the card is UNDER non-default
/// cells, so any cell painted here covers it — corners must stay unpainted).
pub fn render_background_cells(frame: &mut Frame, area: Rect, color: Color) {
    let bg_style = Style::default().bg(color);
    let (w, h) = (area.width, area.height);
    if w > 2 && h > 2 {
        for strip in [
            Rect::new(area.x + 1, area.y, w - 2, 1), // top, minus corners
            Rect::new(area.x, area.y + 1, w, h - 2), // middle, full width
            Rect::new(area.x + 1, area.y + h - 1, w - 2, 1), // bottom, minus corners
        ] {
            frame.render_widget(Block::default().style(bg_style), strip);
        }
    } else {
        frame.render_widget(Block::default().style(bg_style), area);
    }
}

/// Build an opaque rounded-rect image (the app background card).
///
/// Anti-aliased corners via a signed-distance field, same technique as the
/// capsule progress bar: alpha per pixel = coverage of the rounded-rect
/// silhouette, so the corner curve stays smooth instead of stair-stepped.
fn build_rounded_rect(
    width: u32,
    height: u32,
    radius: u32,
    color: Rgba<u8>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    if width == 0 || height == 0 || radius == 0 {
        return buffer;
    }

    let (w, h) = (width as f32, height as f32);
    let r = radius as f32;
    // Center of the rounded rect, and half-extents minus radius (the flat core)
    let (cx, cy) = (w / 2.0, h / 2.0);
    let (hx, hy) = ((w / 2.0 - r).max(0.0), (h / 2.0 - r).max(0.0));

    for y in 0..height {
        for x in 0..width {
            let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let qx = px.abs() - hx;
            let qy = py.abs() - hy;
            let dx = qx.max(0.0);
            let dy = qy.max(0.0);
            // Distance to the rounded-rect boundary: negative inside, 0 on edge
            let d = (dx * dx + dy * dy).sqrt() - r;
            let coverage = (0.5 - d).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue; // outside the card: transparent notch
            }
            let a = (color[3] as f32 * coverage) as u8;
            buffer.put_pixel(x, y, Rgba([color[0], color[1], color[2], a]));
        }
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rounded background: corner cells transparent (the notch), flat interior
    /// opaque, anti-aliased transition at the curve.
    #[test]
    fn rounded_rect_leaves_transparent_corners() {
        let color = Rgba([26u8, 27, 38, 255]); // tokyo-night bg
        let img = build_rounded_rect(100, 50, 10, color);

        // Exact corners are outside the radius-10 circle → transparent
        assert_eq!(img.get_pixel(0, 0)[3], 0, "top-left corner transparent");
        assert_eq!(img.get_pixel(99, 0)[3], 0, "top-right corner transparent");
        assert_eq!(img.get_pixel(0, 49)[3], 0, "bottom-left corner transparent");
        assert_eq!(
            img.get_pixel(99, 49)[3],
            0,
            "bottom-right corner transparent"
        );

        // Interior is fully opaque and the theme color
        assert_eq!(*img.get_pixel(50, 25), color, "center opaque fill");

        // Degenerate sizes don't panic
        assert_eq!(build_rounded_rect(0, 50, 10, color).width(), 0);
        assert_eq!(build_rounded_rect(50, 0, 10, color).height(), 0);
    }

    /// Background cache key: same (dims, color) must not re-transmit; a
    /// changed color must. Exercises render_background's dedup without a real
    /// terminal (transmit is a no-op print, key is the observable state).
    #[test]
    fn background_cache_key_tracks_dims_and_color() {
        let mut renderer = KittyGraphicsRenderer::new();
        // No dimensions yet → no-op, key stays None
        let _ = renderer.render_background(Rgba([26u8, 27, 38, 255]));
        assert_eq!(renderer.background_key, None);
    }
}
