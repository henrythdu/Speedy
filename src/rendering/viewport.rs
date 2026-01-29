//! Viewport management for graphics rendering
//!
//! Implements the viewport overlay pattern that coordinates Ratatui layout
//! with direct terminal graphics. Queries terminal dimensions using CSI
//! escape sequences (14t for pixels, 18t for cells) to calculate cell
//! dimensions for accurate pixel-to-cell coordinate conversion.

use std::io::{self, Write};

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

    /// Convert cell coordinates to pixel coordinates
    ///
    /// # Arguments
    /// * `col` - Column index (0-based)
    /// * `row` - Row index (0-based)
    ///
    /// # Returns
    /// (x, y) pixel coordinates from top-left corner
    pub fn cell_to_pixel(&self, col: u16, row: u16) -> (u32, u32) {
        let x = (col as f32 * self.cell_size.0) as u32;
        let y = (row as f32 * self.cell_size.1) as u32;
        (x, y)
    }

    /// Convert a rectangular area in cells to pixel coordinates
    ///
    /// # Arguments
    /// * `x` - Column start (0-based)
    /// * `y` - Row start (0-based)
    /// * `width` - Width in columns
    /// * `height` - Height in rows
    ///
    /// # Returns
    /// (x, y, width, height) in pixels
    pub fn rect_to_pixel(&self, x: u16, y: u16, width: u16, height: u16) -> (u32, u32, u32, u32) {
        let (pixel_x, pixel_y) = self.cell_to_pixel(x, y);
        let pixel_width = (width as f32 * self.cell_size.0) as u32;
        let pixel_height = (height as f32 * self.cell_size.1) as u32;
        (pixel_x, pixel_y, pixel_width, pixel_height)
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

    /// Query terminal dimensions using CSI escape sequences
    ///
    /// Sends:
    /// - CSI 14t: Query text area size in pixels
    /// - CSI 18t: Query cell count
    ///
    /// # Returns
    /// TerminalDimensions if queries succeed, error otherwise
    pub fn query_dimensions(&mut self) -> Result<TerminalDimensions, ViewportError> {
        // Send CSI 14t to get pixel dimensions
        // Format: ESC [ 14 t
        // Response: ESC [ 4 ; height ; width t
        print!("\x1b[14t");
        io::stdout().flush()?;

        // Send CSI 18t to get cell count
        // Format: ESC [ 18 t
        // Response: ESC [ 8 ; rows ; cols t
        print!("\x1b[18t");
        io::stdout().flush()?;

        // Note: In a real implementation, we'd read the terminal response
        // For now, return an error indicating this needs async/tty handling
        Err(ViewportError::NotImplemented(
            "Terminal response parsing requires async/tty handling".to_string(),
        ))
    }

    /// Set dimensions directly (for testing or manual configuration)
    pub fn set_dimensions(&mut self, dimensions: TerminalDimensions) {
        self.dimensions = Some(dimensions);
    }

    /// Get current dimensions if available
    pub fn get_dimensions(&self) -> Option<TerminalDimensions> {
        self.dimensions
    }

    /// Check if dimensions are available
    pub fn has_dimensions(&self) -> bool {
        self.dimensions.is_some()
    }

    /// Convert Ratatui Rect to pixel coordinates
    ///
    /// # Arguments
    /// * `x` - Column start
    /// * `y` - Row start
    /// * `width` - Width in columns
    /// * `height` - Height in rows
    ///
    /// # Returns
    /// Some((x, y, width, height)) in pixels if dimensions available, None otherwise
    pub fn convert_rect_to_pixels(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Option<(u32, u32, u32, u32)> {
        self.dimensions
            .map(|d| d.rect_to_pixel(x, y, width, height))
    }

    /// Clear stored dimensions
    pub fn clear(&mut self) {
        self.dimensions = None;
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
    /// Terminal response parsing failed
    ParseError(String),
    /// Feature not yet implemented
    NotImplemented(String),
    /// No dimensions available
    NoDimensions,
}

impl std::fmt::Display for ViewportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "Viewport IO error: {}", msg),
            Self::ParseError(msg) => write!(f, "Viewport parse error: {}", msg),
            Self::NotImplemented(msg) => write!(f, "Viewport feature not implemented: {}", msg),
            Self::NoDimensions => write!(f, "No terminal dimensions available"),
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
    fn test_cell_to_pixel_conversion() {
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);

        // Cell (0, 0) should be at pixel (0, 0)
        assert_eq!(dims.cell_to_pixel(0, 0), (0, 0));

        // Cell (40, 12) should be at roughly center
        let (x, y) = dims.cell_to_pixel(40, 12);
        assert_eq!(x, 960); // 40 * 24
        assert_eq!(y, 540); // 12 * 45
    }

    #[test]
    fn test_rect_to_pixel_conversion() {
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);

        // Rect at (10, 5) with size (20, 10)
        let (x, y, w, h) = dims.rect_to_pixel(10, 5, 20, 10);
        assert_eq!(x, 240); // 10 * 24
        assert_eq!(y, 225); // 5 * 45
        assert_eq!(w, 480); // 20 * 24
        assert_eq!(h, 450); // 10 * 45
    }

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new();
        assert!(!viewport.has_dimensions());
        assert!(viewport.get_dimensions().is_none());
    }

    #[test]
    fn test_viewport_set_dimensions() {
        let mut viewport = Viewport::new();
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);

        viewport.set_dimensions(dims);
        assert!(viewport.has_dimensions());
        assert_eq!(viewport.get_dimensions(), Some(dims));
    }

    #[test]
    fn test_viewport_convert_rect() {
        let mut viewport = Viewport::new();
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);
        viewport.set_dimensions(dims);

        // Convert a rect in the middle of screen
        let result = viewport.convert_rect_to_pixels(20, 10, 40, 4);
        assert!(result.is_some());

        let (x, y, w, h) = result.unwrap();
        assert_eq!(x, 480); // 20 * 24
        assert_eq!(y, 450); // 10 * 45
        assert_eq!(w, 960); // 40 * 24
        assert_eq!(h, 180); // 4 * 45
    }

    #[test]
    fn test_viewport_convert_without_dimensions() {
        let viewport = Viewport::new();
        let result = viewport.convert_rect_to_pixels(10, 10, 20, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_viewport_clear() {
        let mut viewport = Viewport::new();
        let dims = TerminalDimensions::new(1920, 1080, 80, 24);

        viewport.set_dimensions(dims);
        assert!(viewport.has_dimensions());

        viewport.clear();
        assert!(!viewport.has_dimensions());
    }

    #[test]
    fn test_terminal_dimensions_zero_cells() {
        // Edge case: zero cells should not panic
        let dims = TerminalDimensions::new(1920, 1080, 0, 0);
        assert_eq!(dims.cell_size.0, 0.0);
        assert_eq!(dims.cell_size.1, 0.0);
    }
}
