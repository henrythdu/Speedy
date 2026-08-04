//! Viewport management for graphics rendering
//!
//! Implements the viewport overlay pattern that coordinates Ratatui layout
//! with direct terminal graphics. Queries terminal dimensions using CSI
//! escape sequences (14t for pixels, 18t for cells) to calculate cell
//! dimensions for accurate pixel-to-cell coordinate conversion.

use crossterm::event;
use std::io::{self, Read, Write};
use std::time::Duration;

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

    /// Query terminal dimensions using CSI escape sequences
    ///
    /// Sends:
    /// - CSI 14t: Query text area size in pixels
    /// - CSI 18t: Query cell count
    ///
    /// # Returns
    /// TerminalDimensions if queries succeed, error otherwise
    ///
    /// # Note
    /// This implementation uses a timeout-based approach. If the terminal
    /// doesn't respond within the timeout or parsing fails, falls back
    /// to estimated cell dimensions (10x20 pixels per cell, a common standard).
    pub fn query_dimensions(&mut self) -> Result<TerminalDimensions, ViewportError> {
        // First, try to get terminal size using crossterm
        let size = crossterm::terminal::size()
            .map_err(|e| ViewportError::IoError(format!("Failed to get terminal size: {}", e)))?;

        // Try to query pixel dimensions
        let pixel_size = self.query_pixel_size();

        // If pixel query succeeded, calculate cell dimensions from actual data
        if let Some((width, height)) = pixel_size {
            let _cell_width = width as f32 / size.0 as f32;
            let _cell_height = height as f32 / size.1 as f32;

            let dims = TerminalDimensions::new(width, height, size.0, size.1);
            self.dimensions = Some(dims);
            return Ok(dims);
        }

        // Fallback: Use estimated cell dimensions (10x20 pixels is common)
        let estimated_cell_width = 10.0;
        let estimated_cell_height = 20.0;
        let pixel_width = (size.0 as f32 * estimated_cell_width) as u32;
        let pixel_height = (size.1 as f32 * estimated_cell_height) as u32;

        let dims = TerminalDimensions::new(pixel_width, pixel_height, size.0, size.1);
        self.dimensions = Some(dims);
        Ok(dims)
    }

    /// Try to query terminal pixel size using CSI 14t
    ///
    /// # SAFETY
    /// This function reads directly from stdin which can conflict with crossterm's
    /// event loop. It MUST only be called during initialization (before TuiManager
    /// starts its event loop) or when the event loop is paused.
    ///
    /// # Returns
    /// Some((width, height)) if query succeeds, None otherwise
    fn query_pixel_size(&self) -> Option<(u32, u32)> {
        // Send CSI 14t: Query text area size in pixels
        // Format: ESC [ 14 t
        // Response: ESC [ 4 ; height ; width t
        print!("\x1b[14t");
        io::stdout().flush().ok()?;

        // Try to read response with short timeout
        let timeout = Duration::from_millis(100);
        if event::poll(timeout).ok()? {
            // Try to read from stdin for the CSI response
            // SAFETY: This is safe because we're called during initialization
            // before the main event loop starts (see TuiManager::new)
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 64];

            // Read response (with timeout)
            if stdin.read(&mut buffer).ok()? > 0 {
                let response = String::from_utf8_lossy(&buffer);
                // Parse CSI response: ESC [ 4 ; height ; width t
                if response.contains("\x1b[4;") {
                    let parts: Vec<&str> = response.split(';').collect();

                    if parts.len() >= 3 {
                        // Format: ESC[4;height;widtht
                        let height = parts[1]
                            .trim_matches(|c: char| !c.is_numeric())
                            .parse::<u32>()
                            .ok()?;
                        let width_str = parts[2].trim_matches(|c: char| !c.is_numeric());
                        let width = width_str.parse::<u32>().ok()?;
                        return Some((width, height));
                    }
                }
            }
        }

        None
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
