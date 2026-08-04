//! Viewport management for graphics rendering
//!
//! Implements the viewport overlay pattern that coordinates Ratatui layout
//! with direct terminal graphics. Queries terminal dimensions via the
//! TIOCGWINSZ ioctl (crossterm's window_size) to calculate cell dimensions
//! for accurate pixel-to-cell coordinate conversion.
//!
//! NOTE: never query CSI 14t / read stdin from here. The crossterm 0.29
//! event source deadlocks when a CSI response arrives mid-poll on its
//! blocking tty fd (speedy startup used to hang until the user pressed a
//! key). window_size() is a pure ioctl — safe at startup AND inside the
//! event loop.

use std::io;

/// Terminal dimension information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalDimensions {
    /// Text area size in pixels (width, height)
    pub pixel_size: (u32, u32),
    /// Cell count (columns, rows)
    pub cell_count: (u16, u16),
    /// Cell size in pixels (width, height)
    pub cell_size: (f32, f32),
}

impl TerminalDimensions {
    /// Create new terminal dimensions
    pub fn new(pixel_width: u32, pixel_height: u32, cols: u16, rows: u16) -> Self {
        let cell_width = if cols > 0 {
            pixel_width as f32 / cols as f32
        } else {
            0.0
        };
        let cell_height = if rows > 0 {
            pixel_height as f32 / rows as f32
        } else {
            0.0
        };

        Self {
            pixel_size: (pixel_width, pixel_height),
            cell_count: (cols, rows),
            cell_size: (cell_width, cell_height),
        }
    }
}

/// Viewport manager for coordinating Ratatui layout with graphics rendering
#[derive(Debug, Clone)]
pub struct Viewport {
    dimensions: Option<TerminalDimensions>,
}

impl Viewport {
    /// Create a new viewport manager
    pub fn new() -> Self {
        Self { dimensions: None }
    }

    /// Query terminal dimensions via the TIOCGWINSZ ioctl.
    ///
    /// crossterm's `window_size()` returns cols/rows plus pixel dimensions
    /// (set by terminals like kitty; 0 on plain ptys). Pixel size 0 falls
    /// back to estimated cell dimensions (10x20 pixels per cell).
    ///
    /// # Returns
    /// TerminalDimensions if the ioctl succeeds, error otherwise
    pub fn query_dimensions(&mut self) -> Result<TerminalDimensions, ViewportError> {
        let size = crossterm::terminal::window_size()
            .map_err(|e| ViewportError::IoError(format!("Failed to get terminal size: {}", e)))?;

        let (cols, rows) = (size.columns, size.rows);
        let (pixel_width, pixel_height) = (size.width as u32, size.height as u32);

        if pixel_width > 0 && pixel_height > 0 {
            self.dimensions = Some(TerminalDimensions::new(
                pixel_width,
                pixel_height,
                cols,
                rows,
            ));
        } else {
            // Fallback: Use estimated cell dimensions (10x20 pixels is common)
            let estimated_cell_width = 10.0;
            let estimated_cell_height = 20.0;
            let pixel_width = (cols as f32 * estimated_cell_width) as u32;
            let pixel_height = (rows as f32 * estimated_cell_height) as u32;

            self.dimensions = Some(TerminalDimensions::new(
                pixel_width,
                pixel_height,
                cols,
                rows,
            ));
        }

        Ok(self.dimensions.expect("dimensions set above"))
    }

    /// Get current dimensions if available
    pub fn get_dimensions(&self) -> Option<TerminalDimensions> {
        self.dimensions
    }

    /// Check if dimensions are available
    pub fn has_dimensions(&self) -> bool {
        self.dimensions.is_some()
    }

    /// Update dimensions after terminal resize using event-provided cell count.
    ///
    /// Cell size (px per cell) is stable across resizes in a given terminal
    /// (font size is fixed; only the number of cols/rows changes), so pixel
    /// size is extrapolated from the previous cell_size.
    ///
    /// NOTE: deliberately does NOT re-query CSI 14t here — reading stdin from
    /// inside the crossterm event loop races the event reader and corrupts the
    /// event stream.
    ///
    /// # Arguments
    /// * `cols` - New column count from resize event
    /// * `rows` - New row count from resize event
    pub fn update_from_resize(&mut self, cols: u16, rows: u16) {
        if let Some(existing) = self.dimensions {
            let pixel_width = (cols as f32 * existing.cell_size.0) as u32;
            let pixel_height = (rows as f32 * existing.cell_size.1) as u32;
            self.dimensions = Some(TerminalDimensions::new(
                pixel_width,
                pixel_height,
                cols,
                rows,
            ));
        }
    }

    /// Convert Ratatui Rect to pixel coordinates
    /// Convert pixel coordinates to cell coordinates (for cursor positioning)
    ///
    /// # Arguments
    /// * `x` - Pixel x coordinate
    /// * `y` - Pixel y coordinate
    ///
    /// # Returns
    /// (col, row) cell coordinates if dimensions available, None otherwise
    pub fn pixel_to_cell(&self, x: u32, y: u32) -> Option<(u16, u16)> {
        self.dimensions.map(|d| {
            let col = (x as f32 / d.cell_size.0).floor() as u16;
            let row = (y as f32 / d.cell_size.1).floor() as u16;
            (col, row)
        })
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during viewport operations
#[derive(Debug, Clone, PartialEq)]
pub enum ViewportError {
    /// IO error when communicating with terminal
    IoError(String),
}

impl std::fmt::Display for ViewportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "Viewport IO error: {}", msg),
        }
    }
}

impl std::error::Error for ViewportError {}

impl From<io::Error> for ViewportError {
    fn from(err: io::Error) -> Self {
        ViewportError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_dimensions_creation() {
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);
        assert_eq!(dims.pixel_size, (1920, 1080));
        assert_eq!(dims.cell_count, (80, 24));
        assert_eq!(dims.cell_size.0, 24.0); // 1920 / 80
        assert_eq!(dims.cell_size.1, 45.0); // 1080 / 24
    }

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new();
        assert!(!viewport.has_dimensions());
        assert!(viewport.get_dimensions().is_none());
    }

    #[test]
    fn test_terminal_dimensions_zero_cells() {
        // Edge case: zero cells should not panic
        let dims = TerminalDimensions::new(1920, 1080, 0, 0);
        assert_eq!(dims.cell_size.0, 0.0);
        assert_eq!(dims.cell_size.1, 0.0);
    }

    #[test]
    fn test_update_from_resize_extrapolates_pixel_size() {
        // Resize must preserve cell size and extrapolate pixel size from the
        // previous dimensions — never query stdin (races the event loop).
        let mut viewport = Viewport::new();
        viewport.dimensions = Some(TerminalDimensions::new(1920, 1080, 80, 24));

        viewport.update_from_resize(100, 40);

        let d = viewport.get_dimensions().unwrap();
        assert_eq!(d.cell_count, (100, 40));
        assert_eq!(d.cell_size.0, 24.0); // 1920/80 preserved
        assert_eq!(d.cell_size.1, 45.0); // 1080/24 preserved
        assert_eq!(d.pixel_size, (2400, 1800)); // 100*24, 40*45
    }

    #[test]
    fn test_update_from_resize_without_dimensions_is_noop() {
        // No previous dimensions → resize leaves state untouched (fallback on
        // next query_dimensions).
        let mut viewport = Viewport::new();
        viewport.update_from_resize(100, 40);
        assert!(viewport.dimensions.is_none());
    }
}
