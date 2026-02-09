//! Sentence progress bar - simple 2px bar centered below word

use crate::engine::config::{
    PROGRESS_BAR_HEIGHT, PROGRESS_BAR_WIDTH_PCT, PROGRESS_BRIGHT_ALPHA, PROGRESS_COLOR_B,
    PROGRESS_COLOR_G, PROGRESS_COLOR_R, PROGRESS_DIM_ALPHA,
};
use imageproc::image::{ImageBuffer, Rgba};

/// Simple 2px progress bar with bright fill, dim background
pub struct SentenceProgressBar {
    width: u32,
    height: u32,
    fill_pct: f64,
    fill_color: Rgba<u8>, // Bright for read portion
    bg_color: Rgba<u8>,   // Dim for unread portion
}

impl SentenceProgressBar {
    pub fn new(container_width: u32) -> Self {
        Self {
            width: (container_width as f64 * PROGRESS_BAR_WIDTH_PCT) as u32,
            height: PROGRESS_BAR_HEIGHT,
            fill_pct: 0.0,
            fill_color: Rgba([PROGRESS_COLOR_R, PROGRESS_COLOR_G, PROGRESS_COLOR_B, PROGRESS_BRIGHT_ALPHA]),
            bg_color: Rgba([PROGRESS_COLOR_R, PROGRESS_COLOR_G, PROGRESS_COLOR_B, PROGRESS_DIM_ALPHA]),
        }
    }

    pub fn update_progress(&mut self, pct: f64) {
        self.fill_pct = pct.clamp(0.0, 1.0);
    }

    /// Render the bar: bright filled portion + dim background
    pub fn render(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let fill_width = (self.width as f64 * self.fill_pct) as u32;
        let mut buffer = ImageBuffer::new(self.width, self.height);

        for x in 0..self.width {
            let color = if x < fill_width {
                self.fill_color
            } else {
                self.bg_color
            };
            buffer.put_pixel(x, 0, color);
            buffer.put_pixel(x, 1, color);
        }

        buffer
    }

    pub fn width(&self) -> u32 {
        self.width
    }
}
