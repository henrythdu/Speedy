//! Sentence progress bar - simple 2px bar centered below word

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
            width: (container_width as f64 * 0.5) as u32,
            height: 2,
            fill_pct: 0.0,
            fill_color: Rgba([169, 177, 214, 255]), // Theme::text bright
            bg_color: Rgba([169, 177, 214, 50]),    // Theme::text dim (20% opacity)
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
